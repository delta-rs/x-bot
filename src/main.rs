mod error;
mod github;
mod x;

use crate::github::{GithubRepo, GithubTweetProducer};
use crate::x::XRepoAutoPoster;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short = 'o', long = "repo-owner")]
    github_repo_owner: String,
    #[arg(short = 'n', long = "repo-name")]
    github_repo_name: String,
    #[arg(short = 'b', long = "branch", default_value = "master")]
    branch: String,
    #[arg(short = 't', long = "bearer_token")]
    bearer_token: String,
    #[arg(short = 'i', long = "interval", default_value_t = 3600)]
    interval_secs: u64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let repo = GithubRepo::new(args.github_repo_owner, args.github_repo_name);
    let procuders = vec![
        GithubTweetProducer::NewContributorFirstCommit(repo.clone(), args.branch),
        GithubTweetProducer::NewRelease(repo),
    ];

    let poster = XRepoAutoPoster::new(args.bearer_token, procuders, args.interval_secs);
    poster.run().await;
}
