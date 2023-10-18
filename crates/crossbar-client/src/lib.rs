pub mod rpc_crossbar {
    tonic::include_proto!("unit.crossbar");
}

use std::str::FromStr;

// use rpc_admin::{a::EchoClient, EchoRequest};
use rpc_crossbar::{crossbar_client::CrossbarClient, push_request};
pub use rpc_crossbar::{push_request::Message, PushBinary, PushRequest, PushResponse, PushText};
use tonic::{
    codegen::InterceptedService,
    metadata::MetadataValue,
    service::Interceptor,
    transport::{Channel, Endpoint},
    Request, Status,
};
use unit_utils::Result;

pub struct AuthInterceptor {
    api_key: String,
}

impl AuthInterceptor {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let Ok(token) = self.api_key.parse::<MetadataValue<_>>() else {
            return Err(Status::unauthenticated("Missing API key"));
        };

        req.metadata_mut().insert("authorization", token.clone());

        Ok(req)
    }
}

pub struct Crossbar {
    pub client: CrossbarClient<InterceptedService<Channel, AuthInterceptor>>,
}

impl Crossbar {
    pub async fn new(endpoint: String, api_key: String) -> Result<Crossbar> {
        let channel = Endpoint::from_str(&endpoint)?.connect().await?;

        let client = CrossbarClient::with_interceptor(channel, AuthInterceptor::new(api_key));

        Ok(Crossbar { client })
    }

    pub async fn push(&mut self, req: PushRequest) -> Result<PushResponse> {
        let response = self.client.push(Request::new(req)).await?;
        let response = response.into_inner();
        Ok(response)
    }

    pub async fn push_text(&mut self, topic: String, text: String) -> Result<PushResponse> {
        let req = PushRequest {
            topic,
            message: Some(push_request::Message::Text(PushText { message: text })),
        };
        self.push(req).await
    }

    pub async fn push_binary(&mut self, topic: String, bytes: Vec<u8>) -> Result<PushResponse> {
        let req = PushRequest {
            topic,
            message: Some(push_request::Message::Binary(PushBinary { message: bytes })),
        };
        self.push(req).await
    }
}
