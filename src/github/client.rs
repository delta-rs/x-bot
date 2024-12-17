use super::contributor::{ContributorManager, ContributorInfo};
use octocrab::Octocrab;
use anyhow::Result;
use tracing::info;

pub struct GitHubClient {
    client: Octocrab,
    repo_owner: String,
    repo_name: String,
    contributor_manager: ContributorManager,
}

impl GitHubClient {
    /// Creates a new instance of `GitHubClient` with the specified token and repository details.
    ///
    /// # Arguments
    /// * `token` - A string containing the personal access token for GitHub API authentication.
    /// * `repo_owner` - A string containing the owner of the repository.
    /// * `repo_name` - A string containing the name of the repository.
    ///
    /// # Returns
    /// A result containing the initialized `GitHubClient` or an error if initialization fails.
    pub async fn new(token: String, repo_owner: String, repo_name: String) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()?;

        let contributor_manager = ContributorManager::new(
            client.clone(),
            repo_owner.clone(),
            repo_name.clone(),
            300, // 5 minutes cache TTL
        );
        
        info!("Github Api Client initialized");

        Ok(Self {
            client,
            repo_owner,
            repo_name,
            contributor_manager,
        })
    }

    /// Checks if the specified user is making their first contribution to the repository.
    ///
    /// # Arguments
    /// * `username` - A string slice containing the username of the contributor.
    ///
    /// # Returns
    /// A result containing `true` if this is the user's first contribution, or `false` otherwise.
    pub async fn is_first_contribution(&self, username: &str) -> Result<bool> {
        self.contributor_manager.is_first_contribution(username).await
    }

    /// Gets detailed information about a contributor.
    ///
    /// # Arguments
    /// * `username` - A string slice containing the username of the contributor.
    ///
    /// # Returns
    /// A result containing an optional `ContributorInfo` if the contributor exists.
    pub async fn get_contributor_info(&self, username: &str) -> Result<Option<ContributorInfo>> {
        self.contributor_manager.get_contributor_info(username).await
    }
}
