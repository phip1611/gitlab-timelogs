#[allow(non_snake_case)]
pub mod types {

    use serde::Deserialize;
    use serde_with::serde_as;
    use serde_with::DurationSeconds;
    use std::time::Duration;

    #[derive(Deserialize, Debug)]
    pub struct Issue {
        pub title: String,
        /// Full http link to issue.
        pub webUrl: String,
        pub epic: Option<Epic>,
    }

    #[serde_as]
    #[derive(Deserialize, Debug)]
    pub struct ResponseNode {
        pub spentAt: String,
        #[serde_as(as = "DurationSeconds<u64>")]
        pub timeSpent: Duration,
        pub summary: String,
        pub issue: Issue,
        pub project: Project,
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
