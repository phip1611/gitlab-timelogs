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

//! Typings for the GitLab API. These types are specific to the graphql query
//! used by this tool.

#[allow(non_snake_case)]
pub mod types {
    use chrono::{DateTime, Local, NaiveDate};
    use serde::Deserialize;
    use std::time::Duration;

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Epic {
        pub title: String,
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Issue {
        pub title: String,
        /// Full http link to issue.
        pub webUrl: String,
        pub epic: Option<Epic>,
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Group {
        pub fullName: String,
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Project {
        pub group: Option<Group>,
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ResponseNode {
        pub spentAt: String,
        /// For some totally weird reason, GitLab allows negative times.
        /// We recommend just deleting these records. But to support the
        /// deserialization, we have to do it like that.
        pub timeSpent: i64,
        pub summary: Option<String>,
        pub issue: Issue,
        pub project: Project,
    }

    impl ResponseNode {
        /// Returns a duration in seconds.
        pub const fn timeSpent(&self) -> (bool, Duration) {
            let dur = Duration::from_secs(self.timeSpent.unsigned_abs());
            (self.timeSpent.is_positive(), dur)
        }

        pub fn epic_name(&self) -> Option<&str> {
            self.issue.epic.as_ref().map(|e| e.title.as_str())
        }

        /// Parses the UTC timestring coming from GitLab in the local timezone of
        /// the user. This is necessary so that entries accounted to a Monday on
        /// `00:00` in CEST are not displayed as Sunday. The value is returned
        /// as [`NaiveDate`] but adjusted to the local time.
        pub fn datetime(&self) -> NaiveDate {
            let date = DateTime::parse_from_rfc3339(&self.spentAt).unwrap();
            let datetime = DateTime::<Local>::from(date);
            datetime.naive_local().date()
        }
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ResponsePageInfo {
        pub hasPreviousPage: bool,
        pub startCursor: Option<String>,
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ResponseTimelogs {
        pub nodes: Vec<ResponseNode>,
        pub pageInfo: ResponsePageInfo,
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ResponseData {
        pub timelogs: ResponseTimelogs,
    }

    /// The response from the GitLab API with all timelogs for the given
    /// time frame.
    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Response {
        pub data: ResponseData,
    }
}
