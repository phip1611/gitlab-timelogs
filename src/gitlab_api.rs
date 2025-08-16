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
    use fmt::{Debug, Display};
    use serde::Deserialize;
    use std::error::Error;
    use std::fmt;
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

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct GraphQLErrorLocation {
        line: u64,
        column: u64,
    }

    impl Display for GraphQLErrorLocation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "line {}, column {}", self.line, self.column)
        }
    }

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct GraphQLErrorResponse {
        pub message: String,
        pub locations: Vec<GraphQLErrorLocation>,
    }

    impl Display for GraphQLErrorResponse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)?;
            if !self.locations.is_empty() {
                let locs: Vec<String> = self.locations.iter().map(|loc| loc.to_string()).collect();
                write!(f, " (at {})", locs.join(", "))?;
            }
            Ok(())
        }
    }

    impl Error for GraphQLErrorResponse {}

    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct GraphQLErrorsResponse(pub Vec<GraphQLErrorResponse>);

    impl Display for GraphQLErrorsResponse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.0.is_empty() {
                write!(f, "No GraphQL errors")
            } else if self.0.len() == 1 {
                write!(f, "GraphQL error: {}", self.0[0])
            } else {
                writeln!(f, "{} GraphQL errors:", self.0.len())?;
                for (i, err) in self.0.iter().enumerate() {
                    writeln!(f, "  {}. {}", i + 1, err)?;
                }
                Ok(())
            }
        }
    }

    impl Error for GraphQLErrorsResponse {}

    /// The serialized/typed GraphQL response from the GitLab API with all
    /// timelogs for the given time frame.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Response {
        PayloadResponse(ResponseData),
        ErrorResponse(GraphQLErrorsResponse),
    }

    impl Response {
        /// Transforms the GraphQL response to a Rust [`Result`].
        pub fn to_result(self) -> Result<ResponseData, GraphQLErrorsResponse> {
            match self {
                Response::PayloadResponse(payload) => Ok(payload),
                Response::ErrorResponse(errors) => Err(errors),
            }
        }
    }

    /// The serialized/typed GraphQL response from the GitLab API with all
    /// timelogs for the given time frame.
    #[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ResponseSerialized {
        pub data: Option<ResponseData>,
        pub errors: Option<GraphQLErrorsResponse>,
    }

    impl ResponseSerialized {
        /// Transforms the GraphQL response to a Rust [`Result`].
        pub fn to_typed(self) -> Response {
            match self {
                ResponseSerialized {
                    data: Some(data),
                    errors: None,
                } => Response::PayloadResponse(data),
                ResponseSerialized {
                    data: None,
                    errors: Some(errors),
                } => Response::ErrorResponse(errors),
                _ => panic!(
                    "Unexpected response: data={:#?}, errors={:#?}",
                    self.data, self.errors
                ),
            }
        }
    }
}
