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

If anything is unclear, reach out in the [Github Discussions](https://github.com/orgs/delta-rs/discussions/categories/general) here on GitHub.

## Contributors

The following contributors have either helped to start this project, have contributed
code, are actively maintaining it (including documentation), or in other ways
being awesome contributors to this project. **We'd like to take a moment to recognize them.**

[<img src="https://github.com/mjovanc.png?size=72" alt="mjovanc" width="72">](https://github.com/mjovanc)

## License

The MIT License.
