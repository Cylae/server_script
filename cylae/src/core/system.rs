use std::process::Command;
use std::fs;
use std::path::Path;
use log::{info, warn};
use anyhow::{Result, Context};
use libc;

pub struct SystemManager;

impl SystemManager {
    pub fn check_root() -> Result<()> {
        unsafe {
            if libc::geteuid() != 0 {
                anyhow::bail!("This script must be run as root (sudo).");
            }
        }
        Ok(())
    }

    pub fn install_system_deps() -> Result<()> {
        info!("Installing System Dependencies");

        // Simple check for apt
        let status = Command::new("which")
            .arg("apt-get")
            .output()?;

        if !status.status.success() {
             anyhow::bail!("This installer currently supports Debian/Ubuntu based systems only (apt-get not found).");
        }

        info!("Updating package list...");
        let update = Command::new("apt-get")
            .args(["update", "-qq"])
            .status()
            .context("Failed to update apt")?;

        if !update.success() {
            warn!("apt-get update returned non-zero exit code.");
        }

        let packages = [
            "python3", "python3-venv", "python3-pip", "python3-dev",
            "git", "curl", "ufw", "build-essential", "libffi-dev", "libssl-dev"
        ];

        info!("Installing packages: {}", packages.join(", "));
        let install = Command::new("apt-get")
            .arg("install")
            .arg("-y")
            .arg("-qq")
            .args(packages)
            .status()
            .context("Failed to install packages")?;

        if !install.success() {
            anyhow::bail!("Failed to install system dependencies.");
        }

        info!("System dependencies installed.");
        Ok(())
    }

    pub fn optimize_system() -> Result<()> {
        info!("Applying System Optimizations");

        let sysctl_path = "/etc/sysctl.d/99-cylae-optimization.conf";
        let content = r#"# Cylae Media Server Optimizations
fs.inotify.max_user_watches=524288
vm.swappiness=10
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
"#;

        info!("Writing optimizations to {}...", sysctl_path);

        if !Path::new("/etc/sysctl.d").exists() {
             fs::create_dir_all("/etc/sysctl.d").context("Failed to create /etc/sysctl.d")?;
        }

        fs::write(sysctl_path, content).context("Failed to write sysctl config")?;

        info!("Applying sysctl settings...");
        let status = Command::new("sysctl")
            .arg("--system")
            .status()
            .context("Failed to run sysctl")?;

        if !status.success() {
             warn!("sysctl --system returned non-zero exit code. Please check your kernel configuration.");
        } else {
             info!("System optimizations applied.");
        }

        Ok(())
    }
}
