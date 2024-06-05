use clap::Parser;

/// CLI Arguments for `clap`. If not present, the values are taken from
/// environment variables.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tool to fetch the timelogs from the GitLab API and display them in a helpful way."
)]
pub struct CliArgs {
    /// The GitLab host without `https://`. For example `gitlab.domain.tld`
    #[arg(long = "host", env)]
    gitlab_host: String,
    /// Your GitLab username
    #[arg(long = "username", env)]
    gitlab_username: String,
    /// Token with read access to GitLab API
    #[arg(long = "token", env)]
    gitlab_token: String,
    /// Amount of entries to fetch (from the end, i.e. freshest data).
    #[arg(long = "last", env, default_value = "20")]
    gitlab_last: usize,
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
    pub fn last(&self) -> usize {
        self.gitlab_last
    }
}
