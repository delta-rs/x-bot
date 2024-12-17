use octocrab::etag::{EntityTag, Etagged};
use octocrab::models::events::Event;
use octocrab::models::repos::RepoCommit;
use octocrab::service::middleware::retry::RetryConfig;
use octocrab::Page;
use octocrab::{
    models::events::payload::WrappedEventPayload,
    Octocrab,
};
use std::collections::HashSet;
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};

use crate::{AuthConfiguration, GithubConfiguration};

pub struct Client {
    branch: String,
    exit_on_poll_error: bool,
    octocrab: Octocrab,
    owner: String,
    per_page: u8,
    poll_frequency: Duration,
    repository: String,
}

impl Client {
    pub fn new(config: GithubConfiguration, auth: AuthConfiguration) -> anyhow::Result<Self> {
        let mut client_builder = Octocrab::builder()
            .set_connect_timeout(config.connect_timeout)
            .set_read_timeout(config.read_timeout);

        if let Some(retries) = config.max_retries {
            client_builder = client_builder.add_retry_config(RetryConfig::Simple(retries))
        }

        if let Some(token) = auth.github_bearer_token {
            client_builder = client_builder.personal_token(token);
        }

        if let Some(url) = config.base_url {
            client_builder = client_builder.base_uri(url)?;
        }

        let octocrab = client_builder.build()?;

        Ok(Self {
            branch: config.branch,
            exit_on_poll_error: config.exit_on_poll_error,
            octocrab,
            owner: config.owner,
            per_page: config.fetch_pages_per_request,
            poll_frequency: config.poll_frequency,
            repository: config.repository,
        })
    }

    pub async fn gather_announced_contributors(&self) -> anyhow::Result<HashSet<String>> {
        let mut page_index: u32 = 0;
        let mut has_next_page = true;
        let mut announced_contributors = HashSet::new();
        while has_next_page {
            let page = self.fetch_commit_page(page_index).await?;

            has_next_page = page.next.is_some();
            page_index += self.per_page as u32;

            page.into_iter()
                .filter_map(|commit| commit.commit.author)
                .for_each(|author| {
                    announced_contributors.insert(author.email);
                });
        }

        log::debug!(
            "Number of gathered previous contributors: {:?}",
            announced_contributors.len()
        );

        Ok(announced_contributors)
    }

    pub async fn start_poll_events(self) -> anyhow::Result<mpsc::Receiver<WrappedEventPayload>> {
        let (sender, receiver) = mpsc::channel(1);
        let mut etag = None;
        tokio::spawn(async move {
            loop {
                log::debug!(
                    "Poll github events in {:?}/{:?} with etag {:?}",
                    self.owner,
                    self.repository,
                    etag
                );

                match self.fetch_event_page(etag.clone()).await {
                    Ok(events) => {
                        let payload_iter = etag
                            .and(events.value)
                            .into_iter()
                            .flat_map(|value| value.into_iter())
                            .filter_map(|event| event.payload);

                        for payload in payload_iter {
                            if sender.send(payload).await.is_err() {
                                break;
                            }
                        }
                        etag = events.etag;
                    }
                    Err(error) => {
                        log::error!(
                            "Failed to poll events from {:?}/{:?} due to {:?}",
                            self.owner,
                            self.repository,
                            error
                        );
                        if self.exit_on_poll_error {
                            break;
                        }
                    }
                }

                sleep(self.poll_frequency).await;
            }
        });

        Ok(receiver)
    }

    async fn fetch_commit_page(
        &self,
        page_index: u32,
    ) -> Result<Page<RepoCommit>, octocrab::Error> {
        log::debug!(
            "Fetch commits from {:?}/{:?} page {:?}",
            self.owner,
            self.repository,
            page_index
        );
        self.octocrab
            .repos(&self.owner, &self.repository)
            .list_commits()
            .branch(&self.branch)
            .per_page(self.per_page)
            .page(page_index)
            .send()
            .await
    }

    async fn fetch_event_page(
        &self,
        etag: Option<EntityTag>,
    ) -> Result<Etagged<Page<Event>>, octocrab::Error> {
        self.octocrab
            .repos(&self.owner, &self.repository)
            .events()
            .etag(etag)
            .per_page(self.per_page)
            .send()
            .await
    }
}
