#[allow(non_snake_case)]
pub mod types {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Timelog {
        pub spentAt: String,
        pub timeSpent: i32,
    }

    #[derive(Deserialize, Debug)]
    pub struct TimelogResponse {
        pub data: TimelogData,
    }

    #[derive(Deserialize, Debug)]
    pub struct TimelogData {
        pub timelogs: TimelogNodes,
    }

    #[derive(Deserialize, Debug)]
    pub struct TimelogNodes {
        pub nodes: Vec<Timelog>,
    }
}
