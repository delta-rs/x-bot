use crate::{
    github::{
        client::GitHubClient, 
        types::{
            PingEvent, 
            PushEvent, 
            ReleaseEvent}},
    x::client::XClient};
use std::sync::Arc;
use axum::{
    http::{StatusCode, HeaderMap},
    extract::State};
use anyhow::Result;
use tracing::{debug, error, info, warn};

/// A handler for incoming webhook events from GitHub.
pub struct WebhookHandler {
    github_client: GitHubClient,
    x_client: Arc<XClient>,
}

impl WebhookHandler {
    /// Creates a new instance of [WebhookHandler](WebhookHandler).
    ///
    /// # Arguments
    /// * `github_client` - An instance of `GitHubClient` for interacting with the GitHub API.
    /// * `x_client` - An Arc wrapped instance of [XClient](XClient) for thread-safe posting to Twitter.
    ///
    /// # Returns
    /// An instance of [WebhookHandler](WebhookHandler).
    pub fn new(github_client: GitHubClient, x_client: Arc<XClient>) -> Self {
        Self {
            github_client,
            x_client,
        }
    }

    /// Handles push events from GitHub.
    ///
    /// # Arguments
    /// * `event` - A `PushEvent` containing the details of the push event.
    ///
    /// # Returns
    /// A result indicating success or failure.
    /// Key Features
    /// Event Filtering:
    /// The method only processes pushes to the master or main branches. If the push is to a different branch, it returns early with Ok(()).
    /// Iterating Over Commits:
    /// It iterates through the commits in the push event, checking each commit for the author's username.
    /// First Contribution Check:
    /// For each commit, it checks if the author is making their first contribution using self.github_client.is_first_contribution(&username).await?.
    /// Tweet Formatting:
    /// Constructs a tweet message that includes the contributor's username, commit message, and a link to the commit.
    /// Posting to X (Twitter):
    /// Uses the self.x_client.post_with_retry(&tweet).await? method to post the tweet to X.
    /// Logging:
    /// Logs the tweet message before posting it.
    pub async fn handle_push(&self, event: PushEvent) -> Result<()> {
        debug!("Handling push event for ref: {}", event.git_ref);
        
        // Only handle pushes to master/main branch
        if !event.git_ref.ends_with("/main") && !event.git_ref.ends_with("/master") {
            debug!("Ignoring push to non-main branch: {}", event.git_ref);
            return Ok(());
        }

        info!("Processing push to main branch with {} commits", event.commits.len());
        let repo_owner = &event.repository.owner.login;
        
        for commit in event.commits {
            if let Some(username) = &commit.author.username {
                // Skip if the committer is the repo owner
                if username == repo_owner {
                    debug!("Skipping commit from repository owner: {}", username);
                    continue;
                }

                debug!("Checking if {} is a first-time contributor", username);
                
                if self.github_client.is_first_contribution(username).await? {
                    info!("Found first-time contributor: {}", username);
                    
                    let tweet = format!(
                        "Delta got a new contributor {}!\nDetails: {}\nLink: {}",
                        username,
                        commit.message,
                        commit.url
                    );
                    
                    info!("Posting tweet about new contributor: {}", tweet);
                    match self.x_client.post_with_retry(&tweet).await {
                        Ok(_) => info!("Successfully posted tweet about new contributor {}", username),
                        Err(e) => error!("Failed to post tweet about new contributor: {:?}", e),
                    }
                } else {
                    debug!("Contributor {} has previous contributions", username);
                }
            } else {
                warn!("Commit {} has no associated username", commit.id);
            }
        }
        
        Ok(())
    }

    /// Handles release events from GitHub.
    ///
    /// # Arguments
    /// * `event` - A `ReleaseEvent` containing the details of the release event.
    ///
    /// # Returns
    /// A result indicating success or failure.
    /// Key Features
    /// Event Filtering:
    /// The method only processes releases that are marked as "published". If the action is not "published", it returns early with Ok(()).
    /// Tweet Formatting:
    /// Constructs a tweet message that includes the version tag and a link to the release notes.
    /// Posting to X (Twitter):
    /// Uses the self.x_client.send_tweet(&tweet).await? method to post the tweet to X.
    /// Logging:
    /// Logs the tweet message before posting it.

    pub async fn handle_release(&self, event: ReleaseEvent) -> Result<()> {
        // Only process published releases
        if event.action != "published" {
            return Ok(());
        }

        let repo_name = &event.repository.full_name;
        let version = &event.release.tag_name;
        // let release_name = event.release.name.unwrap_or_else(|| version.clone());
        
        let tweet = format!(
            "New release ({}) of Delta out! 🎉\nLink to release notes: {}",
            version,
            event.release.html_url
        );

        info!("Posting new release tweet for {}: {}", repo_name, tweet);
        if let Err(e) = self.x_client.send_tweet(&tweet).await {
            error!("Failed to post tweet for new release {}: {}", version, e);
        }

        Ok(())
    }

}

// App state that will be shared across requests
pub struct AppState {
    pub webhook_handler: WebhookHandler,
}


// Health check endpoint
pub async fn health_check() -> &'static str {
    info!("Health check debug message");
    "Health-Check-OK"
}

pub async fn call_back() -> &'static str {
    info!("Call_back debug message");
    "Callback-OK"
}


// Webhook handler that uses app state
pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    debug!("Received raw webhook body: {}", body);
    
    // Get the event type from headers
    let event_type = headers
        .get("x-github-event")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            error!("Missing x-github-event header");
            StatusCode::BAD_REQUEST
        })?;
    
    debug!("GitHub Event Type: {}", event_type);
    
    // Parse the body based on event type
    let result = match event_type {
        "ping" => {
            debug!("Handling ping event");
            let _ping_event: PingEvent = serde_json::from_str(&body).map_err(|e| {
                error!("Failed to parse ping event: {:?}", e);
                StatusCode::UNPROCESSABLE_ENTITY
            })?;
            info!("Received ping event - webhook is configured correctly");
            Ok(StatusCode::OK)
        },
        "push" => {
            debug!("Handling push event");
            let push_event: PushEvent = serde_json::from_str(&body).map_err(|e| {
                error!("Failed to parse push event: {:?}", e);
                StatusCode::UNPROCESSABLE_ENTITY
            })?;
            state.webhook_handler.handle_push(push_event).await.map_err(|e| {
                error!("Error handling push event: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            Ok(StatusCode::OK)
        },
        "release" => {
            debug!("Handling release event");
            let release_event: ReleaseEvent = serde_json::from_str(&body).map_err(|e| {
                error!("Failed to parse release event: {:?}", e);
                StatusCode::UNPROCESSABLE_ENTITY
            })?;
            state.webhook_handler.handle_release(release_event).await.map_err(|e| {
                error!("Error handling release event: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            Ok(StatusCode::OK)
        },
        _ => {
            error!("Unsupported event type: {}", event_type);
            Err(StatusCode::NOT_IMPLEMENTED)
        }
    };
    
    result
}