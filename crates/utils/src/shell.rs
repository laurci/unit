use anyhow::{bail, Result};

pub fn run_cmd(cmd: &str) -> Result<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;

    if !output.status.success() {
        bail!("Command failed with status code {}: {}", output.status, cmd);
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn which(cmd: &str) -> Result<String> {
    run_cmd(&format!("which {}", cmd))
}
