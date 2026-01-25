use crate::core::system::SystemManager;
use log::{info, error, debug};
use which::which;

pub struct FirewallManager;

impl FirewallManager {
    pub fn is_available() -> bool {
        which("ufw").is_ok()
    }

    pub fn allow_port(port: &str, comment: &str) {
        if !Self::is_available() {
            debug!("Firewall (ufw) not detected. Skipping allow {}.", port);
            return;
        }

        let mut args = vec!["allow", port];
        let comment_str = format!("Cylae: {}", comment);
        if !comment.is_empty() {
            args.push("comment");
            args.push(&comment_str);
        }

        if let Err(e) = SystemManager::run_command("ufw", &args) {
            error!("Failed to allow port {}: {}", port, e);
        } else {
            info!("Firewall: Allowed port {}", port);
        }
    }

    pub fn setup_firewall() {
        if !Self::is_available() {
            return;
        }

        info!("Configuring firewall...");
        // Ensure SSH is allowed before enabling!
        Self::allow_port("22/tcp", "SSH Access");
        Self::allow_port("ssh", "SSH Access"); // Fallback

        // Default deny incoming
        if let Err(e) = SystemManager::run_command("ufw", &["default", "deny", "incoming"]) {
             error!("Failed to set default deny incoming: {}", e);
        }

        // Force enable
        if let Err(e) = SystemManager::run_command("ufw", &["--force", "enable"]) {
            error!("Failed to enable firewall: {}", e);
        } else {
             info!("Firewall enabled.");
        }
    }
}
