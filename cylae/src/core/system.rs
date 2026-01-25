use std::process::{Command, Stdio};
use log::{info, error, debug};
use nix::unistd::Uid;
use anyhow::{Result, Context};

pub struct SystemManager;

impl SystemManager {
    pub fn require_root() -> Result<()> {
        if !Uid::effective().is_root() {
            anyhow::bail!("This application requires root privileges. Please run with sudo.");
        }
        Ok(())
    }

    pub fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
        debug!("Executing: {} {:?}", cmd, args);
        let output = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute command: {} {:?}", cmd, args))?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             // Don't error immediately if it's just non-zero, let caller handle?
             // But existing code raised exceptions.
             error!("Command failed: {}", stderr);
             anyhow::bail!("Command failed: {} {:?} - {}", cmd, args, stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    pub fn install_dependencies() -> Result<()> {
        info!("Installing system dependencies...");
        // Check if apt exists
        if which::which("apt-get").is_err() {
            anyhow::bail!("apt-get not found. Only Debian/Ubuntu systems are supported.");
        }

        let deps = vec![
            "curl", "wget", "git", "jq", "software-properties-common", "apt-transport-https", "ca-certificates", "gnupg",
            "ufw", "htop", "iotop", "ncdu", "net-tools", "dnsutils", "tzdata"
        ];

        Self::run_command("apt-get", &["update"])?;

        let mut install_args = vec!["install", "-y"];
        install_args.extend(deps);

        // Use DEBIAN_FRONTEND=noninteractive to avoid prompts
        debug!("Running apt-get install...");
        let status = Command::new("apt-get")
            .args(&install_args)
            .env("DEBIAN_FRONTEND", "noninteractive")
            .status()
            .context("Failed to run apt-get install")?;

        if !status.success() {
            anyhow::bail!("apt-get install failed");
        }

        info!("System dependencies installed.");
        Ok(())
    }

    pub fn apply_optimizations() -> Result<()> {
        info!("Applying system optimizations...");

        // Sysctl tweaks
        let tweaks = vec![
            ("fs.inotify.max_user_watches", "524288"),
            ("vm.swappiness", "10"),
            ("net.core.default_qdisc", "fq"),
            ("net.ipv4.tcp_congestion_control", "bbr"),
        ];

        for (key, value) in tweaks {
            // Apply immediately
            if let Err(e) = Self::run_command("sysctl", &["-w", &format!("{}={}", key, value)]) {
                error!("Failed to apply sysctl {}: {}", key, e);
            }
        }

        info!("Optimizations applied.");
        Ok(())
    }

    pub fn get_uid_gid() -> (u32, u32) {
         // Return current user/group or 1000:1000 fallback
         if let Ok(uid_str) = std::env::var("SUDO_UID") {
             if let Ok(gid_str) = std::env::var("SUDO_GID") {
                 if let (Ok(uid), Ok(gid)) = (uid_str.parse::<u32>(), gid_str.parse::<u32>()) {
                     return (uid, gid);
                 }
             }
         }
         (1000, 1000)
    }

    pub fn get_timezone() -> String {
        if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
             return content.trim().to_string();
        }
        if let Ok(path) = std::fs::read_link("/etc/localtime") {
             if let Some(str_path) = path.to_str() {
                 if let Some(pos) = str_path.find("zoneinfo/") {
                     return str_path[pos + 9..].to_string();
                 }
             }
        }
        "Etc/UTC".to_string()
    }
}
