use anyhow::{bail, Context, Result};
use log::{info, warn};
use std::process::Command;
use which::which;

pub fn configure() -> Result<()> {
    info!("Configuring Firewall (UFW)...");

    // Check if ufw exists
    if which("ufw").is_err() {
        warn!("UFW not found. Skipping firewall configuration.");
        return Ok(());
    }

    // Reset? No, let's just apply rules idempotent-ish.

    // Default Deny Incoming
    run_ufw(&["default", "deny", "incoming"])?;

    // Default Allow Outgoing
    run_ufw(&["default", "allow", "outgoing"])?;

    // Allow SSH
    run_ufw(&["allow", "ssh"])?;
    run_ufw(&["allow", "22/tcp"])?;

    // Enable
    info!("Enabling UFW...");
    run_ufw(&["--force", "enable"])?;

    Ok(())
}

fn run_ufw(args: &[&str]) -> Result<()> {
    let status = Command::new("ufw")
        .args(args)
        .status()
        .context("Failed to execute ufw command")?;

    if !status.success() {
        bail!("ufw command failed: {:?}", args);
    }
    Ok(())
}
