use axum::extract::ws::Message;
use log::info;
use tokio::sync::broadcast;
use unit_crossbar::CrossbarMessage;

#[derive(Clone, Debug)]
pub enum BusMessage {
    Ready {
        connection_id: String,
    },
    Kill {
        connection_id: String,
    },
    TxWsMessage {
        connection_id: String,
        message: Message,
    },
    RxWsMessage {
        connection_id: String,
        message: Message,
    },
    CrossbarMessage(CrossbarMessage),
}

#[derive(Clone)]
pub struct Bus {
    channel: broadcast::Sender<BusMessage>,
}

impl Bus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { channel: tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BusMessage> {
        self.channel.subscribe()
    }

    pub fn send(&self, msg: BusMessage) {
        let _ = self.channel.send(msg);
    }
}

pub fn start_bus_monitor_task(bus: Bus) {
    tokio::spawn(async move {
        while let Ok(msg) = bus.subscribe().recv().await {
            match msg {
                BusMessage::Ready { connection_id } => {
                    info!("[{}] client connection ready", connection_id);
                }
                BusMessage::Kill { connection_id } => {
                    info!("[{}] client disconnected", connection_id);
                }
                BusMessage::RxWsMessage {
                    connection_id,
                    message,
                } => {
                    info!("[{}] ws rx {:?}", connection_id, message);
                }
                BusMessage::TxWsMessage {
                    connection_id,
                    message,
                } => {
                    info!("[{}] ws tx {:?}", connection_id, message);
                }
                BusMessage::CrossbarMessage(msg) => {
                    info!("[{}] crossbar_message {:?}", msg.topic, msg.content);
                }
            };
        }
    });
}
