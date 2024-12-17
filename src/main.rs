use serde::Deserialize;
use std::time::Duration;

mod bot;
mod github;
mod x;

#[derive(Deserialize, Debug, Clone)]
struct AuthConfiguration {
    github_bearer_token: Option<String>,
    x_client_identifier: String,
    x_client_secret: String,
    x_token: String,
    x_token_secret: String,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
struct XConfiguration {
    base_url: Option<String>,
    http_client_max_idle: usize,
    http_client_timeout: Duration,
    max_retries: Option<u64>,
}

impl Default for XConfiguration {
    fn default() -> Self {
        Self {
            base_url: None,
            http_client_max_idle: 1,
            http_client_timeout: Duration::from_secs(5),
            max_retries: Some(3),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
struct GithubConfiguration {
    base_url: Option<String>,
    branch: String,
    connect_timeout: Option<Duration>,
    exit_on_poll_error: bool,
    fetch_pages_per_request: u8,
    max_retries: Option<usize>,
    owner: String,
    poll_frequency: Duration,
    read_timeout: Option<Duration>,
    repository: String,
}

impl Default for GithubConfiguration {
    fn default() -> Self {
        Self {
            base_url: None,
            branch: "master".to_string(),
            connect_timeout: None,
            exit_on_poll_error: false,
            fetch_pages_per_request: 100,
            max_retries: Some(3),
            owner: "delta-rs".to_string(),
            poll_frequency: Duration::from_secs(300),
            read_timeout: None,
            repository: "delta".to_string(),
        }
    }
}

#[derive(Debug)]
struct Configuration {
    auth: AuthConfiguration,
    github: GithubConfiguration,
    x: XConfiguration,
}

impl Configuration {
    fn from_env() -> anyhow::Result<Self> {
        let x = envy::prefixed("X_").from_env()?;
        let github = envy::prefixed("GITHUB_").from_env()?;
        let auth = envy::from_env()?;

        log::info!("Running with config: {:#?} {:#?}", github, x);

        Ok(Self { github, x, auth })
    }
}

struct NewContributor {
    commit_message: String,
    name: String,
    url: String,
}

struct NewRelease {
    url: String,
    version: String,
}

enum Announcement {
    Contributor(Vec<NewContributor>),
    Release(NewRelease),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Configuration::from_env()?;
    bot::start(config).await?;

    Ok(())
}

#[cfg(test)]
mod test;
