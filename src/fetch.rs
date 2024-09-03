/*
MIT License

Copyright (c) 2024 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Functionality to fetch data from the GitLab API.
//!
//! [`fetch_results`] is the entry point.

use crate::gitlab_api::types::Response;
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde_json::json;

const GRAPHQL_TEMPLATE: &str = include_str!("./gitlab-query.graphql");

/// Transforms a [`NaiveDate`] to a `DateTime<Local>`.
fn naive_date_to_local_datetime(date: NaiveDate) -> DateTime<Local> {
    date.and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .unwrap()
}

/// Performs a single request against the GitLab API, getting exactly one page
/// of the paged data source. The data is filtered for the date span to make the
/// request smaller/quicker.
///
/// # Parameters
/// - `username`: The exact GitLab username of the user.
/// - `host`: Host name of the GitLab instance without `https://`
/// - `token`: GitLab token to access the GitLab instance. Must have at least
///            READ access.
/// - `before`: Identifier from previous request to get the next page of the
///             paginated result.
/// - `start_date`: Inclusive begin date.
/// - `end_date`: Inclusive end date.
fn fetch_result(
    username: &str,
    host: &str,
    token: &str,
    before: Option<&str>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Response {
    let graphql_query = GRAPHQL_TEMPLATE
        .replace("%USERNAME%", username)
        .replace("%BEFORE%", before.unwrap_or_default())
        // GitLab API ignores the time component and just looks at the
        // date and the timezone.
        .replace(
            "%START_DATE%",
            naive_date_to_local_datetime(start_date)
                .to_string()
                .as_str(),
        )
        // GitLab API ignores the time component and just looks at the
        // date and the timezone.
        .replace(
            "%END_DATE%",
            naive_date_to_local_datetime(end_date).to_string().as_str(),
        );
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
///
/// # Parameters
/// - `username`: The exact GitLab username of the user.
/// - `host`: Host name of the GitLab instance without `https://`
/// - `token`: GitLab token to access the GitLab instance. Must have at least
///            READ access.
/// - `start_date`: Inclusive begin date.
/// - `end_date`: Inclusive end date.
pub fn fetch_results(
    username: &str,
    host: &str,
    token: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Response {
    let base = fetch_result(username, host, token, None, start_date, end_date);

    let mut aggregated = base;
    while aggregated.data.timelogs.pageInfo.hasPreviousPage {
        let mut next = fetch_result(
            username,
            host,
            token,
            Some(
                &aggregated
                    .data
                    .timelogs
                    .pageInfo
                    .startCursor
                    .expect("Should be valid string at this point"),
            ),
            start_date,
            end_date,
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
