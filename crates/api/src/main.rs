use crate::config::CONFIG;

mod auth;
mod config;
mod server;
mod service;

use server::start_grpc_api;
use unit_index::Index;
use unit_pubsub::PubSub;
use unit_utils::Result;

fn setup_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger();

    let index = Index::load(CONFIG.storage_location.clone())?;
    let pubsub = PubSub::connect(CONFIG.redis.clone()).await?;

    let addr = format!("0.0.0.0:{port}", port = CONFIG.grpc_port);
    start_grpc_api(addr, index, pubsub).await?;

    Ok(())
}
