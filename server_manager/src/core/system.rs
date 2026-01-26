use nix::unistd::{Uid, User};
use std::process::Command;
use std::path::Path;
use std::fs;
use std::env;
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

    info!("Updating package lists...");
    let update_status = Command::new("apt-get")
        .arg("update")
        .status()
        .context("Failed to execute apt-get update")?;

    if !update_status.success() {
        warn!("apt-get update failed, continuing with install...");
    }

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

pub fn ensure_media_user() -> Result<(u32, u32)> {
    // 1. Check if running under sudo (use the real user)
    if let Ok(sudo_user) = env::var("SUDO_USER") {
        if !sudo_user.is_empty() {
            info!("Detected sudo user: {}", sudo_user);
            if let Ok(Some(user)) = User::from_name(&sudo_user) {
                info!("Using detected user: {} (UID: {}, GID: {})", sudo_user, user.uid, user.gid);
                return Ok((user.uid.as_raw(), user.gid.as_raw()));
            }
        }
    }

    // 2. Check if 'server_manager' user exists
    if let Ok(Some(user)) = User::from_name("server_manager") {
        info!("Found existing system user: server_manager (UID: {})", user.uid);
        return Ok((user.uid.as_raw(), user.gid.as_raw()));
    }

    // 3. Create 'server_manager' user
    info!("Creating system user: server_manager");
    let status = Command::new("useradd")
        .arg("-r") // System account
        .arg("-m") // Create home directory (useful for some configs)
        .arg("-d").arg("/opt/server_manager") // Home is install dir
        .arg("-s").arg("/bin/bash") // Shell allowed for debugging
        .arg("server_manager")
        .status()
        .context("Failed to create user server_manager")?;

    if !status.success() {
        bail!("Failed to execute useradd command");
    }

    // Verify creation
    if let Ok(Some(user)) = User::from_name("server_manager") {
        Ok((user.uid.as_raw(), user.gid.as_raw()))
    } else {
        bail!("User server_manager was created but could not be found")
    }
}

pub fn chown_recursive(path: &Path, uid: u32, gid: u32) -> Result<()> {
    info!("Recursively setting ownership of {:?} to {}:{}", path, uid, gid);
    let status = Command::new("chown")
        .arg("-R")
        .arg(format!("{}:{}", uid, gid))
        .arg(path)
        .status()
        .context("Failed to execute chown")?;

    if !status.success() {
        warn!("chown command returned non-zero exit code");
    }
    Ok(())
}

pub fn apply_optimizations() -> Result<()> {
    info!("Applying system optimizations for media server performance...");

    let config = r#"# Server Manager Media Server Optimizations
fs.inotify.max_user_watches=524288
vm.swappiness=10
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
"#;

    let path = Path::new("/etc/sysctl.d/99-server-manager-optimization.conf");
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
