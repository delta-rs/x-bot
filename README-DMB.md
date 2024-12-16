# X Bot for Delta Repository

A Rust-based bot that automatically posts updates about the Delta repository to X (Twitter).
Based on X tweet: https://x.com/mjovanc/status/1866861106423074952

## Features

- Detects and announces new contributors to the Delta repository
- Posts notifications about new releases
- Uses webhook-based architecture for real-time updates

## Message Types

1. New Contributor Message:
```
Delta got a new contributor [Contributor Name]!
Details: [Commit message]
Link: [Commit link]
```

2. New Release Message:
```
New release ([Version Number]) of Delta out! 🎉
Link to release notes: [Release link]
```

## Setup

1. Clone the repository
2. Set up environment variables:
   - `GITHUB_TOKEN`: GitHub API token
   - `X_API_KEY`: X (Twitter) API key
   - `X_API_SECRET`: X (Twitter) API secret
   - `X_ACCESS_TOKEN`: X (Twitter) access token
   - `X_ACCESS_SECRET`: X (Twitter) access token secret
   - `REPO_OWNER`: The owner of the GitHub repository
   - `REPO_NAME`: The name of the GitHub repository


## Github Token Setup

1. Go to your GitHub account settings
2. Navigate to Developer settings > Personal access tokens
3. Generate a new token (classic) with the `repo` and `admin:repo_hook` scopes

## Github Repo Webhook Setup

1. Go to your GitHub repository settings
2. Navigate to Webhooks > Add webhook
3. Enter the webhook URL from the environment variable `WEBHOOK_PATH`
4. Select the events, "Push" and "Releases" you want to trigger the webhook

## Webhooks

* **Definition** : Webhooks are HTTP callbacks that allow one application to send real-time data to another whenever a specific event occurs.
* **Communication** : They use a request-response model. When an event occurs, the source application makes an HTTP POST request to a predefined URL (the webhook endpoint) on the target application, sending data about the event.
* **Use Case** : Commonly used for event-driven architectures where you want to notify another service about events, such as GitHub sending a notification about a new commit or release.
* **Example** : When a new issue is created in a GitHub repository, GitHub can send a webhook notification to your application to inform it of the new issue.


## X api Setup
Get these from the X Developer Portal (https://developer.twitter.com/en/portal/dashboard)

1. Go to your X (Twitter) dashboard
2. Navigate to your API keys
3. Create a new API key

## Testing and Development 

   - This bot is tested as the server running on a Local machine with Ubuntu 24.10.
   - ngrok, from https://ngrok.com/, is used to expose the webhook URL to the public internet for local development.
   - A blank (cargo new bot_test) repo is used for testing for release and push events.
   - Example of a tweet for a release: https://x.com/dmbtechdev/status/1868616134259343861


## Project Structure

```
src/
├── github/     # GitHub API integration
├── x/          # X API integration
├── webhook/    # Webhook handling
└── config/     # Configuration management
```

## Feel free to contribute or comment on the repo
https://github.com/dmbtechdev/x-bot


## Forked Repo
https://github.com/delta-rs/x-bot


## License

[MIT License](LICENSE)