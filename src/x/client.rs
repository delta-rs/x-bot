use std::sync::{atomic::{AtomicU64, Ordering}, Arc};
use tokio::time::{sleep, Duration};
use twitter_v2::{
    authorization::Oauth1aToken, 
    TwitterApi};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use chrono::Utc;

const MAX_RETRIES: u32 = 3;
const RATE_LIMIT_WINDOW: u64 = 15 * 60; // 15 minutes in seconds
const TWEETS_PER_WINDOW: u64 = 50; // X API allows 50 tweets per 15 minutes

pub struct XClient {
    client: TwitterApi<Oauth1aToken>,
    tweet_count: Arc<AtomicU64>,
    window_start: Arc<AtomicU64>,
}

impl XClient {
    /// Creates a new instance of `XClient` with OAuth 1.0a credentials.
    ///
    /// # Arguments
    /// * `api_key` - The API key (Consumer Key)
    /// * `api_secret` - The API secret (Consumer Secret)
    /// * `access_token` - The access token
    /// * `access_secret` - The access token secret
    ///
    /// # Returns
    /// A result containing the initialized `XClient` or an error if initialization fails.
    pub async fn new(
        api_key: String,
        api_secret: String,
        access_token: String,
        access_secret: String,
    ) -> Result<Self> {
        let auth = Oauth1aToken::new(
            api_key,
            api_secret,
            access_token,
            access_secret,
        );
        let client = TwitterApi::new(auth);
        
        info!("X Api Client initialized");
        
        Ok(Self { 
            client,
            tweet_count: Arc::new(AtomicU64::new(0)),
            window_start: Arc::new(AtomicU64::new(Utc::now().timestamp() as u64)),
        })
    }

    /// Posts a tweet with retry mechanism and rate limiting
    pub async fn post_with_retry(&self, text: &str) -> Result<String> {
        info!("Attempting to post tweet: {}", text);
        
        for attempt in 1..=MAX_RETRIES {
            match self.send_tweet(text).await {
                Ok(id) => {
                    info!("Successfully posted tweet with ID: {}", id);
                    return Ok(id);
                }
                Err(e) => {
                    error!("Failed to post tweet (attempt {}/{}): {:?}", attempt, MAX_RETRIES, e);
                    if attempt < MAX_RETRIES {
                        warn!("Retrying in {} seconds...", attempt * 2);
                        sleep(Duration::from_secs(attempt as u64 * 2)).await;
                    }
                }
            }
        }
        
        Err(anyhow!("Failed to post tweet after {} attempts", MAX_RETRIES))
    }

    /// Posts a tweet with the specified text to Twitter.
    ///
    /// # Arguments
    /// * `text` - A string slice containing the text of the tweet.
    ///
    /// # Returns
    /// A result containing the tweet ID as a string if successful, or an error if the posting fails.
    pub async fn send_tweet(&self, text: &str) -> Result<String> {
        debug!("Checking rate limits before sending tweet");
        
        // Rate limiting check
        let now = Utc::now().timestamp() as u64;
        let window_start = self.window_start.load(Ordering::Relaxed);
        let tweet_count = self.tweet_count.load(Ordering::Relaxed);
        
        if now - window_start >= RATE_LIMIT_WINDOW {
            debug!("Rate limit window expired, resetting counts");
            self.window_start.store(now, Ordering::Relaxed);
            self.tweet_count.store(0, Ordering::Relaxed);
        } else if tweet_count >= TWEETS_PER_WINDOW {
            let wait_time = RATE_LIMIT_WINDOW - (now - window_start);
            warn!("Rate limit reached. Waiting {} seconds", wait_time);
            sleep(Duration::from_secs(wait_time)).await;
            self.window_start.store(now, Ordering::Relaxed);
            self.tweet_count.store(0, Ordering::Relaxed);
        }
        
        debug!("Sending tweet to X API");
        match self.client.post_tweet().text(text.to_owned()).send().await {
            Ok(response) => {
                info!("Tweet posted successfully");
                self.tweet_count.fetch_add(1, Ordering::Relaxed);
                match &response.data {
                    Some(tweet) => Ok(tweet.id.to_string()),
                    None => Err(anyhow!("No tweet data in response"))
                }
            }
            Err(e) => {
                error!("Error from X API: {:?}", e);
                Err(anyhow!("Failed to post tweet: {:?}", e))
            }
        }
    }
}
