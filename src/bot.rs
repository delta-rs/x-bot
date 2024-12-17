use std::collections::HashSet;

use octocrab::models::events::payload::{EventPayload, ReleaseEventAction, WrappedEventPayload};
use octocrab::models::repos::CommitAuthor;

use super::{github, x, Announcement, Configuration, NewContributor, NewRelease};

pub async fn start(config: Configuration) -> anyhow::Result<()> {
    let github_client = github::Client::new(config.github, config.auth.clone())?;
    let x_client = x::Client::new(config.x, config.auth)?;

    let mut announced_contributors = github_client.gather_announced_contributors().await?;
    let mut receiver = github_client.start_poll_events().await?;

    while let Some(event) = receiver.recv().await {
        if let Some(announcement) = filter_github_event(&mut announced_contributors, event).await {
            tweet_announcement(&x_client, announcement).await?;
        }
    }

    Ok(())
}

async fn filter_github_event(
    announced_contributors: &mut HashSet<String>,
    event: WrappedEventPayload,
) -> Option<Announcement> {
    match event.specific {
        Some(EventPayload::PushEvent(payload)) => Some(Announcement::Contributor(
            payload
                .commits
                .into_iter()
                .filter(|commit| commit.distinct)
                .filter(|commit| is_first_contribution(announced_contributors, &commit.author))
                .map(|commit| NewContributor {
                    name: commit.author.name,
                    commit_message: commit.message,
                    url: commit.url.to_string(),
                })
                .collect(),
        )),
        Some(EventPayload::ReleaseEvent(payload))
            if payload.action == ReleaseEventAction::Published =>
        {
            Some(Announcement::Release(NewRelease {
                version: payload.release.name.unwrap_or(payload.release.tag_name),
                url: payload.release.html_url.to_string(),
            }))
        }
        _ => None,
    }
}

async fn tweet_announcement(
    x_client: &x::Client,
    announcement: Announcement,
) -> anyhow::Result<()> {
    match announcement {
        Announcement::Contributor(new_contributors) => {
            for new_contributor in new_contributors.into_iter() {
                x_client.post_tweet(new_contributor.into()).await
            }
        }
        Announcement::Release(new_release) => x_client.post_tweet(new_release.into()).await,
    }

    Ok(())
}

pub fn is_first_contribution(
    announced_contributors: &mut HashSet<String>,
    author: &CommitAuthor,
) -> bool {
    if announced_contributors.insert(author.email.clone()) {
        log::info!(
            "Number of announced contributors: {:?}",
            announced_contributors.len()
        );
        true
    } else {
        false
    }
}
