# Delta 𝕏 Bot

The official 𝕏 bot for Delta.

## What needs to be done

We need an X bot that posts updates to `@deltaml_org` with information from GitHub API.

**You will use the GitHub API to:**

- Detect when a new contributor makes their first commit to the `master` branch of the delta repository.
- Detect when a new release of the delta repository is published.

**You will use the X API to:**

- Post a message to `@deltaml_org` whenever a new contributor makes their first commit to the `master` branch of the delta repository.
- Post a message to `@deltaml_org` whenever a new release of the delta repository is published.

### We need (for now) two types of posts:

#### 1. For new contributors:

```
Delta got a new contributor [Contributor Name]!

Details: [Commit message]  

Link: [Commit link]
```

#### 2. For new releases:

```
New release ([Version Number]) of Delta out! 🎉
  
Link to release notes: [Release link]
```

Implement these features in the bot, ensuring the messages are posted automatically whenever these events occur.

---



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

---



If anything is unclear, reach out in the [Github Discussions](https://github.com/orgs/delta-rs/discussions/categories/general) here on GitHub.


## Contributors

The following contributors have either helped to start this project, have contributed
code, are actively maintaining it (including documentation), or in other ways
being awesome contributors to this project. **We'd like to take a moment to recognize them.**

[<img src="https://github.com/mjovanc.png?size=72" alt="mjovanc" width="72">](https://github.com/mjovanc)
[<img src="https://github.com/dmbtechdev.png?size=72" alt="dmbtechdev" width="72">](https://github.com/dmbtechdev)

## License

The MIT License.
