use tonic::{metadata::MetadataValue, Request, Status};

use crate::config::CONFIG;

pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = CONFIG.grpc_api_key.parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
