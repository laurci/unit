use cmd::start_cli;
use unit_utils::{env, Result};

mod cmd;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    env::load_env();

    start_cli().await?;

    Ok(())
}
