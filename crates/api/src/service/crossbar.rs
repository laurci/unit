mod rpc_crossbar {
    tonic::include_proto!("unit.crossbar");
}

use futures::StreamExt;
use tonic::{Request, Response, Status};
use unit_crossbar::{encode_crossbar_message, CrossbarMessage, CROSSBAR_TOPIC};
use unit_pubsub::PubSub;
use unit_utils::Result;

use self::rpc_crossbar::crossbar_server::Crossbar;
pub use self::rpc_crossbar::crossbar_server::CrossbarServer;

pub struct CrossbarService {
    pubsub: PubSub,
}

impl CrossbarService {
    pub fn new(pubsub: PubSub) -> Self {
        Self { pubsub }
    }
}

#[tonic::async_trait]
impl Crossbar for CrossbarService {
    async fn push(
        &self,
        request: Request<rpc_crossbar::PushRequest>,
    ) -> Result<Response<rpc_crossbar::PushResponse>, Status> {
        let request: rpc_crossbar::PushRequest = request.into_inner();

        if request.message.is_none() {
            return Err(Status::invalid_argument("message is empty"));
        }
        let message = request.message.unwrap();
        let topic = request.topic;

        let message = match message {
            rpc_crossbar::push_request::Message::Binary(bytes) => {
                CrossbarMessage::binary(topic, bytes.message)
            }
            rpc_crossbar::push_request::Message::Text(text) => {
                CrossbarMessage::text(topic, text.message)
            }
        };

        let Ok(message) = encode_crossbar_message(message) else {
            return Err(Status::internal("Failed to encode message"));
        };

        let Ok(_) = self.pubsub.publish(CROSSBAR_TOPIC, message).await else {
            return Err(Status::internal("Failed to publish message"));
        };

        Ok(Response::new(rpc_crossbar::PushResponse {}))
    }

    async fn push_stream(
        &self,
        request: Request<tonic::Streaming<rpc_crossbar::PushRequest>>,
    ) -> Result<Response<rpc_crossbar::PushResponse>, Status> {
        let mut stream = request.into_inner();

        while let Some(request) = stream.next().await {
            let request = request?;
            if request.message.is_none() {
                return Err(Status::invalid_argument("message is empty"));
            }

            let message = request.message.unwrap();
            let topic = request.topic;

            let message = match message {
                rpc_crossbar::push_request::Message::Binary(bytes) => {
                    CrossbarMessage::binary(topic, bytes.message)
                }
                rpc_crossbar::push_request::Message::Text(text) => {
                    CrossbarMessage::text(topic, text.message)
                }
            };

            let Ok(message) = encode_crossbar_message(message) else {
                return Err(Status::internal("Failed to encode message"));
            };

            let Ok(_) = self.pubsub.publish(CROSSBAR_TOPIC, message).await else {
                return Err(Status::internal("Failed to publish message"));
            };
        }

        Ok(Response::new(rpc_crossbar::PushResponse {}))
    }
}
