use clap::Parser;
use std::num::NonZeroUsize;

/// CLI Arguments for `clap`. If not present, the values are taken from
/// environment variables.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tool to fetch the timelogs from the GitLab API and display them in a helpful way."
)]
pub struct CliArgs {
    /// The GitLab host without `https://`. For example `gitlab.domain.tld`.
    ///
    /// You can also use the env variable `GITLAB_HOST`.
    #[arg(long = "host", env)]
    gitlab_host: String,
    /// Your GitLab username.
    ///
    /// You can also use the env variable `GITLAB_USERNAME`.
    #[arg(long = "username", env)]
    gitlab_username: String,
    /// Token with read access to GitLab API.
    ///
    /// You can also use the env variable `GITLAB_TOKEN`.
    #[arg(long = "token", env)]
    gitlab_token: String,
    /// How many days (starting with today = 1).
    ///
    /// You can also use the env variable `GITLAB_DAYS`.
    #[arg(long = "days", env, default_value = "1")]
    gitlab_days: NonZeroUsize,
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
    pub fn days(&self) -> usize {
        self.gitlab_days.into()
    }
}
