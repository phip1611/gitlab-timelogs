use crate::gitlab_api::types::{Response, ResponseNode};
use chrono::{DateTime, Datelike, Local, Weekday, NaiveDate};
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

/// Safe estimation of time tracking entries per day.
const ENTRIES_PER_DAY_ESTIMATE: usize = 10;

fn main() {
    let cli = cli::CliArgs::parse();
    let graphql_query = GRAPHQL_TEMPLATE
        .replace("%USERNAME%", cli.username())
        .replace(
            "%LAST%",
            &format!("{}", cli.days() * ENTRIES_PER_DAY_ESTIMATE),
        );
    let payload = json!({ "query": graphql_query });

    let authorization = format!("Bearer {token}", token = cli.token());
    let url = format!("https://{host}/api/graphql", host = cli.host());
    let client = Client::new();

    let res = client
        .post(url)
        .header(AUTHORIZATION, authorization)
        .json(&payload)
        .send()
        .unwrap()
        .json::<Response>()
        .unwrap();

    for day in find_dates(&res, cli.days()) {
        print_day(&day, &res);
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
    find_logs_of_day(date, res)
        .map(|node| node.timeSpent)
        .sum()
}

fn find_logs_of_day<'a>(date: &'a NaiveDate, res: &'a Response) -> impl Iterator<Item = &'a ResponseNode> {
    res.data
        .timelogs
        .nodes
        .iter()
        .filter(|node| {
            let node_date = parse_gitlab_datetime(&node.spentAt);
            node_date == *date
        })
}

fn print_timelog(log: &ResponseNode) {
    print!("  ");
    print_duration(log.timeSpent);
    println!(
        "  [{}]: {}",
        Style::new().dimmed().paint(log.issue.epic.title.clone()),
        Style::new().bold().paint(log.issue.title.clone()),
    );
    for line in log.summary.lines() {
        println!("             {line}");
    }
}

fn print_day(day: &NaiveDate, data: &Response) {
    let total = find_total_time_per_day(&day, data);

    let day_print = format!("{day}, {}", day.weekday());

    print!("{}  (total: ", Style::new().bold().paint(day_print));
    print_duration(total);
    println!(")");

    // Sanity checks and print warnings
    {
        let hours_threshold = 10;
        if total.as_secs() > hours_threshold * 60 * 60 {
            println!(
                "  {}",
                Style::new()
                    .bold()
                    .fg(Color::Yellow)
                    .paint("WARN: More than 10 hours per work day! Is this correct?")
            );
        }

        match day.weekday() {
            Weekday::Sat | Weekday::Sun => {
                println!(
                    "  {}",
                    Style::new()
                        .bold()
                        .fg(Color::Yellow)
                        .paint("WARN: You shouldn't work on the weekend, right?")
                );
            }
            _ => {}
        }
    }

    for log in find_logs_of_day(&day, data) {
        print_timelog(log);
    }
    println!();
}

fn duration_to_hhmm(dur: Duration) -> (u64, u64) {
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
