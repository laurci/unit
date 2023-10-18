use fred::{
    prelude::{ClientLike, RedisClient},
    types::{Builder, RedisConfig},
};
use unit_utils::{shared_config::ConfigRedis, Result};

pub use fred::prelude::{PubsubInterface, RedisValue};

pub struct PubSub {
    pub subscriber: RedisClient,
    pub publisher: RedisClient,
}

impl PubSub {
    pub async fn connect(config: ConfigRedis) -> Result<PubSub> {
        let config = RedisConfig::from_url(&config.to_url())?;

        let subscriber = Builder::from_config(config.clone()).build()?;
        let publisher = subscriber.clone_new();

        subscriber.connect();
        publisher.connect();

        subscriber.wait_for_connect().await?;
        publisher.wait_for_connect().await?;

        Ok(PubSub {
            subscriber,
            publisher,
        })
    }

    pub async fn publish(&self, topic: &str, message: Vec<u8>) -> Result<()> {
        self.publisher
            .publish(topic, RedisValue::Bytes(message.into()))
            .await?;
        Ok(())
    }
}
