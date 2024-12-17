use indoc::formatdoc;
use oauth1::ParameterList;
use reqwest::{Response, Url};
use serde::Serialize;
use std::{borrow::Cow, time::Duration};
use tokio::time::sleep;

use super::{AuthConfiguration, NewContributor, NewRelease, XConfiguration};

#[derive(Debug, Serialize, oauth1::Request)]
pub struct PostTweetRequest {
    text: String,
}

impl PostTweetRequest {
    pub fn announce_commiter(name: String, commit_message: String, commit_link: String) -> Self {
        let text = formatdoc! {"
            Delta got a new contributor {name}!

            Details: {commit_message} 

            Link: {commit_link} 
        "};

        Self { text }
    }

    pub fn announce_release(version: String, release_notes_link: String) -> Self {
        let text = formatdoc! {"
            New release ({version}) of Delta out! 🎉
  
            Link to release notes: {release_notes_link}
        "};

        Self { text }
    }
}

impl From<NewContributor> for PostTweetRequest {
    fn from(new_committer: NewContributor) -> Self {
        Self::announce_commiter(
            new_committer.name,
            new_committer.commit_message,
            new_committer.url,
        )
    }
}

impl From<NewRelease> for PostTweetRequest {
    fn from(new_release: NewRelease) -> Self {
        Self::announce_release(new_release.version, new_release.url)
    }
}

pub struct Client {
    oauth_token: oauth1::Token,
    http_client: reqwest::Client,
    base_url: Url,
    max_attempts: u64,
}

impl Client {
    pub fn new(config: XConfiguration, auth: AuthConfiguration) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(config.http_client_max_idle)
            .timeout(config.http_client_timeout)
            .connection_verbose(true)
            .build()?;

        let base_url = config
            .base_url
            .map(|url| url.parse())
            .unwrap_or("https://api.x.com/".parse())
            .map(|url: Url| {
                let mut url = url.clone();
                url.set_path("/2/tweets");
                url
            })?;

        let oauth_token = oauth1::Token::from_parts(
            auth.x_client_identifier,
            auth.x_client_secret,
            auth.x_token,
            auth.x_token_secret,
        );

        Ok(Self {
            http_client,
            oauth_token,
            base_url,
            max_attempts: config.max_retries.map(|r| r + 1).unwrap_or(1),
        })
    }

    pub async fn post_tweet(&self, tweet: PostTweetRequest) {
        log::info!("Attempt to tweet: {:?}", tweet);
        for attempts in 1..=self.max_attempts {
            match self.send_post(&tweet).await {
                Ok(response) => {
                    log::debug!("Tweeted successfully. X's response: {:?}", response);
                    return;
                }
                Err(error) => {
                    log::error!("Retry tweet. (attempt {:?}) (error: {:?})", attempts, error);
                    if error
                        .status()
                        .filter(|status| status.is_client_error())
                        .is_some()
                    {
                        break;
                    } 
                }
            }

            sleep(Duration::from_secs(3 * attempts)).await
        }

        log::error!("Gave up trying to tweet");
    }

    async fn send_post(&self, tweet: &PostTweetRequest) -> Result<Response, reqwest::Error> {
        let authorization = self.authorize();
        self.http_client
            .post(self.base_url.clone())
            .header("authorization", authorization)
            .json(tweet)
            .send()
            .await
            .and_then(|res| res.error_for_status())
    }

    fn authorize(&self) -> String {
        let query_list: ParameterList<Cow<'_, _>, Cow<'_, _>> =
            ParameterList::from_iter(self.base_url.query_pairs());
        oauth1::authorize(
            "POST",
            &self.base_url,
            &query_list,
            &self.oauth_token,
            oauth1::HMAC_SHA1,
        )
    }
}
