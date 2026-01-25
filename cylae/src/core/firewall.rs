use std::process::Command;
use log::{info, warn, error};
use anyhow::{Result, Context};
use which::which;

pub struct FirewallManager;

impl FirewallManager {
    pub fn configure_firewall() -> Result<()> {
        info!("Configuring Firewall");

        if which("ufw").is_err() {
            warn!("ufw not found. Skipping firewall configuration.");
            return Ok(());
        }

        // Check status
        let status = Command::new("ufw")
            .arg("status")
            .output()
            .context("Failed to check ufw status")?;

        let stdout = String::from_utf8_lossy(&status.stdout);
        if stdout.contains("Status: active") {
            info!("Firewall is already active.");
            return Ok(());
        }

        info!("Setting up basic firewall rules...");
        let commands = [
            vec!["default", "deny", "incoming"],
            vec!["default", "allow", "outgoing"],
            vec!["allow", "ssh"],
            vec!["allow", "22/tcp"],
        ];

        for args in commands {
            let res = Command::new("ufw")
                .args(&args)
                .status()
                .context(format!("Failed to run ufw {:?}", args))?;

            if !res.success() {
                warn!("Failed to apply rule: ufw {:?}", args);
            }
        }

        // Enable without prompt
        info!("Enabling firewall...");
        let enable = Command::new("ufw")
            .args(["--force", "enable"])
            .status()
            .context("Failed to enable ufw")?;

        if enable.success() {
            info!("Firewall configured and enabled.");
        } else {
            error!("Failed to enable firewall.");
            anyhow::bail!("Failed to enable firewall.");
        }

        Ok(())
    }
}
