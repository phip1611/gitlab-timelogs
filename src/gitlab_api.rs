#[allow(non_snake_case)]
pub mod types {
    use super::*;
    use serde::Deserialize;
    use serde_with::serde_as;
    use std::time::Duration;
    use serde_with::DurationSeconds;

    #[derive(Deserialize, Debug)]
    pub struct Issue {
        pub title: String,
        pub epic: Epic,
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
    pub struct ResponseTimelogs {
        pub nodes: Vec<ResponseNode>,
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
        pub group: Group,
    }

    #[derive(Deserialize, Debug)]
    pub struct Group {
        pub name: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Epic {
        pub title: String,
    }
}
