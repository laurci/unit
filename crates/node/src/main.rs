mod bus;
mod config;
mod crossbar;
mod runtime;
mod server;

use unit_utils::Result;

use crate::{
    bus::{start_bus_monitor_task, Bus},
    crossbar::start_crossbar_monitor_task,
    server::serve_ws,
};

fn setup_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger();

    let bus = Bus::new();

    start_bus_monitor_task(bus.clone());
    start_crossbar_monitor_task(bus.clone()).await?;
    serve_ws("0.0.0.0:6447".to_owned(), bus).await?;

    Ok(())
}
