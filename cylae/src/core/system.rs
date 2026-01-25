use std::process::Command;
use anyhow::{Result, bail, Context};
use log::{info, warn};
use std::fs;

pub struct SystemManager;

impl SystemManager {
    pub fn new() -> Self {
        Self
    }

    pub fn check_root(&self) -> Result<()> {
        let euid = unsafe { libc::geteuid() };
        if euid != 0 {
            bail!("This application must be run as root (sudo).");
        }
        Ok(())
    }

    pub fn install_system_deps(&self) -> Result<()> {
        info!("Checking system dependencies...");

        // Check for apt-get
        if which::which("apt-get").is_err() {
            bail!("This installer currently supports Debian/Ubuntu based systems only (apt-get not found).");
        }

        // Keeping legacy dependencies to ensure full compatibility, plus essentials.
        let packages = vec![
            "python3", "python3-venv", "python3-pip", "python3-dev",
            "git", "curl", "ufw", "build-essential", "libffi-dev", "libssl-dev"
        ];

        info!("Updating package list...");
        self.run_apt(&["update", "-qq"])?;

        info!("Installing packages: {}", packages.join(", "));
        let mut args = vec!["install", "-y", "-qq"];
        args.extend(packages);
        self.run_apt(&args)?;

        Ok(())
    }

    fn run_apt(&self, args: &[&str]) -> Result<()> {
        let status = Command::new("apt-get")
            .env("DEBIAN_FRONTEND", "noninteractive")
            .args(args)
            .status()
            .context("Failed to execute apt-get")?;

        if !status.success() {
            bail!("apt-get failed with status: {}", status);
        }
        Ok(())
    }

    pub fn optimize_system(&self) -> Result<()> {
        info!("Applying system optimizations...");

        let sysctl_conf_path = "/etc/sysctl.d/99-cylae-optimization.conf";
        // Combined optimizations from legacy code
        let config_content =
r#"# Cylae Media Server Optimizations
fs.inotify.max_user_watches=524288
vm.swappiness=10
vm.vfs_cache_pressure=50
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
net.core.somaxconn=4096
net.ipv4.tcp_rmem=4096 87380 67108864
net.ipv4.tcp_wmem=4096 65536 67108864
net.core.rmem_max=67108864
net.core.wmem_max=67108864
"#;

        fs::write(sysctl_conf_path, config_content)
            .context("Failed to write sysctl configuration")?;

        let status = Command::new("sysctl")
            .arg("--system")
            .status()
            .context("Failed to reload sysctl")?;

        if !status.success() {
            warn!("Failed to apply sysctl settings immediately. They will apply on reboot.");
        } else {
            info!("System optimizations applied successfully.");
        }

        Ok(())
    }
}
