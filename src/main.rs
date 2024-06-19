#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
// I can't do anything about this; fault of the dependencies
#![allow(clippy::multiple_crate_versions)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

use crate::gitlab_api::types::{Response, ResponseNode};
use chrono::{DateTime, Datelike, Local, NaiveDate, Weekday};
use clap::Parser;
use nu_ansi_term::{Color, Style};
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde_json::json;
use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

mod cli;
mod gitlab_api;

const GRAPHQL_TEMPLATE: &str = include_str!("./gitlab-query.graphql");

/// Performs a single request against the GitLab API, getting exactly one page
/// of the paged data source.
fn fetch_result(username: &str, host: &str, token: &str, before: Option<&str>) -> Response {
    let graphql_query = GRAPHQL_TEMPLATE
        .replace("%USERNAME%", username)
        .replace("%BEFORE%", before.unwrap_or_default());
    let payload = json!({ "query": graphql_query });

    let authorization = format!("Bearer {token}", token = token);
    let url = format!("https://{host}/api/graphql", host = host);
    let client = Client::new();

    client
        .post(url)
        .header(AUTHORIZATION, authorization)
        .json(&payload)
        .send()
        .unwrap()
        .json::<Response>()
        .unwrap()
}

/// Fetches all results from the API with pagination in mind.
fn fetch_all_results(username: &str, host: &str, token: &str) -> Response {
    let base = fetch_result(username, host, token, None);

    let mut aggregated = base;
    while aggregated.data.timelogs.pageInfo.hasPreviousPage {
        let mut next = fetch_result(
            username,
            host,
            token,
            Some(&aggregated.data.timelogs.pageInfo.startCursor),
        );

        // Ordering here is not that important, happens later anyway.
        next.data
            .timelogs
            .nodes
            .extend(aggregated.data.timelogs.nodes);
        aggregated = next;
    }
    aggregated
}

fn main() {
    let cli = cli::CliArgs::parse();

    let res = fetch_all_results(cli.username(), cli.host(), cli.token());
    let dates = aggregate_and_sort_dates(&res, cli.days());

    if dates.is_empty() {
        print_warning(
            "Empty response. Is the username correct? Does the token has read permission?",
            0,
        );
    } else {
        for (i, (&week, dates_per_week)) in dates.iter().enumerate() {
            let is_last = i == dates.len() - 1;

            print_week(week, dates_per_week, &res);
            if !is_last {
                println!();
            }
        }
    }
}

/// Returns a sorted list from oldest to newest date with records for the last
/// `days_n` days.
fn find_dates(res: &Response, days_n: usize) -> BTreeSet<NaiveDate> {
    let days = res
        .data
        .timelogs
        .nodes
        .iter()
        .map(|node| parse_gitlab_datetime(&node.spentAt))
        .collect::<BTreeSet<_>>();

    let days_n = min(days.len(), days_n);
    let skip = days.len() - days_n;

    days.into_iter().skip(skip).collect()
}

/// Aggregates and sorts the dates with records from the response from GitLab
/// so that we get a sorted collection of weeks and a sorted collection of each
/// day with entries per week.
///
/// This only takes the last `days_n` days into account.
fn aggregate_and_sort_dates(
    response: &Response,
    days_n: usize,
) -> BTreeMap<(i32 /* year */, u32 /* iso week */), BTreeSet<NaiveDate>> {
    let mut week_to_dates_map = BTreeMap::new();

    let dates = find_dates(response, days_n);

    for date in dates.iter().copied() {
        let week = date.iso_week().week();
        let key = (date.year(), week);
        week_to_dates_map
            .entry(key)
            .and_modify(|set: &mut BTreeSet<NaiveDate>| {
                set.insert(date);
            })
            .or_insert_with(|| {
                let mut set = BTreeSet::new();
                set.insert(date);
                set
            });
    }

    week_to_dates_map
}

/// Parses the UTC timestring coming from GitLab in the local timezone of
/// the user. This is necessary so that entries accoutned to a Monday on `00:00`
/// in CEST are not displayed as Sunday.
///
/// # Parameters
/// - `datestring` in GitLab format such as `"2024-06-09T22:00:00Z"`.
fn parse_gitlab_datetime(datestring: &str) -> NaiveDate {
    let date = DateTime::parse_from_rfc3339(datestring).unwrap();
    let date = DateTime::<Local>::from(date);
    // simplify
    date.naive_local().date()
}

fn calc_total_time_per_day(date: &NaiveDate, res: &Response) -> Duration {
    find_logs_of_day(date, res)
        .map(|node| node.timeSpent().1)
        .sum()
}

fn find_logs_of_day<'a>(
    date: &'a NaiveDate,
    res: &'a Response,
) -> impl Iterator<Item = &'a ResponseNode> {
    res.data.timelogs.nodes.iter().filter(|node| {
        let node_date = parse_gitlab_datetime(&node.spentAt);
        node_date == *date
    })
}

fn print_timelog(log: &ResponseNode) {
    let (duration_is_positive, duration) = log.timeSpent();
    print!("  ");
    print_duration(duration, Color::Magenta);
    println!(
        "  {issue_name}",
        issue_name = Style::new()
            .bold()
            .fg(Color::Green)
            .paint(log.issue.title.clone()),
    );
    let min_minutes_threshold = 15;
    if !duration_is_positive {
        // msg is aligned with the suspicious data output
        print_warning(
            "^ ERROR: You have logged this time as NEGATIVE: Update the ticket!",
            3,
        );
    }
    if duration.as_secs() / 60 < min_minutes_threshold {
        // msg is aligned with the suspicious data output
        print_warning("^ WARN: Less than 15 minutes! Is this correct?", 6);
    }

    // Print issue metadata.
    let epic_name = log
        .issue
        .epic
        .as_ref()
        .map(|e| e.title.as_str())
        .unwrap_or("<no epic>");
    let whitespace = " ".repeat(11);
    println!(
        "{whitespace}{link}",
        link = Style::new().dimmed().paint(&log.issue.webUrl)
    );
    if let Some(group) = &log.project.group {
        println!(
            "{whitespace}[{epic_key} {epic_name}, {group_key} {group_name}]",
            epic_key = Style::new().dimmed().paint("Epic:"),
            epic_name = Style::new().bold().paint(epic_name),
            group_key = Style::new().dimmed().paint("Group:"),
            group_name = Style::new().bold().paint(&group.fullName),
            whitespace = " ".repeat(11),
        );
    }

    if let Some(lines) = log.summary.as_ref().map(|t| t.lines()) {
        for line in lines {
            println!("             {line}");
        }
    }
}

fn print_warning(msg: &str, indention: usize) {
    println!(
        "{indention}{msg}",
        indention = " ".repeat(indention),
        msg = Style::new().bold().fg(Color::Yellow).paint(msg),
    );
}

fn print_date(day: &NaiveDate, data: &Response) {
    let total = calc_total_time_per_day(day, data);

    let day_print = format!("{day}, {}", day.weekday());

    print!("{}  (", Style::new().bold().paint(day_print));
    print_duration(total, Color::Blue);
    println!(")");

    // Sanity checks and print warnings
    {
        let max_hours_threshold = 10;
        if total.as_secs() > max_hours_threshold * 60 * 60 {
            // msg is aligned with the suspicious data output
            print_warning("^ WARN: More than 10 hours! Is this correct?", 18);
        }

        match day.weekday() {
            Weekday::Sat | Weekday::Sun => {
                // msg is aligned with the suspicious data output
                print_warning("^ WARN: You shouldn't work on the weekend, right?", 12);
            }
            _ => {}
        }
    }

    for log in find_logs_of_day(day, data) {
        print_timelog(log);
    }
}

fn print_week(
    week: (i32 /* year */, u32 /* iso week */),
    dates: &BTreeSet<NaiveDate>,
    data: &Response,
) {
    let week_style = Style::new().bold();
    let week_print = format!("WEEK {}-W{:>2}", week.0, week.1);
    println!(
        "{delim} {week_print} {delim}",
        delim = week_style.paint("======================"),
        week_print = week_style.paint(week_print)
    );
    let total_week_time = dates
        .iter()
        .map(|date| calc_total_time_per_day(date, data))
        .sum::<Duration>();
    print!(
        "{total_time_key}       ",
        total_time_key = Style::new().bold().paint("Total time:")
    );
    print_duration(total_week_time, Color::Blue);
    println!();
    println!();

    for (i, date) in dates.iter().enumerate() {
        print_date(date, data);

        let is_last = i == dates.len() - 1;
        if !is_last {
            println!();
        }
    }
}

const fn duration_to_hhmm(dur: Duration) -> (u64, u64) {
    let hours = dur.as_secs() / 60 / 60;
    let remaining_secs = dur.as_secs() - (hours * 60 * 60);
    let minutes = remaining_secs / 60;
    (hours, minutes)
}

fn print_duration(duration: Duration, color: Color) {
    let (hours, minutes) = duration_to_hhmm(duration);
    let print_str = format!("{hours:>2}h {minutes:02}m");
    print!("{}", Style::new().bold().fg(color).paint(print_str));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_to_hhmm() {
        assert_eq!(duration_to_hhmm(Duration::from_secs(0)), (0, 0));
        assert_eq!(duration_to_hhmm(Duration::from_secs(59)), (0, 0));
        assert_eq!(duration_to_hhmm(Duration::from_secs(60)), (0, 1));
        assert_eq!(duration_to_hhmm(Duration::from_secs(61)), (0, 1));
        assert_eq!(duration_to_hhmm(Duration::from_secs(119)), (0, 1));
        assert_eq!(duration_to_hhmm(Duration::from_secs(120)), (0, 2));
        let h = 3;
        let m = 7;
        assert_eq!(
            duration_to_hhmm(Duration::from_secs(h * 60 * 60 + m * 60)),
            (h, m)
        );
    }
}
