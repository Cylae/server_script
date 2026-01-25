use std::process::Command;
use anyhow::{Result, Context};
use log::{info, warn};
use which::which;

pub struct FirewallManager;

impl FirewallManager {
    pub fn new() -> Self {
        Self
    }

    pub fn configure_firewall(&self) -> Result<()> {
        info!("Configuring Firewall...");

        if which("ufw").is_err() {
            warn!("ufw not found. Skipping firewall configuration.");
            return Ok(());
        }

        // Check if already active
        let status_output = Command::new("ufw")
            .arg("status")
            .output()
            .context("Failed to check ufw status")?;

        let status_str = String::from_utf8_lossy(&status_output.stdout);
        if status_str.contains("Status: active") {
            info!("Firewall is already active.");
            return Ok(());
        }

        info!("Setting up basic firewall rules...");
        self.run_ufw(&["default", "deny", "incoming"])?;
        self.run_ufw(&["default", "allow", "outgoing"])?;
        self.run_ufw(&["allow", "ssh"])?;
        self.run_ufw(&["allow", "22/tcp"])?;

        info!("Enabling firewall...");
        self.run_ufw(&["--force", "enable"])?;

        info!("Firewall configured and enabled.");
        Ok(())
    }

    fn run_ufw(&self, args: &[&str]) -> Result<()> {
        let status = Command::new("ufw")
            .args(args)
            .status()
            .context(format!("Failed to execute ufw {:?}", args))?;

        if !status.success() {
            warn!("ufw command failed: {:?}", args);
        }
        Ok(())
    }
}
