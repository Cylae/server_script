use anyhow::{bail, Context, Result};
use log::{info, warn};
use std::process::Command;
use which::which;

pub fn check_installation() -> bool {
    which("docker").is_ok()
}

pub fn install_with_ops(ops: &dyn crate::core::ops::DockerOps) -> Result<()> {
    if ops.is_installed() {
        info!("Docker is already installed.");
        return Ok(());
    }
    ops.install()
}

pub fn install() -> Result<()> {
    if check_installation() {
        info!("Docker is already installed.");
        return Ok(());
    }

    info!("Docker not found. Installing via official script...");

    let tmp_dir = std::env::temp_dir();
    let script_path = tmp_dir.join(format!("get-docker-{:08x}.sh", rand::random::<u32>()));

    // Download script
    let status = Command::new("curl")
        .args(["-fsSL", "https://get.docker.com", "-o"])
        .arg(&script_path)
        .status()
        .context("Failed to download Docker install script")?;

    if !status.success() {
        let _ = std::fs::remove_file(&script_path);
        bail!("Failed to download get-docker.sh");
    }

    // Run script
    let status = Command::new("sh")
        .arg(&script_path)
        .status()
        .context("Failed to execute Docker install script")?;

    // Cleanup
    if let Err(e) = std::fs::remove_file(&script_path) {
        warn!("Failed to remove temporary install script: {}", e);
    }

    if !status.success() {
        bail!("Docker installation script failed");
    }

    info!("Docker installed successfully.");
    Ok(())
}
