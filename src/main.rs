use x_bot::{
    config::env::Config,
    github::client::GitHubClient,
    webhook::handler::{
        WebhookHandler,
        AppState,
        handle_webhook,
        health_check, 
        call_back},
    x::{
        client::XClient,
        // post_queue::PostQueue,
        // scheduler::PostScheduler
    }};
use std::sync::Arc;
use tokio::net::TcpListener;
use axum::{
    Router,
    routing::{post, get}};
use anyhow::Result;
use tracing::{info, debug};
use tracing_subscriber::{
    layer::SubscriberExt, 
    util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    
    // Clear the terminal
    std::process::Command::new("clear").status().unwrap();println!("\n");
    
    // Load configuration
    let config = Config::from_env()?;
        
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string())
        ))
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();

    info!("Starting X Bot with log level: {}", config.log_level);
    debug!("Debug logging is enabled");

    // Use rate limiting
    println!("Rate limit: {} requests per {} seconds", 
        config.rate_limit.max_requests, 
        config.rate_limit.window_seconds);
 
    // Use retry configuration
    println!("Retrying up to {} times", config.retry.max_attempts);

    // Get webhook URL
    println!("Webhook URL: {}", config.webhook_url());
    
    // Initialize GitHub client
    let github_client = GitHubClient::new(
        config.secrets.github_token().to_owned(),
        config.repo_owner.clone(),
        config.repo_name.clone()
    ).await?;

    // Initialize X client
    let x_client = Arc::new(XClient::new(
        config.secrets.x_api_key().to_owned(),
        config.secrets.x_api_secret().to_owned(),
        config.secrets.x_access_token().to_owned(),
        config.secrets.x_access_secret().to_owned()
    ).await?);
    
    // Create webhook handler
    let webhook_handler = WebhookHandler::new(
        github_client,
        Arc::clone(&x_client),
    );

    // Create app state
    let state = Arc::new(AppState {
        webhook_handler,
    });

    // Build router
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/health", get(health_check))
        .route("/callback", get(call_back))
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Listening on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}