use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    Push(PushEvent),
    Release(ReleaseEvent),
    Ping(PingEvent),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PingEvent {
    pub zen: String,
    pub hook_id: u64,
    pub hook: WebhookInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookInfo {
    pub url: String,
    pub test_url: String,
    pub ping_url: String,
    pub id: u64,
    pub active: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub commits: Vec<Commit>,
    pub repository: Repository,
    pub sender: GitHubUser,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub author: CommitAuthor,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub full_name: String,
    pub owner: GitHubUser,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReleaseEvent {
    pub action: String,
    pub release: Release,
    pub repository: Repository,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub html_url: String,
}
