use std::path::Path;
use std::process::Command;
use log::{warn, debug};
use nix::unistd::{Uid, Gid};
use anyhow::Result;
use std::env;

use crate::core::hardware::{HardwareManager, HardwareProfile};

pub struct SystemManager;

impl SystemManager {
    /// Enforces execution with elevated privileges (root).
    pub fn check_root() -> Result<()> {
        if !Uid::effective().is_root() {
            anyhow::bail!("Insufficient privileges. This operation requires root access.");
        }
        Ok(())
    }

    /// Determines the hardware profile using the GDHD heuristics via HardwareManager.
    pub fn get_hardware_profile() -> &'static HardwareProfile {
        HardwareManager::get_hardware_profile()
    }

    /// Returns the recommended concurrency limit via HardwareManager.
    pub fn get_concurrency_limit() -> usize {
        HardwareManager::get_concurrency_limit()
    }

    /// Returns the UID and GID of the real user (even if running as root via sudo).
    pub fn get_uid_gid() -> (u32, u32) {
        // If running under sudo, use the original user's ID
        if let (Ok(sudo_uid), Ok(sudo_gid)) = (env::var("SUDO_UID"), env::var("SUDO_GID")) {
             if let (Ok(uid), Ok(gid)) = (sudo_uid.parse::<u32>(), sudo_gid.parse::<u32>()) {
                 return (uid, gid);
             }
        }

        // Fallback to current user (which might be root if check_root passed)
        (Uid::current().as_raw(), Gid::current().as_raw())
    }

    /// Retrieves the system timezone.
    pub fn get_timezone() -> String {
        // Check /etc/timezone first
        if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
            return content.trim().to_string();
        }

        // Check symlink at /etc/localtime
        if let Ok(path) = std::fs::read_link("/etc/localtime") {
            if let Some(str_path) = path.to_str() {
                return str_path.replace("/usr/share/zoneinfo/", "");
            }
        }

        warn!("Could not determine timezone. Defaulting to UTC.");
        "UTC".to_string()
    }

    /// Checks if a system command is available.
    pub fn check_command(command: &str) -> bool {
        which::which(command).is_ok()
    }

    /// Runs a shell command safely.
    /// Note: Rust `Command` does not use shell by default, satisfying security requirements.
    pub fn run_command(command: &str, args: &[&str], check: bool) -> Result<std::process::Output> {
        debug!("Executing: {} {:?}", command, args);
        let output = Command::new(command)
            .args(args)
            .output()?;

        if check && !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {} {:?}\nError: {}", command, args, stderr);
        }

        Ok(output)
    }
}
