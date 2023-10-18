use clap::{Parser, Subcommand};
use unit_utils::{err::bail, Result};

use self::deploy::Deploy;

mod deploy;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "CLI to interact with unit API")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Deploy code to unit
    Deploy(Deploy),
}

pub async fn start_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Deploy(deploy)) => deploy::run_deploy(deploy).await?,
        None => bail!("No command provided"),
    };

    Ok(())
}

