pub mod rpc_admin {
    tonic::include_proto!("unit.admin");
}

use std::str::FromStr;

// use rpc_admin::{a::EchoClient, EchoRequest};
use rpc_admin::{admin_client::AdminClient, UpdateAppRequest};
use tonic::{
    codegen::InterceptedService,
    metadata::MetadataValue,
    service::Interceptor,
    transport::{Channel, Endpoint},
    Request, Status,
};
use unit_utils::{env, Result};

struct AuthInterceptor;

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let Ok(token) = env::required_str("UNIT_CLI_API_KEY").parse::<MetadataValue<_>>() else {
            return Err(Status::unauthenticated("Missing API key"));
        };

        req.metadata_mut().insert("authorization", token.clone());

        Ok(req)
    }
}

pub struct Admin {
    client: AdminClient<InterceptedService<Channel, AuthInterceptor>>,
}

impl Admin {
    pub async fn new() -> Result<Admin> {
        let endpoint = env::required_str("UNIT_CLI_API_ENDPOINT");
        let channel = Endpoint::from_str(&endpoint)?.connect().await?;

        let client = AdminClient::with_interceptor(channel, AuthInterceptor);

        Ok(Admin { client })
    }

    pub async fn update_app(&mut self, code: Vec<u8>) -> Result<()> {
        let response = self
            .client
            .update_app(Request::new(UpdateAppRequest { code }))
            .await?;

        println!("RESPONSE={:?}", response);

        Ok(())
    }
}
