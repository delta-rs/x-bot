use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use octocrab::{models::repos::Release, Octocrab};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use twitter::prelude::*;
use std::collections::HashMap;
use log::{info, warn};
use env_logger;

#[derive(Deserialize)]
struct Commit {
    sha: String,
    commit: CommitDetails,
    html_url: String,
    author: Option<Author>,
}

#[derive(Deserialize)]
struct CommitDetails {
    message: String,
    author: CommitAuthor,
}

#[derive(Deserialize)]
struct CommitAuthor {
    name: String,
    date: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Hash)]
struct Author {
    login: String,
}

#[derive(Deserialize, Serialize)]
struct ContributorData {
    contributors: HashSet<String>,
}

#[derive(Deserialize, Serialize)]
struct ReleaseData {
    latest_release_tag: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct IssueData {
    issues: HashSet<String>,
}

#[derive(Deserialize, Serialize)]
struct PullRequestData {
    pull_requests: HashSet<String>,
}

#[derive(Deserialize, Serialize)]
struct CommentData {
    comments: HashSet<String>,
}

#[derive(Deserialize, Serialize)]
struct StarData {
    stargazers: HashSet<String>,
}

#[derive(Debug)]
struct BotState {
    last_checked_contributors: Option<DateTime<Utc>>,
    last_checked_releases: Option<DateTime<Utc>>,
    last_checked_issues: Option<DateTime<Utc>>,
    last_checked_pull_requests: Option<DateTime<Utc>>,
    last_checked_comments: Option<DateTime<Utc>>,
    last_checked_stars: Option<DateTime<Utc>>,
}

async fn check_new_contributors(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    contributor_file: &PathBuf,
    state: &Mutex<BotState>,
    client: &TwitterClient,
) -> Result<()> {
    let mut state = state.lock().await;

    if state.last_checked_contributors.is_none()
        || Utc::now() - *state.last_checked_contributors > Duration::from_secs(3600)
    {
        info!("Checking for new contributors...");
        let commits: Vec<Commit> = octocrab
            .repos(owner, repo)
            .list_commits()
            .branch("master")
            .per_page(10)
            .send()
            .await
            .context("Failed to fetch commits")?;

        let mut existing_contributors: HashSet<String> = match File::open(contributor_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                serde_json::from_str(&contents)?
            }
            Err(_) => ContributorData {
                contributors: HashSet::new(),
            },
        }
        .contributors;

        let mut new_contributors = HashSet::new();
        for commit in commits {
            if let Some(author) = &commit.author {
                if !existing_contributors.contains(&author.login) {
                    new_contributors.insert(author.login.clone());
                }
            }
        }

        if !new_contributors.is_empty() {
            for contributor in &new_contributors {
                let tweet_text = format!(
                    "Delta community got a new contributor! Welcome to the community, @{contributor}! 🎉\n\nDetails: {}\nLink: {}",
                    commit.commit.message, commit.html_url
                );

                post_tweet(client, &tweet_text).await?;

                if let Ok(user) = client.users().show_by_username(contributor).await {
                    let welcome_message = format!(
                        "Hi @{contributor}, welcome to the Delta community! We're thrilled to have you contribute. 🎉"
                    );
                    send_direct_message(client, user.data.id.clone(), &welcome_message).await?;
                }
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(contributor_file)
            .context("Failed to open contributor file")?;
        let updated_data = ContributorData {
            contributors: existing_contributors.union(&new_contributors).cloned().collect(),
        };
        serde_json::to_writer_pretty(&mut file, &updated_data)?;

        state.last_checked_contributors = Some(Utc::now());
    }

    Ok(())
}

async fn check_new_releases(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    release_file: &PathBuf,
    state: &Mutex<BotState>,
    client: &TwitterClient,
) -> Result<()> {
    let mut state = state.lock().await;

    if state.last_checked_releases.is_none()
        || Utc::now() - *state.last_checked_releases > Duration::from_secs(3600)
    {
        info!("Checking for new releases...");
        let releases: Vec<Release> = octocrab
            .repos(owner, repo)
            .list_releases()
            .per_page(5)
            .send()
            .await
            .context("Failed to fetch releases")?;

        let mut existing_release_data: ReleaseData = match File::open(release_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                serde_json::from_str(&contents)?
            }
            Err(_) => ReleaseData {
                latest_release_tag: None,
            },
        };

        let mut new_release_tag = None;
        for release in releases {
            new_release_tag = Some(release.tag_name.clone());
            break;
        }

        if new_release_tag != existing_release_data.latest_release_tag {
            if let Some(tag_name) = &new_release_tag {
                let tweet_text = format!(
                    "New release ({}) of Delta is out! 🎉\n\nLink to release notes: {}",
                    tag_name,
                    new_release_tag.as_ref().unwrap_or_default()
                );

                post_tweet(client, &tweet_text).await?;
            }

            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(release_file)
                .context("Failed to open release file")?;
            let updated_data = ReleaseData {
                latest_release_tag: new_release_tag.clone(),
            };
            serde_json::to_writer_pretty(&mut file, &updated_data)?;
        }

        state.last_checked_releases = Some(Utc::now());
    }

    Ok(())
}

async fn check_new_issues(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    issue_file: &PathBuf,
    state: &Mutex<BotState>,
    client: &TwitterClient,
) -> Result<()> {
    let mut state = state.lock().await;

    if state.last_checked_issues.is_none()
        || Utc::now() - *state.last_checked_issues > Duration::from_secs(3600)
    {
        info!("Checking for new issues...");
        let issues: Vec<octocrab::models::Issue> = octocrab
            .issues(owner, repo)
            .list()
            .state(octocrab::params::State::Open)
            .per_page(10)
            .send()
            .await
            .context("Failed to fetch issues")?;

        let mut existing_issues: HashSet<String> = match File::open(issue_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                serde_json::from_str(&contents)?
            }
            Err(_) => IssueData {
                issues: HashSet::new(),
            },
        }
        .issues;

        let mut new_issues = HashSet::new();
        for issue in issues {
            if !existing_issues.contains(&issue.title) {
                new_issues.insert(issue.title.clone());
            }
        }

        if !new_issues.is_empty() {
            for issue in &new_issues {
                let tweet_text = format!(
                    "New issue opened in Delta: {}\n\nLink: {}",
                    issue, issue.html_url
                );

                post_tweet(client, &tweet_text).await?;
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(issue_file)
            .context("Failed to open issue file")?;
        let updated_data = IssueData {
            issues: existing_issues.union(&new_issues).cloned().collect(),
        };
        serde_json::to_writer_pretty(&mut file, &updated_data)?;

        state.last_checked_issues = Some(Utc::now());
    }

    Ok(())
}

async fn check_new_pull_requests(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    pr_file: &PathBuf,
    state: &Mutex<BotState>,
    client: &TwitterClient,
) -> Result<()> {
    let mut state = state.lock().await;

    if state.last_checked_pull_requests.is_none()
        || Utc::now() - *state.last_checked_pull_requests > Duration::from_secs(3600)
    {
        info!("Checking for new pull requests...");
        let pull_requests: Vec<octocrab::models::PullRequest> = octocrab
            .pulls(owner, repo)
            .list()
            .state(octocrab::params::State::Open)
            .per_page(10)
            .send()
            .await
            .context("Failed to fetch pull requests")?;

        let mut existing_pull_requests: HashSet<String> = match File::open(pr_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                serde_json::from_str(&contents)?
            }
            Err(_) => PullRequestData {
                pull_requests: HashSet::new(),
            },
        }
        .pull_requests;

        let mut new_pull_requests = HashSet::new();
        for pr in pull_requests {
            if !existing_pull_requests.contains(&pr.title) {
                new_pull_requests.insert(pr.title.clone());
            }
        }

        if !new_pull_requests.is_empty() {
            for pr in &new_pull_requests {
                let tweet_text = format!(
                    "New pull request opened in Delta: {}\n\nLink: {}",
                    pr, pr.html_url
                );

                post_tweet(client, &tweet_text).await?;
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(pr_file)
            .context("Failed to open pull request file")?;
        let updated_data = PullRequestData {
            pull_requests: existing_pull_requests.union(&new_pull_requests).cloned().collect(),
        };
        serde_json::to_writer_pretty(&mut file, &updated_data)?;

        state.last_checked_pull_requests = Some(Utc::now());
    }

    Ok(())
}

async fn check_new_comments(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    comment_file: &PathBuf,
    state: &Mutex<BotState>,
    client: &TwitterClient,
) -> Result<()> {
    let mut state = state.lock().await;

    if state.last_checked_comments.is_none()
        || Utc::now() - *state.last_checked_comments > Duration::from_secs(3600)
    {
        info!("Checking for new comments...");
        let issues: Vec<octocrab::models::Issue> = octocrab
            .issues(owner, repo)
            .list()
            .state(octocrab::params::State::Open)
            .per_page(10)
            .send()
            .await
            .context("Failed to fetch issues")?;

        let mut existing_comments: HashSet<String> = match File::open(comment_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                serde_json::from_str(&contents)?
            }
            Err(_) => CommentData {
                comments: HashSet::new(),
            },
        }
        .comments;

        let mut new_comments = HashSet::new();
        for issue in issues {
            let comments: Vec<octocrab::models::IssueComment> = octocrab
                .issues(owner, repo)
                .issue_number(issue.number)
                .list_comments()
                .per_page(10)
                .send()
                .await
                .context("Failed to fetch comments")?;

            for comment in comments {
                if !existing_comments.contains(&comment.body) {
                    new_comments.insert(comment.body.clone());
                }
            }
        }

        if !new_comments.is_empty() {
            for comment in &new_comments {
                let tweet_text = format!(
                    "New comment on Delta issue: {}\n\nLink: {}",
                    comment, comment.html_url
                );

                post_tweet(client, &tweet_text).await?;
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(comment_file)
            .context("Failed to open comment file")?;
        let updated_data = CommentData {
            comments: existing_comments.union(&new_comments).cloned().collect(),
        };
        serde_json::to_writer_pretty(&mut file, &updated_data)?;

        state.last_checked_comments = Some(Utc::now());
    }

    Ok(())
}

async fn check_new_stars(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    star_file: &PathBuf,
    state: &Mutex<BotState>,
    client: &TwitterClient,
) -> Result<()> {
    let mut state = state.lock().await;

    if state.last_checked_stars.is_none()
        || Utc::now() - *state.last_checked_stars > Duration::from_secs(3600)
    {
        info!("Checking for new stars...");
        let stargazers: Vec<octocrab::models::User> = octocrab
            .repos(owner, repo)
            .list_stargazers()
            .per_page(10)
            .send()
            .await
            .context("Failed to fetch stargazers")?;

        let mut existing_stargazers: HashSet<String> = match File::open(star_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                serde_json::from_str(&contents)?
            }
            Err(_) => StarData {
                stargazers: HashSet::new(),
            },
        }
        .stargazers;

        let mut new_stargazers = HashSet::new();
        for stargazer in stargazers {
            if !existing_stargazers.contains(&stargazer.login) {
                new_stargazers.insert(stargazer.login.clone());
            }
        }

        if !new_stargazers.is_empty() {
            for stargazer in &new_stargazers {
                let tweet_text = format!(
                    "Delta got a new star from @{stargazer}! ⭐\n\nLink: {}",
                    stargazer, stargazer.html_url
                );

                post_tweet(client, &tweet_text).await?;
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(star_file)
            .context("Failed to open star file")?;
        let updated_data = StarData {
            stargazers: existing_stargazers.union(&new_stargazers).cloned().collect(),
        };
        serde_json::to_writer_pretty(&mut file, &updated_data)?;

        state.last_checked_stars = Some(Utc::now());
    }

    Ok(())
}

async fn post_tweet(client: &TwitterClient, tweet_text: &str) -> Result<()> {
    let tweet_request = TweetRequest::new(tweet_text);
    let tweet_result = client.tweets().create(tweet_request).await?;

    if let Ok(tweet) = tweet_result {
        info!("Tweet posted successfully: {}", tweet.data.id);
    }

    Ok(())
}

async fn send_direct_message(client: &TwitterClient, user_id: &str, message: &str) -> Result<()> {
    let dm_request = DirectMessageRequest::new(user_id, message);
    let dm_result = client.direct_messages().create(dm_request).await?;

    if let Ok(dm) = dm_result {
        info!("Direct message sent successfully: {:?}", dm);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let github_token = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN environment variable not set")?;
    let twitter_consumer_key =
        env::var("TWITTER_CONSUMER_KEY").context("TWITTER_CONSUMER_KEY environment variable not set")?;
    let twitter_consumer_secret =
        env::var("TWITTER_CONSUMER_SECRET").context("TWITTER_CONSUMER_SECRET environment variable not set")?;
    let twitter_access_token =
        env::var("TWITTER_ACCESS_TOKEN").context("TWITTER_ACCESS_TOKEN environment variable not set")?;
    let twitter_access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET")
        .context("TWITTER_ACCESS_TOKEN_SECRET environment variable not set")?;

    let octocrab = Octocrab::builder().personal_token(github_token).build()?;
    let twitter_client = TwitterClient::new(
        twitter_consumer_key,
        twitter_consumer_secret,
        twitter_access_token,
        twitter_access_token_secret,
    );

    let contributor_file = PathBuf::from("contributors.json");
    let release_file = PathBuf::from("releases.json");
    let issue_file = PathBuf::from("issues.json");
    let pr_file = PathBuf::from("pull_requests.json");
    let comment_file = PathBuf::from("comments.json");
    let star_file = PathBuf::from("stars.json");
    let state = Mutex::new(BotState {
        last_checked_contributors: None,
        last_checked_releases: None,
        last_checked_issues: None,
        last_checked_pull_requests: None,
        last_checked_comments: None,
        last_checked_stars: None,
    });

    loop {
        if let Err(e) = check_new_contributors(
            &octocrab,
            "owner",
            "repo",
            &contributor_file,
            &state,
            &twitter_client,
        )
        .await
        {
            warn!("Error checking new contributors: {:?}", e);
        }

        if let Err(e) = check_new_releases(
            &octocrab,
            "owner",
            "repo",
            &release_file,
            &state,
            &twitter_client,
        )
        .await
        {
            warn!("Error checking new releases: {:?}", e);
        }

        if let Err(e) = check_new_issues(
            &octocrab,
            "owner",
            "repo",
            &issue_file,
            &state,
            &twitter_client,
        )
        .await
        {
            warn!("Error checking new issues: {:?}", e);
        }

        if let Err(e) = check_new_pull_requests(
            &octocrab,
            "owner",
            "repo",
            &pr_file,
            &state,
            &twitter_client,
        )
        .await
        {
            warn!("Error checking new pull requests: {:?}", e);
        }

        if let Err(e) = check_new_comments(
            &octocrab,
            "owner",
            "repo",
            &comment_file,
            &state,
            &twitter_client,
        )
        .await
        {
            warn!("Error checking new comments: {:?}", e);
        }

        if let Err(e) = check_new_stars(
            &octocrab,
            "owner",
            "repo",
            &star_file,
            &state,
            &twitter_client,
        )
        .await
        {
            warn!("Error checking new stars: {:?}", e);
        }

        sleep(Duration::from_secs(3600)).await;
    }
}
