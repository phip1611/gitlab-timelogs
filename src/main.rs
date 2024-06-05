use crate::gitlab_api::types::Response;
use clap::Parser;
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde_json::json;
use std::fmt::format;

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

    dbg!(res);
}
