use crate::core::system::SystemManager;
use log::{info, warn, error, debug};

pub struct FirewallManager;

impl FirewallManager {
    pub fn is_available() -> bool {
        SystemManager::check_command("ufw")
    }

    pub fn allow_port(port: &str, comment: &str) {
        if !Self::is_available() {
            debug!("Firewall (ufw) not detected. Skipping rule.");
            return;
        }

        let mut args = vec!["allow", port];
        if !comment.is_empty() {
            args.push("comment");
            args.push(comment);
        }

        match SystemManager::run_command("ufw", &args, true) {
            Ok(_) => info!("Firewall: Allowed port {}", port),
            Err(e) => error!("Failed to allow port {}: {}", port, e),
        }
    }

    pub fn deny_port(port: &str) {
        if !Self::is_available() {
            return;
        }

        // ufw delete allow <port>
        match SystemManager::run_command("ufw", &["delete", "allow", port], true) {
            Ok(_) => info!("Firewall: Removed rule for port {}", port),
            Err(e) => warn!("Failed to remove firewall rule for {}: {}", port, e),
        }
    }

    pub fn enable() {
        if !Self::is_available() {
            return;
        }

        // Check if active
        if let Ok(output) = SystemManager::run_command("ufw", &["status"], false) {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Status: active") {
                return;
            }
        }

        info!("Enabling Firewall (ufw)...");
        if let Err(e) = SystemManager::run_command("ufw", &["--force", "enable"], true) {
            error!("Failed to enable firewall: {}", e);
        }
    }

    pub fn ensure_basic_rules() {
        if !Self::is_available() {
            return;
        }

        info!("Configuring basic firewall rules...");
        Self::allow_port("ssh", "SSH Access");
        Self::allow_port("22", "SSH Fallback");
    }
}
