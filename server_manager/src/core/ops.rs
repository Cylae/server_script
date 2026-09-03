use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use log::info;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Trait abstraction for host system operations.
#[async_trait]
pub trait SystemOps: Send + Sync {
    fn is_root(&self) -> bool;
    fn install_dependencies(&self) -> Result<()>;
    fn create_system_user(&self, username: &str, password: &str) -> Result<()>;
    fn delete_system_user(&self, username: &str) -> Result<()>;
    fn set_system_quota(&self, username: &str, quota_gb: u64) -> Result<()>;
}

/// Trait abstraction for Docker & Docker Compose operations.
#[async_trait]
pub trait DockerOps: Send + Sync {
    fn is_installed(&self) -> bool;
    fn install(&self) -> Result<()>;
    fn compose_up(&self, compose_file: &Path) -> Result<()>;
    fn compose_down(&self, compose_file: &Path) -> Result<()>;
    fn compose_pull(&self, compose_file: &Path) -> Result<()>;
    fn prune_system(&self) -> Result<()>;
}

/// Trait abstraction for Firewall operations.
#[async_trait]
pub trait FirewallBackend: Send + Sync {
    fn is_active(&self) -> Result<bool>;
    fn allow_port(&self, port: u16, proto: &str) -> Result<()>;
    fn deny_port(&self, port: u16, proto: &str) -> Result<()>;
    fn configure_defaults(&self) -> Result<()>;
}

// ---------------- Real Implementations ----------------

pub struct RealSystemOps;

#[async_trait]
impl SystemOps for RealSystemOps {
    fn is_root(&self) -> bool {
        crate::core::system::is_root()
    }

    fn install_dependencies(&self) -> Result<()> {
        crate::core::system::install_dependencies()
    }

    fn create_system_user(&self, username: &str, password: &str) -> Result<()> {
        crate::core::system::create_system_user(username, password)
    }

    fn delete_system_user(&self, username: &str) -> Result<()> {
        crate::core::system::delete_system_user(username)
    }

    fn set_system_quota(&self, username: &str, quota_gb: u64) -> Result<()> {
        crate::core::system::set_system_quota(username, quota_gb)
    }
}

pub struct RealDockerOps;

#[async_trait]
impl DockerOps for RealDockerOps {
    fn is_installed(&self) -> bool {
        crate::core::docker::check_installation()
    }

    fn install(&self) -> Result<()> {
        crate::core::docker::install()
    }

    fn compose_up(&self, compose_file: &Path) -> Result<()> {
        let status = Command::new("docker")
            .args(["compose", "-f", &compose_file.to_string_lossy(), "up", "-d"])
            .status()
            .context("Failed to spawn docker compose up")?;
        if !status.success() {
            bail!("docker compose up failed with status: {}", status);
        }
        Ok(())
    }

    fn compose_down(&self, compose_file: &Path) -> Result<()> {
        let status = Command::new("docker")
            .args(["compose", "-f", &compose_file.to_string_lossy(), "down"])
            .status()
            .context("Failed to spawn docker compose down")?;
        if !status.success() {
            bail!("docker compose down failed with status: {}", status);
        }
        Ok(())
    }

    fn compose_pull(&self, compose_file: &Path) -> Result<()> {
        let status = Command::new("docker")
            .args(["compose", "-f", &compose_file.to_string_lossy(), "pull"])
            .status()
            .context("Failed to spawn docker compose pull")?;
        if !status.success() {
            bail!("docker compose pull failed with status: {}", status);
        }
        Ok(())
    }

    fn prune_system(&self) -> Result<()> {
        let status = Command::new("docker")
            .args(["system", "prune", "-af", "--volumes"])
            .status()
            .context("Failed to spawn docker system prune")?;
        if !status.success() {
            bail!("docker system prune failed with status: {}", status);
        }
        Ok(())
    }
}

pub struct RealFirewallBackend;

#[async_trait]
impl FirewallBackend for RealFirewallBackend {
    fn is_active(&self) -> Result<bool> {
        let status = Command::new("ufw")
            .arg("status")
            .output()
            .context("Failed to check ufw status")?;
        let text = String::from_utf8_lossy(&status.stdout);
        Ok(text.contains("Status: active"))
    }

    fn allow_port(&self, port: u16, proto: &str) -> Result<()> {
        let port_rule = format!("{}/{}", port, proto);
        let status = Command::new("ufw")
            .args(["allow", &port_rule])
            .status()
            .context("Failed to execute ufw allow")?;
        if !status.success() {
            bail!("ufw allow {} failed", port_rule);
        }
        Ok(())
    }

    fn deny_port(&self, port: u16, proto: &str) -> Result<()> {
        let port_rule = format!("{}/{}", port, proto);
        let status = Command::new("ufw")
            .args(["deny", &port_rule])
            .status()
            .context("Failed to execute ufw deny")?;
        if !status.success() {
            bail!("ufw deny {} failed", port_rule);
        }
        Ok(())
    }

    fn configure_defaults(&self) -> Result<()> {
        crate::core::firewall::configure()
    }
}

// ---------------- Mock Implementations for Tests & Dry Run ----------------

#[derive(Default)]
pub struct MockSystemOps {
    pub root: AtomicBool,
    pub calls: Mutex<Vec<String>>,
}

#[async_trait]
impl SystemOps for MockSystemOps {
    fn is_root(&self) -> bool {
        self.root.load(Ordering::Relaxed)
    }

    fn install_dependencies(&self) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push("install_dependencies".to_string()));
        info!("Mock: install_dependencies called");
        Ok(())
    }

    fn create_system_user(&self, username: &str, _password: &str) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("create_system_user:{}", username)));
        info!("Mock: create_system_user called for {}", username);
        Ok(())
    }

    fn delete_system_user(&self, username: &str) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("delete_system_user:{}", username)));
        info!("Mock: delete_system_user called for {}", username);
        Ok(())
    }

    fn set_system_quota(&self, username: &str, quota_gb: u64) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("set_quota:{}:{}", username, quota_gb)));
        info!(
            "Mock: set_system_quota called for {} ({}GB)",
            username, quota_gb
        );
        Ok(())
    }
}

#[derive(Default)]
pub struct MockDockerOps {
    pub installed: AtomicBool,
    pub calls: Mutex<Vec<String>>,
}

#[async_trait]
impl DockerOps for MockDockerOps {
    fn is_installed(&self) -> bool {
        self.installed.load(Ordering::Relaxed)
    }

    fn install(&self) -> Result<()> {
        self.installed.store(true, Ordering::Relaxed);
        let _ = self.calls.lock().map(|mut c| c.push("install".to_string()));
        Ok(())
    }

    fn compose_up(&self, compose_file: &Path) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("compose_up:{}", compose_file.display())));
        Ok(())
    }

    fn compose_down(&self, compose_file: &Path) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("compose_down:{}", compose_file.display())));
        Ok(())
    }

    fn compose_pull(&self, compose_file: &Path) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("compose_pull:{}", compose_file.display())));
        Ok(())
    }

    fn prune_system(&self) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push("prune_system".to_string()));
        Ok(())
    }
}

#[derive(Default)]
pub struct MockFirewallBackend {
    pub active: AtomicBool,
    pub allowed_ports: Mutex<Vec<u16>>,
}

#[async_trait]
impl FirewallBackend for MockFirewallBackend {
    fn is_active(&self) -> Result<bool> {
        Ok(self.active.load(Ordering::Relaxed))
    }

    fn allow_port(&self, port: u16, _proto: &str) -> Result<()> {
        let _ = self.allowed_ports.lock().map(|mut p| p.push(port));
        Ok(())
    }

    fn deny_port(&self, port: u16, _proto: &str) -> Result<()> {
        let _ = self
            .allowed_ports
            .lock()
            .map(|mut p| p.retain(|&x| x != port));
        Ok(())
    }

    fn configure_defaults(&self) -> Result<()> {
        let _ = self.allowed_ports.lock().map(|mut p| {
            p.extend_from_slice(&[22, 80, 443, 8099]);
        });
        Ok(())
    }
}
