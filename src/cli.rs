use chrono::{Local, NaiveDate};
use clap::Parser;
use std::str::FromStr;

/// CLI Arguments for `clap`. If not present, the values are taken from
/// environment variables.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tool to fetch the timelogs from the GitLab API and display them in a helpful way."
)]
pub struct CliArgs {
    /// The GitLab host without `https://`. For example `gitlab.domain.tld`.
    #[arg(long = "host", env)]
    gitlab_host: String,
    /// Your GitLab username.
    #[arg(long = "username", env)]
    gitlab_username: String,
    /// Token with read access to GitLab API. You can get one on
    /// `https://<gitlab_host>-/user_settings/personal_access_tokens`.
    #[arg(long = "token", env)]
    gitlab_token: String,
    /// Filter for newest inclusive date. For example `2024-06-01`.
    ///
    /// Must be no more than `--after`.
    #[arg(long = "before")]
    gitlab_before: Option<NaiveDate>,
    /// Filter for oldest inclusive date. For example `2024-06-01`.
    ///
    /// Must be no less than `--before`.
    #[arg(long = "after")]
    gitlab_after: Option<NaiveDate>,
}

impl CliArgs {
    pub fn host(&self) -> &str {
        &self.gitlab_host
    }
    pub fn username(&self) -> &str {
        &self.gitlab_username
    }
    pub fn token(&self) -> &str {
        &self.gitlab_token
    }
    pub fn before(&self) -> NaiveDate {
        self.gitlab_before
            .unwrap_or_else(|| Local::now().naive_local().date())
    }
    pub fn after(&self) -> NaiveDate {
        self.gitlab_after
            .unwrap_or_else(|| NaiveDate::from_str("1970-01-01").unwrap())
    }
}
