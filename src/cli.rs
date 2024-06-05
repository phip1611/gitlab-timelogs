use clap::Parser;

/// CLI Arguments for `clap`. If not present, the values are taken from
/// environment variables.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tool to fetch the timelogs from the GitLab API and display them in a helpful way."
)]
pub struct CliArgs {
    /// The GitLab host. For example `https://gitlab.domain.tld`
    #[arg(long = "host", env)]
    gitlab_host: String,
    /// ID of the group for that you want to get the data. For example
    /// the ID of the group `%my_company%`.
    #[arg(long = "group-id", env)]
    gitlab_group_id: u64,
    /// Your GitLab username
    #[arg(long = "username", env)]
    gitlab_username: String,
    /// Token with read access to GitLab API
    #[arg(long = "token", env)]
    gitlab_token: String
}
