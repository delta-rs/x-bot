use crate::error::Error;
use chrono::{DateTime, Utc};
use octocrab::models::repos::RepoCommitPage;
use std::collections::HashMap;

#[derive(Clone)]
pub struct GithubRepo {
    owner: String,
    name: String,
}

impl GithubRepo {
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }
}

pub enum GithubTweetProducer {
    NewContributorFirstCommit(GithubRepo, String),
    NewRelease(GithubRepo),
}

impl GithubTweetProducer {
    pub async fn try_produce(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<String>, Error> {
        let octocrab = octocrab::instance();

        match self {
            Self::NewContributorFirstCommit(repo, branch) => {
                let mut author_commits: HashMap<String, RepoCommitPage> = HashMap::new();
                let mut page: u32 = 1;

                loop {
                    let repo_commits_after_start = match octocrab
                        .repos(&repo.owner, &repo.name)
                        .list_commits()
                        .branch(branch)
                        .since(start)
                        .page(page)
                        .per_page(100)
                        .send()
                        .await
                    {
                        Ok(p) => p.items,
                        Err(err) => match octocrab_err_is_ok(&err) {
                            true => Vec::new(),
                            false => return Err(Error::GitHubError(err.to_string())),
                        },
                    };

                    if repo_commits_after_start.len() == 0 {
                        break;
                    }

                    for rc in repo_commits_after_start {
                        if let Some(author) = rc.author {
                            author_commits.insert(author.id.to_string(), rc.commit);
                        }
                    }

                    page += 1;
                }

                let mut tweets = Vec::new();

                for (author_id, rcp) in author_commits {
                    let repo_commits_before_start = match octocrab
                        .repos(&repo.owner, &repo.name)
                        .list_commits()
                        .branch(branch)
                        .author(&author_id)
                        .until(start)
                        .send()
                        .await
                    {
                        Ok(p) => p.items,
                        Err(err) => match octocrab_err_is_ok(&err) {
                            true => Vec::new(),
                            false => return Err(Error::GitHubError(err.to_string())),
                        },
                    };

                    if repo_commits_before_start.len() == 0 {
                        let tweet = format!(
                            "Delta got a new contributor {}!\n\nDetails: {}\n\nLink: {}",
                            author_id, rcp.message, rcp.url,
                        );
                        tweets.push(tweet);
                    }
                }

                Ok(tweets)
            }
            Self::NewRelease(repo) => {
                let release = match octocrab
                    .repos(&repo.owner, &repo.name)
                    .releases()
                    .get_latest()
                    .await
                {
                    Ok(r) => r,
                    Err(err) => match octocrab_err_is_ok(&err) {
                        true => return Ok(Vec::new()),
                        false => return Err(Error::GitHubError(err.to_string())),
                    },
                };

                let mut tweets = Vec::new();

                // Check if release occured between start and end times
                if let Some(publish_date) = release.published_at {
                    if start <= publish_date && publish_date <= end {
                        let tweet = format!(
                            "New release ({}) of Delta out! 🎉\n\nLink to release notes: {}",
                            release.tag_name, release.url,
                        );
                        tweets.push(tweet);
                    }
                }

                Ok(tweets)
            }
        }
    }
}

fn octocrab_err_is_ok(err: &octocrab::Error) -> bool {
    match err {
        // Allowing octocrab::Error::GitHub to proceed,
        // because this error is being produced when a list of
        // zero commits is returned from the octocrab query
        octocrab::Error::GitHub { .. } => true,
        _ => false,
    }
}
