use std::{collections::HashMap,sync::Arc};
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::info;
use chrono::{DateTime, Utc};

/// Represents a contributor's information
#[derive(Debug, Clone)]
pub struct ContributorInfo {
    pub username: String,
    pub total_commits: usize,
    pub first_contribution_date: DateTime<Utc>,
    pub latest_contribution_date: DateTime<Utc>,
}

/// Manages contributor information with caching
pub struct ContributorManager {
    client: octocrab::Octocrab,
    repo_owner: String,
    repo_name: String,
    
    // Cache of contributor information
    // The HashMap structure is used here because:
    // Fast Lookups: We need O(1) lookups in is_first_contribution and get_contributor_info methods to quickly check contributor status
    // Unique Keys: Each GitHub username (the key) maps to exactly one ContributorInfo struct
    // Efficient Updates: During cache refresh, we can quickly update existing contributor information
    contributors_cache: Arc<RwLock<HashMap<String, ContributorInfo>>>,
    // Cache TTL in seconds
    cache_ttl: u64,
    // Last cache refresh timestamp
    last_refresh: Arc<RwLock<DateTime<Utc>>>,
}

impl ContributorManager {
    /// Creates a new ContributorManager
    pub fn new(
        client: octocrab::Octocrab,
        repo_owner: String,
        repo_name: String,
        cache_ttl: u64,
    ) -> Self {
        Self {
            client,
            repo_owner,
            repo_name,
            contributors_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            last_refresh: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Checks if a user is making their first contribution
    pub async fn is_first_contribution(&self, username: &str) -> Result<bool> {
        self.refresh_cache_if_needed().await?;
        
        let cache = self.contributors_cache.read().await;
        Ok(!cache.contains_key(username))
    }

    /// Gets detailed information about a contributor
    pub async fn get_contributor_info(&self, username: &str) -> Result<Option<ContributorInfo>> {
        self.refresh_cache_if_needed().await?;
        
        let cache = self.contributors_cache.read().await;
        Ok(cache.get(username).cloned())
    }

    /// Refreshes the cache if it's expired
    async fn refresh_cache_if_needed(&self) -> Result<()> {
        let now = Utc::now();
        let last_refresh = *self.last_refresh.read().await;
        
        if (now - last_refresh).num_seconds() as u64 > self.cache_ttl {
            self.refresh_cache().await?;
        }
        
        Ok(())
    }

    /// Refreshes the contributor cache
    async fn refresh_cache(&self) -> Result<()> {
        info!("Refreshing contributor cache for {}/{}", self.repo_owner, self.repo_name);
        
        let mut cache = self.contributors_cache.write().await;
        let mut new_cache: HashMap<String, ContributorInfo> = HashMap::new();

        // Get all commits
        let commits = self.client
            .repos(&self.repo_owner, &self.repo_name)
            .list_commits()
            .per_page(100) // Maximum allowed per page
            .send()
            .await?;

        for commit in commits.items {
            if let Some(author) = commit.author {
                let username = author.login;
                // Safely access the commit date through the commit author
                if let Some(commit_author) = &commit.commit.author {
                    if let Some(date) = commit_author.date {
                        match new_cache.get_mut(&username) {
                            Some(info) => {
                                info.total_commits += 1;
                                if date < info.first_contribution_date {
                                    info.first_contribution_date = date;
                                }
                                if date > info.latest_contribution_date {
                                    info.latest_contribution_date = date;
                                }
                            }
                            None => {
                                new_cache.insert(username.clone(), ContributorInfo {
                                    username,
                                    total_commits: 1,
                                    first_contribution_date: date,
                                    latest_contribution_date: date,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Update the cache
        *cache = new_cache;
        *self.last_refresh.write().await = Utc::now();
        
        info!("Successfully refreshed contributor cache with {} contributors", cache.len());
        Ok(())
    }
}
