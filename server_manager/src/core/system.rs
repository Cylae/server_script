use nix::unistd::{Uid, User};
use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::Write;
use anyhow::{Result, Context, bail};
use log::{info, warn};
use sysinfo::{System, SystemExt};

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
        "htop", "iotop", "net-tools", "quota", // Useful utils
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

fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        bail!("Username cannot be empty");
    }
    // Allow alphanumeric, dashes, underscores. No spaces.
    if !username.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        bail!("Username contains invalid characters. Use alphanumeric, '-' or '_'.");
    }
    Ok(())
}

fn get_uid(username: &str) -> Result<u32> {
    match User::from_name(username).context("Failed to lookup user")? {
        Some(user) => Ok(user.uid.as_raw()),
        None => bail!("User '{}' not found", username),
    }
}

pub fn create_system_user(username: &str, password: &str) -> Result<()> {
    validate_username(username)?;

    // Check if user exists to prevent hijacking
    if get_uid(username).is_ok() {
        bail!("System user '{}' already exists. Aborting creation to prevent hijacking.", username);
    }

    info!("Creating system user '{}'...", username);
    // useradd -m -s /bin/bash <username>
    let status = Command::new("useradd")
        .arg("-m")
        .arg("-s")
        .arg("/bin/bash")
        .arg(username)
        .status()
        .context("Failed to run useradd")?;

    if !status.success() {
        bail!("useradd failed to create user '{}'", username);
    }

    set_system_user_password(username, password)
}

pub fn delete_system_user(username: &str) -> Result<()> {
    validate_username(username)?;

    // Security check: Don't delete system users (UID < 1000)
    let uid = get_uid(username)?;
    if uid < 1000 {
        bail!("Cannot delete protected system user '{}' (UID {} < 1000).", username, uid);
    }

    info!("Deleting system user '{}'...", username);
    let status = Command::new("userdel")
        .arg("-r")
        .arg(username)
        .status()
        .context("Failed to run userdel")?;

    if !status.success() {
        bail!("userdel failed");
    }
    Ok(())
}

pub fn set_system_user_password(username: &str, password: &str) -> Result<()> {
    validate_username(username)?;

    // Security check: Don't modify system users (UID < 1000)
    // We allow if user was just created (which we can't easily track here statelessly),
    // but `create_system_user` calls this.
    // Ideally we should allow it if we just created it.
    // However, `useradd` creates standard users with UID >= 1000.
    // So this check is safe for new users too.
    let uid = get_uid(username)?;
    if uid < 1000 {
        bail!("Cannot modify password for protected system user '{}' (UID {} < 1000).", username, uid);
    }

    info!("Setting password for system user '{}'...", username);
    let mut child = Command::new("chpasswd")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn chpasswd")?;

    {
        let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
        stdin.write_all(format!("{}:{}", username, password).as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        bail!("chpasswd failed");
    }
    Ok(())
}

fn get_home_device() -> Result<String> {
    // df -P /home | tail -1 | awk '{print $1}'
    let output = Command::new("df")
        .arg("-P")
        .arg("/home")
        .output()
        .context("Failed to run df")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 2 {
        bail!("Unexpected df output");
    }
    let device = lines[1].split_whitespace().next().unwrap_or("").to_string();
    if device.is_empty() {
        bail!("Could not parse device from df");
    }
    Ok(device)
}

pub fn set_system_quota(username: &str, quota_gb: u64) -> Result<()> {
    info!("Setting quota for user '{}': {} GB", username, quota_gb);

    // Check if quota command exists
    if Command::new("which").arg("setquota").output().is_err() {
        warn!("quota tool not found. Skipping quota setup.");
        return Ok(());
    }

    let device = match get_home_device() {
        Ok(d) => d,
        Err(e) => {
            warn!("Could not determine filesystem for /home: {}. Skipping quota.", e);
            return Ok(());
        }
    };

    // 1GB = 1048576 KB (blocks)
    let blocks = quota_gb * 1024 * 1024;
    let soft_blocks = blocks;
    let hard_blocks = blocks;

    // setquota -u <user> <block-soft> <block-hard> <inode-soft> <inode-hard> <device>
    let status = Command::new("setquota")
        .arg("-u")
        .arg(username)
        .arg(soft_blocks.to_string())
        .arg(hard_blocks.to_string())
        .arg("0")
        .arg("0")
        .arg(&device)
        .status();

    match status {
        Ok(s) => {
            if !s.success() {
                warn!("setquota failed. Quotas might not be enabled on {}. (Exit code: {:?})", device, s.code());
            } else {
                info!("Quota set successfully.");
            }
        },
        Err(e) => {
             warn!("Failed to execute setquota: {}", e);
        }
    }
    Ok(())
}

pub fn apply_optimizations() -> Result<()> {
    info!("Applying system optimizations for media server performance...");

    let mut sys = System::new();
    sys.refresh_memory();
    let ram_gb = sys.total_memory() / 1024 / 1024 / 1024;

    // Aggressive swappiness reduction for high RAM
    let swappiness = if ram_gb > 16 { 1 } else { 10 };

    let config = format!(r#"# Server Manager Media Server Optimizations
fs.inotify.max_user_watches=524288
vm.swappiness={}
vm.dirty_ratio=10
vm.dirty_background_ratio=5
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
net.core.somaxconn=4096
net.ipv4.tcp_fastopen=3
vm.max_map_count=262144
net.core.rmem_max=4194304
net.core.wmem_max=1048576
"#, swappiness);

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
