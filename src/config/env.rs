use std::{
    env::var,
    str::FromStr,
    fmt::{Display, Formatter}};
use serde::Deserialize;
use anyhow::Context;


/// Runtime environment for the application
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

// impl Default for Environment {
//     fn default() -> Self {
//         Environment::Development
//     }
// }

// impl From<Environment> for String {
//     fn from(environment: Environment) -> String {
//         match environment {
//             Environment::Development => "development".to_string(),
//             Environment::Production => "production".to_string(),
//         }
//     }
// }

// convert string from env var file to Environment
impl FromStr for Environment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(anyhow::anyhow!("Invalid environment: {}", s)),
        }
    }
}

/// Server configuration settings
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub webhook_path: String,
}

// impl Default for ServerConfig {
//     fn default() -> Self {
//         Self {
//             host: "127.0.0.1".to_string(),
//             port: 7878,
//             webhook_path: "/webhook".to_string(),
//         }
//     }
// }

/// Rate limiting configuration
#[derive(Debug, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests per window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

// impl Default for RateLimitConfig {
//     fn default() -> Self {
//         Self {
//             max_requests: 100,
//             window_seconds: 3600,
//         }
//     }
// }

/// Retry configuration for failed operations
#[derive(Debug, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
}

// impl Default for RetryConfig {
//     fn default() -> Self {
//         Self {
//             max_attempts: 3,
//             initial_delay_ms: 1000,
//             max_delay_ms: 5000,
//         }
//     }
// }

/// Cache configuration
// #[derive(Debug, Deserialize)]
// pub struct CacheConfig {
//     /// Enable caching
//     pub enabled: bool,
//     /// Cache TTL in seconds
//     pub ttl_seconds: u64,
//     /// Maximum cache size in items
//     pub max_items: usize,
// }

// impl Default for CacheConfig {
//     fn default() -> Self {
//         Self {
//             enabled: true,
//             ttl_seconds: 3600,
//             max_items: 1000,
//         }
//     }
// }

/// API timeout configuration
#[derive(Debug, Deserialize)]
pub struct TimeoutConfig {
    /// Connect timeout in seconds
    pub connect_seconds: u64,
    /// Read timeout in seconds
    pub read_seconds: u64,
    /// Write timeout in seconds
    pub write_seconds: u64,
}

// impl Default for TimeoutConfig {
//     fn default() -> Self {
//         Self {
//             connect_seconds: 10,
//             read_seconds: 30,
//             write_seconds: 30,
//         }
//     }
// }

/// Sensitive configuration that should never be logged or displayed
#[derive(Debug,Deserialize)]
pub struct Secrets {
    /// GitHub personal access token for API authentication
    github_token: String,
        
    /// X API key for API authentication
    x_api_key: String,
    
    /// X API secret for API authentication
    x_api_secret: String,
    
    /// X access token for API authentication
    x_access_token: String,
    
    /// X access secret for API authentication
    x_access_secret: String,
}

impl Display for Secrets {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Secret tokens getters in a controlled manner
impl Secrets {
    pub fn github_token(&self) -> &str {
        &self.github_token
    }

    pub fn x_api_key(&self) -> &str {
        &self.x_api_key
    }

    pub fn x_api_secret(&self) -> &str {
        &self.x_api_secret
    }

    pub fn x_access_token(&self) -> &str {
        &self.x_access_token
    }

    pub fn x_access_secret(&self) -> &str {
        &self.x_access_secret
    }

    /// Validate all secrets
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.github_token.is_empty() {
            return Err(anyhow::anyhow!("GITHUB_TOKEN must be set"));
        }
        if self.github_token.len() != 40 {
            return Err(anyhow::anyhow!("GitHub token must be exactly 40 characters long"));
        } 
        if self.x_api_key.is_empty() {
            return Err(anyhow::anyhow!("X_API_KEY must be set"));
        }
        if self.x_api_key.len() < 25 {
            return Err(anyhow::anyhow!("X_API_KEY must be at least 32 characters long"));
        }
        if self.x_api_secret.is_empty() {
            return Err(anyhow::anyhow!("X_API_SECRET must be set"));
        }
        if self.x_api_secret.len() < 32 {
            return Err(anyhow::anyhow!("X_API_SECRET must be at least 32 characters long"));
        }
        if self.x_access_token.is_empty() {
            return Err(anyhow::anyhow!("X_ACCESS_TOKEN must be set"));
        }
        if self.x_access_token.len() < 32 {
            return Err(anyhow::anyhow!("X_ACCESS_TOKEN must be at least 32 characters long"));
        }
        if self.x_access_secret.is_empty() {
            return Err(anyhow::anyhow!("X_ACCESS_SECRET must be set"));
        }
        if self.x_access_secret.len() < 32 {
            return Err(anyhow::anyhow!("X_ACCESS_SECRET must be at least 32 characters long"));
        }
        Ok(())
    }
}

/// Configuration structure for the application
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Current runtime environment
    // #[serde(default)]
    pub environment: Environment,

    /// Server configuration
    // #[serde(default)]
    pub server: ServerConfig,

    /// Rate limiting configuration
    // #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Retry configuration
    // #[serde(default)]
    pub retry: RetryConfig,

    /// API timeout configuration
    // #[serde(default)]
    pub timeout: TimeoutConfig,

    /// Sensitive configuration values
    pub secrets: Secrets,
    
    /// GitHub repository owner (username or organization)
    pub repo_owner: String,
    
    /// GitHub repository name
    pub repo_name: String,

    /// Log level for the application
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    /// Loads configuration from environment variables
    ///
    /// # Returns
    /// A Result containing the Config if successful, or an error if any required
    /// environment variables are missing
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();

        // Load environment-specific settings
        let environment = var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .parse()?;

        // Load secrets first and validate them
        let secrets = Secrets {
            github_token: var("GITHUB_TOKEN")
                .context("GITHUB_TOKEN must be set")?,
            x_api_key: var("X_API_KEY")
                .context("X_API_KEY must be set")?,
            x_api_secret: var("X_API_SECRET")
                .context("X_API_SECRET must be set")?,
            x_access_token: var("X_ACCESS_TOKEN")
                .context("X_ACCESS_TOKEN must be set")?,
            x_access_secret: var("X_ACCESS_SECRET")
                .context("X_ACCESS_SECRET must be set")?,
        };
        secrets.validate()?;

        // Load server configuration
        let server = ServerConfig {
            host: var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: var("SERVER_PORT")
                .unwrap_or_else(|_| "7878".to_string())
                .parse()
                .context("SERVER_PORT must be a valid port number")?,
            webhook_path: var("WEBHOOK_PATH")
                .unwrap_or_else(|_| "/webhook".to_string()),
        };

        // Load rate limit configuration
        let rate_limit = RateLimitConfig {
            max_requests: var("RATE_LIMIT_MAX_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("RATE_LIMIT_MAX_REQUESTS must be a positive integer")?,
            window_seconds: var("RATE_LIMIT_WINDOW_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .context("RATE_LIMIT_WINDOW_SECONDS must be a positive integer")?,
        };

        // Load retry configuration
        let retry = RetryConfig {
            max_attempts: var("RETRY_MAX_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .context("RETRY_MAX_ATTEMPTS must be a positive integer")?,
            initial_delay_ms: var("RETRY_INITIAL_DELAY_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .context("RETRY_INITIAL_DELAY_MS must be a positive integer")?,
            max_delay_ms: var("RETRY_MAX_DELAY_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .context("RETRY_MAX_DELAY_MS must be a positive integer")?,
        };

        // Load timeout configuration
        let timeout = TimeoutConfig {
            connect_seconds: var("TIMEOUT_CONNECT_SECONDS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("TIMEOUT_CONNECT_SECONDS must be a positive integer")?,
            read_seconds: var("TIMEOUT_READ_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("TIMEOUT_READ_SECONDS must be a positive integer")?,
            write_seconds: var("TIMEOUT_WRITE_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("TIMEOUT_WRITE_SECONDS must be a positive integer")?,
        };

        let config = Config {
            environment,
            server,
            rate_limit,
            retry,
            timeout,
            secrets,
            repo_owner: var("REPO_OWNER")
                .context("REPO_OWNER must be set")?,
            repo_name: var("REPO_NAME")
                .context("REPO_NAME must be set")?,
            log_level: var("LOG_LEVEL")
                .unwrap_or_else(|_| default_log_level()),
        };

        config.validate()?;
        Ok(config)
    }

    /// Get secrets safely inside Config
    pub fn github_token(&self) -> &str {
        self.secrets.github_token()
    }

    pub fn x_api_key(&self) -> &str {
        self.secrets.x_api_key()
    }

    pub fn x_api_secret(&self) -> &str {
        self.secrets.x_api_secret()
    }

    pub fn x_access_token(&self) -> &str {
        self.secrets.x_access_token()
    }

    pub fn x_access_secret(&self) -> &str {
        self.secrets.x_access_secret()
    }

    /// Validates the configuration values
    fn validate(&self) -> anyhow::Result<()> {
        if self.repo_owner.is_empty() || self.repo_name.is_empty() {
            return Err(anyhow::anyhow!("Repository owner and name cannot be empty"));
        }

        match self.log_level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => Ok(()),
            _ => Err(anyhow::anyhow!("Invalid log level: {}", self.log_level)),
        }?;

        // Validate rate limit configuration
        if self.rate_limit.max_requests == 0 {
            return Err(anyhow::anyhow!("Rate limit max requests must be greater than 0"));
        }
        if self.rate_limit.window_seconds == 0 {
            return Err(anyhow::anyhow!("Rate limit window seconds must be greater than 0"));
        }

        // Validate retry configuration
        if self.retry.max_attempts == 0 {
            return Err(anyhow::anyhow!("Retry max attempts must be greater than 0"));
        }
        if self.retry.initial_delay_ms == 0 {
            return Err(anyhow::anyhow!("Retry initial delay must be greater than 0"));
        }
        if self.retry.max_delay_ms < self.retry.initial_delay_ms {
            return Err(anyhow::anyhow!("Retry max delay must be greater than or equal to initial delay"));
        }

        // Validate timeout configuration
        if self.timeout.connect_seconds == 0 {
            return Err(anyhow::anyhow!("Connect timeout must be greater than 0"));
        }
        if self.timeout.read_seconds == 0 {
            return Err(anyhow::anyhow!("Read timeout must be greater than 0"));
        }
        if self.timeout.write_seconds == 0 {
            return Err(anyhow::anyhow!("Write timeout must be greater than 0"));
        }

        Ok(())
    }

    // /// Returns true if running in development mode
    // pub fn is_development(&self) -> bool {
    //     matches!(self.environment, Environment::Development)
    // }

    // /// Returns true if running in production mode
    // pub fn is_production(&self) -> bool {
    //     matches!(self.environment, Environment::Production)
    // }

    // /// Get connect timeout as Duration
    // pub fn connect_timeout(&self) -> Duration {
    //     Duration::from_secs(self.timeout.connect_seconds)
    // }

    // /// Get read timeout as Duration
    // pub fn read_timeout(&self) -> Duration {
    //     Duration::from_secs(self.timeout.read_seconds)
    // }

    // /// Get write timeout as Duration
    // pub fn write_timeout(&self) -> Duration {
    //     Duration::from_secs(self.timeout.write_seconds)
    // }

    /// Get the full webhook URL path
    pub fn webhook_url(&self) -> String {
        format!("http://{}{}",self.server.host, self.server.webhook_path)
    }
}
