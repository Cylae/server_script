use nix::unistd::Uid;
use std::process::Command;
use std::path::Path;
use std::fs;
use anyhow::{Result, Context, bail};
use log::{info, warn};

pub fn check_root() -> Result<()> {
    if !Uid::effective().is_root() {
        bail!("This application must be run as root.");
    }
    Ok(())
}

pub fn install_dependencies() -> Result<()> {
    info!("Checking system dependencies...");

    // Check for apt-get
    if Command::new("which").arg("apt-get").output().is_err() {
        bail!("apt-get not found. This tool supports Debian/Ubuntu based systems only.");
    }

    let pkgs = vec![
        "curl", "git", "ufw", "lsb-release", "ca-certificates", "gnupg",
        "htop", "iotop", "net-tools", // Useful utils
        "build-essential"
    ];

    info!("Installing dependencies: {:?}", pkgs);
    let status = Command::new("apt-get")
        .arg("install")
        .arg("-y")
        .args(&pkgs)
        .status()
        .context("Failed to execute apt-get")?;

    if !status.success() {
        bail!("Failed to install dependencies via apt-get");
    }

    Ok(())
}

pub fn apply_optimizations() -> Result<()> {
    info!("Applying system optimizations for media server performance...");

    let config = r#"# Cylae Media Server Optimizations
fs.inotify.max_user_watches=524288
vm.swappiness=10
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
"#;

    let path = Path::new("/etc/sysctl.d/99-cylae-optimization.conf");
    fs::write(path, config).context("Failed to write sysctl config")?;

    let status = Command::new("sysctl")
        .arg("--system")
        .status()
        .context("Failed to reload sysctl")?;

    if !status.success() {
        warn!("sysctl reload returned non-zero exit code");
    }

    Ok(())
}
