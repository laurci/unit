use std::{env, path::PathBuf};

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

    let code_path = if !args.path.is_absolute() {
        let cwd = env::current_dir()?;
        cwd.join(args.path)
    } else {
        args.path
    };

    let code = std::fs::read(code_path)?;

    let mut admin = Admin::new().await?;
    admin.update_app(code).await?;

    println!("Deployed app successfully");

    Ok(())
}
