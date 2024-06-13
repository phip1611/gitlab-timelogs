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
use std::collections::BTreeSet;
use std::time::Duration;

mod cli;
mod gitlab_api;

const GRAPHQL_TEMPLATE: &str = include_str!("./gitlab-query.graphql");

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

/// Fetches all results from the API. If necessary and requested, this also uses
/// the pagination feature to get really all results. This however takes some
/// more time.
fn fetch_all_results(username: &str, host: &str, token: &str, use_pagination: bool) -> Response {
    let base = fetch_result(username, host, token, None);

    if !use_pagination {
        return base;
    }

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

    let res = fetch_all_results(cli.username(), cli.host(), cli.token(), cli.pagination());
    let dates = find_dates(&res, cli.days());

    if dates.is_empty() {
        print_warning(
            "Empty response. Is the username correct? Does the token has read permission?",
            0,
        );
    } else {
        for (i, day) in dates.iter().enumerate() {
            print_day(day, &res);
            let is_last = i == dates.len() - 1;
            if !is_last {
                println!();
            }
        }
    }
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

/// Returns a sorted list from oldest to newest date with data for the last
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

fn find_total_time_per_day(date: &NaiveDate, res: &Response) -> Duration {
    find_logs_of_day(date, res).map(|node| node.timeSpent).sum()
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
    print!("  ");
    print_duration(log.timeSpent);
    println!(
        "  [{}]: {}",
        Style::new().dimmed().paint(
            log.issue
                .epic
                .as_ref()
                .map(|e| e.title.as_str())
                .unwrap_or("<no epic>")
        ),
        Style::new().bold().paint(log.issue.title.clone()),
    );

    let min_minutes_threshold = 15;
    if log.timeSpent.as_secs() / 60 < min_minutes_threshold {
        // msg is aligned with the suspicious data output
        print_warning("^ WARN: Less than 15 minutes! Is this correct?", 6);
    }

    for line in log.summary.lines() {
        println!("             {line}");
    }
}

fn print_warning(msg: &str, indention: usize) {
    println!(
        "{indention}{msg}",
        indention = " ".repeat(indention),
        msg = Style::new().bold().fg(Color::Yellow).paint(msg),
    );
}

fn print_day(day: &NaiveDate, data: &Response) {
    let total = find_total_time_per_day(day, data);

    let day_print = format!("{day}, {}", day.weekday());

    print!("{}  (", Style::new().bold().paint(day_print));
    print_duration(total);
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

const fn duration_to_hhmm(dur: Duration) -> (u64, u64) {
    let hours = dur.as_secs() / 60 / 60;
    let remaining_secs = dur.as_secs() - (hours * 60 * 60);
    let minutes = remaining_secs / 60;
    (hours, minutes)
}

fn print_duration(duration: Duration) {
    let (hours, minutes) = duration_to_hhmm(duration);
    let print_str = format!("{hours:>2}h {minutes:02}m");
    print!("{}", Style::new().bold().fg(Color::Blue).paint(print_str));
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
