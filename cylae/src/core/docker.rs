use std::process::Command;
use anyhow::{Result, Context, bail};
use log::info;
use which::which;

pub fn check_installation() -> bool {
    which("docker").is_ok()
}

pub fn install() -> Result<()> {
    if check_installation() {
        info!("Docker is already installed.");
        return Ok(());
    }

    info!("Docker not found. Installing via official script...");

    // Download script
    let status = Command::new("curl")
        .args(&["-fsSL", "https://get.docker.com", "-o", "get-docker.sh"])
        .status()
        .context("Failed to download Docker install script")?;

    if !status.success() {
        bail!("Failed to download get-docker.sh");
    }

    // Run script
    let status = Command::new("sh")
        .arg("get-docker.sh")
        .status()
        .context("Failed to execute Docker install script")?;

    if !status.success() {
        bail!("Docker installation script failed");
    }

    // Cleanup
    let _ = std::fs::remove_file("get-docker.sh");

    info!("Docker installed successfully.");
    Ok(())
}
