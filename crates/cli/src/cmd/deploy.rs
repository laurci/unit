use std::path::PathBuf;

use clap::Args;
use unit_utils::Result;

use crate::services::Admin;

#[derive(Args, Debug)]
pub struct Deploy {
    /// Path to code
    path: PathBuf,
    // #[arg(short = 'd', long = "digits")]
    // only_digits: bool,
}

pub async fn run_deploy(args: Deploy) -> Result<()> {
    println!("Deploying app from: {}", args.path.to_str().unwrap());

    let code = std::fs::read(args.path)?;

    let mut admin = Admin::new().await?;
    admin.update_app(code).await?;

    Ok(())
}
