use std::process::Command;
use anyhow::{Result, Context, bail};
use log::info;
use which::which;
use std::path::Path;
use std::fs;

pub struct DockerManager;

impl DockerManager {
    pub fn new() -> Self {
        Self
    }

    pub fn check_and_install_docker(&self) -> Result<()> {
        info!("Checking Docker Installation...");

        if which("docker").is_ok() {
            info!("Docker is already installed.");
            return Ok(());
        }

        info!("Docker not found. Installing via official script...");
        self.install_docker()?;

        info!("Docker installed successfully.");
        Ok(())
    }

    fn install_docker(&self) -> Result<()> {
        let script_url = "https://get.docker.com";
        let script_name = "get-docker.sh";

        // Download script
        let status = Command::new("curl")
            .args(&["-fsSL", script_url, "-o", script_name])
            .status()
            .context("Failed to download Docker install script")?;

        if !status.success() {
            bail!("Failed to download Docker install script.");
        }

        // Run script
        let status = Command::new("sh")
            .arg(script_name)
            .status()
            .context("Failed to run Docker install script")?;

        if !status.success() {
            bail!("Docker installation script failed.");
        }

        // Cleanup
        if Path::new(script_name).exists() {
            let _ = fs::remove_file(script_name);
        }

        Ok(())
    }
}
