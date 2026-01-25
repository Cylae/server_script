use std::process::Command;
use std::path::Path;
use std::fs;
use log::info;
use anyhow::{Result, Context};
use which::which;

pub struct DockerManager;

impl DockerManager {
    pub fn check_and_install_docker() -> Result<()> {
        info!("Checking Docker Installation");

        if which("docker").is_ok() {
            info!("Docker is already installed.");
            return Ok(());
        }

        info!("Docker not found. Installing via official script...");
        let script_url = "https://get.docker.com";
        let script_path = "get-docker.sh";

        let status = Command::new("curl")
            .args(["-fsSL", script_url, "-o", script_path])
            .status()
            .context("Failed to download Docker install script")?;

        if !status.success() {
            anyhow::bail!("Failed to download Docker install script.");
        }

        let run_status = Command::new("sh")
            .arg(script_path)
            .status()
            .context("Failed to run Docker install script")?;

        if !run_status.success() {
             // cleanup
             let _ = fs::remove_file(script_path);
             anyhow::bail!("Docker installation script failed.");
        }

        if Path::new(script_path).exists() {
            let _ = fs::remove_file(script_path);
        }

        info!("Docker installed successfully.");
        Ok(())
    }
}
