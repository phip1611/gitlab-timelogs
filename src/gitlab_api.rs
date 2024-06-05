#[allow(non_snake_case)]
pub mod types {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Issue {
        pub title: String,
        pub epic: Epic,
    }

    #[derive(Deserialize, Debug)]
    pub struct ResponseEntry {
        pub spentAt: String,
        pub timeSpent: u64,
        pub summary: String,
        pub issue: Issue,
        pub project: Project,
    }

    #[derive(Deserialize, Debug)]
    pub struct ResponseTimelogs {
        pub nodes: Vec<ResponseEntry>,
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
