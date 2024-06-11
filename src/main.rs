use crate::gitlab_api::types::{Response, ResponseNode};
use clap::Parser;
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::format;
use std::time::Duration;

mod cli;
mod gitlab_api;

const GRAPHQL_TEMPLATE: &str = include_str!("./gitlab-query.graphql");

fn main() {
    let cli = cli::CliArgs::parse();
    let graphql_query = GRAPHQL_TEMPLATE
        .replace("%USERNAME%", cli.username())
        .replace("%LAST%", &format!("{}", cli.last()));
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

    let all_dates = find_dates(&res);

    for date in &all_dates {
        let total_time = find_total_time_per_day(date, &res);
        print!("|- {date} ");
        // TODO support .5h
        let total_time_minutes = total_time.as_secs() / 60;
        if total_time_minutes >= 60 {
            print!("(total time: {:>2} hours)", total_time_minutes / 60);
        } else {
            print!("(total time: {:>2} minutes)", total_time_minutes);
        }
        println!();

        if total_time_minutes / 60 > 10 {
            println!("+++ WARNING +++");
        }

        let logs_of_day = find_logs_of_day(date, &res);
        for node in logs_of_day {
            print!(" \\- ");
            let minutes = node.timeSpent.as_secs() / 60;
            if minutes >= 60 {
                print!("{:>2}h", minutes / 60);
            } else {
                print!("{:2>}m", minutes);
            }
            print!(" {} [{}]", node.issue.title, node.issue.epic.title);
            println!();
            println!("  |   - {}", node.summary.trim().replace("\n", "<br>"));
        }
    }
}

fn find_dates(res: &Response) -> BTreeSet<&str> {
    res
        .data
        .timelogs
        .nodes
        .iter()
        .map(|node| &node.spentAt[0..10])
        .collect::<BTreeSet<_>>()
}

fn find_total_time_per_day(date: &str, res: &Response) -> Duration {
    res.data.timelogs.nodes.iter()
        .filter(|node| node.spentAt.starts_with(date))
        .map(|node| node.timeSpent)
        .sum()
}

fn find_logs_of_day<'a>(date: &'a str, res: &'a Response) -> Vec<&'a ResponseNode> {
    res.data.timelogs.nodes.iter()
        .filter(|node| node.spentAt.starts_with(date))
        .collect()
}
