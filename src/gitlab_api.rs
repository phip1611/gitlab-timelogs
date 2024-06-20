#[allow(non_snake_case)]
pub mod types {

    use serde::Deserialize;
    use std::time::Duration;

    #[derive(Deserialize, Debug)]
    pub struct Issue {
        pub title: String,
        /// Full http link to issue.
        pub webUrl: String,
        pub epic: Option<Epic>,
    }

    #[derive(Deserialize, Debug)]
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
    }

    #[derive(Deserialize, Debug)]
    pub struct ResponsePageInfo {
        pub hasPreviousPage: bool,
        pub startCursor: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct ResponseTimelogs {
        pub nodes: Vec<ResponseNode>,
        pub pageInfo: ResponsePageInfo,
    }

    #[derive(Deserialize, Debug)]
    pub struct ResponseData {
        pub timelogs: ResponseTimelogs,
    }

    #[derive(Deserialize, Debug)]
    pub struct Response {
        pub data: ResponseData,
    }

    #[derive(Deserialize, Debug)]
    pub struct Project {
        pub group: Option<Group>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Group {
        pub fullName: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Epic {
        pub title: String,
    }
}
