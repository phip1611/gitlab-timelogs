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
    #[arg(long = "host", env)]
    gitlab_host: String,
    /// Your GitLab username.
    #[arg(long = "username", env)]
    gitlab_username: String,
    /// Token with read access to GitLab API. You can get one on
    /// `https://<gitlab_host>-/user_settings/personal_access_tokens`.
    #[arg(long = "token", env)]
    gitlab_token: String,
    /// How many days (starting with today = 1) to display starting from the
    /// latest day with a record. This doesn't influence the request size.
    /// For large requests, please also specify `--pagination`.
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
