use crate::{
    bus::{Bus, BusMessage},
    config::CONFIG,
};
use log::info;
use unit_crossbar::{decode_crossbar_message, CROSSBAR_TOPIC};
use unit_pubsub::{PubSub, PubsubInterface, RedisValue};
use unit_utils::Result;

pub async fn start_crossbar_monitor_task(bus: Bus) -> Result<()> {
    let pubsub = PubSub::connect(CONFIG.redis.clone()).await?;

    let mut stream = pubsub.subscriber.on_message();
    tokio::spawn(async move {
        while let Ok(msg) = stream.recv().await {
            match msg.value {
                RedisValue::String(text) => {
                    let bytes = text.as_bytes();
                    let bytes = bytes.to_vec();
                    let Ok(msg) = decode_crossbar_message(bytes) else {
                        continue;
                    };
                    bus.send(BusMessage::CrossbarMessage(msg));
                }
                _ => continue,
            }
        }
    });
    pubsub.subscriber.subscribe(CROSSBAR_TOPIC).await?;
    info!("monitor task started");

    Ok(())
}
