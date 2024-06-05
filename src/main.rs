use clap::Parser;

mod cli;

const GRAPHQL_TEMPLATE: &str = include_str!("./gitlab-query.graphql");

fn main() {
    let cli = cli::CliArgs::parse();

}
