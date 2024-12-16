use crate::{error::Error, github::GithubTweetProducer};
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};
use twitter_v2::{authorization::BearerToken, TwitterApi};

pub struct XRepoAutoPoster {
    bearer_token: String,
    tweet_producers: Vec<GithubTweetProducer>,
    interval_secs: u64,
}

impl XRepoAutoPoster {
    pub fn new(
        bearer_token: impl Into<String>,
        tweet_producers: Vec<GithubTweetProducer>,
        interval_secs: u64,
    ) -> Self {
        Self {
            bearer_token: bearer_token.into(),
            tweet_producers,
            interval_secs,
        }
    }

    async fn publish_tweet(&self, tweet: impl Into<String>) -> Result<(), Error> {
        let auth = BearerToken::new(&self.bearer_token);
        TwitterApi::new(auth)
            .post_tweet()
            .text(tweet.into())
            .send()
            .await?;
        Ok(())
    }

    pub async fn run(&self) {
        let mut start = SystemTime::now();

        loop {
            sleep(Duration::from_secs(self.interval_secs));
            let end = SystemTime::now();

            for tp in &self.tweet_producers {
                match tp.try_produce(start.into(), end.into()).await {
                    Ok(tweets) => {
                        for tweet in &tweets {
                            let _ = self
                                .publish_tweet(tweet)
                                .await
                                .map_err(|err| println!("{}", err));
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            }

            start = end;
        }
    }
}
