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
    /// Checks whether a systemd service unit is currently active.
    fn is_service_active(&self, service_name: &str) -> bool;
    /// Stops and disables a systemd service unit.
    fn stop_system_service(&self, service_name: &str) -> Result<()>;
    /// Vacuums systemd journal logs to the specified size (e.g., "100M").
    fn vacuum_journal(&self, retention: &str) -> Result<()>;
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
    /// Runs `docker compose up -d --remove-orphans` in the current directory.
    fn compose_up_remove_orphans(&self) -> Result<()>;
    /// Checks whether the Docker daemon is responding.
    fn is_daemon_running(&self) -> bool;
    /// Pulls Docker images for the compose stack in the current directory.
    fn compose_pull_current(&self) -> Result<()>;
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

    fn is_service_active(&self, service_name: &str) -> bool {
        Command::new("/usr/bin/systemctl")
            .args(["is-active", "--quiet", service_name])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn stop_system_service(&self, service_name: &str) -> Result<()> {
        let _ = Command::new("/usr/bin/systemctl")
            .args(["stop", service_name])
            .status();
        let _ = Command::new("/usr/bin/systemctl")
            .args(["disable", service_name])
            .status();
        Ok(())
    }

    fn vacuum_journal(&self, retention: &str) -> Result<()> {
        let arg = format!("--vacuum-size={}", retention);
        let path = if Path::new("/usr/bin/journalctl").exists() {
            "/usr/bin/journalctl"
        } else {
            "journalctl"
        };
        let status = Command::new(path)
            .arg(&arg)
            .status()
            .context("Failed to vacuum journal")?;
        if !status.success() {
            bail!("journalctl --vacuum-size failed with status: {}", status);
        }
        Ok(())
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
        let status = Command::new("/usr/bin/docker")
            .args(["compose", "-f", &compose_file.to_string_lossy(), "up", "-d"])
            .status()
            .context("Failed to spawn docker compose up")?;
        if !status.success() {
            bail!("docker compose up failed with status: {}", status);
        }
        Ok(())
    }

    fn compose_down(&self, compose_file: &Path) -> Result<()> {
        let status = Command::new("/usr/bin/docker")
            .args(["compose", "-f", &compose_file.to_string_lossy(), "down"])
            .status()
            .context("Failed to spawn docker compose down")?;
        if !status.success() {
            bail!("docker compose down failed with status: {}", status);
        }
        Ok(())
    }

    fn compose_pull(&self, compose_file: &Path) -> Result<()> {
        let status = Command::new("/usr/bin/docker")
            .args(["compose", "-f", &compose_file.to_string_lossy(), "pull"])
            .status()
            .context("Failed to spawn docker compose pull")?;
        if !status.success() {
            bail!("docker compose pull failed with status: {}", status);
        }
        Ok(())
    }

    fn prune_system(&self) -> Result<()> {
        // SECURITY (F05, REQ-OPS-004): Previous implementation used `-af --volumes`
        // which removes ALL unused images and ALL anonymous volumes, including
        // those belonging to other workloads on the same host. Using `-f`
        // with a project label filter limits cleanup strictly to resources
        // owned by server_manager, preserving the non-destructive host guarantee.
        let status = Command::new("/usr/bin/docker")
            .args([
                "system",
                "prune",
                "-f",
                "--filter",
                "label=com.docker.compose.project=server_manager",
            ])
            .status()
            .context("Failed to spawn docker system prune")?;
        if !status.success() {
            bail!("docker system prune failed with status: {}", status);
        }
        Ok(())
    }

    fn compose_up_remove_orphans(&self) -> Result<()> {
        let status = Command::new("/usr/bin/docker")
            .args(["compose", "up", "-d", "--remove-orphans"])
            .status()
            .context("Failed to spawn docker compose up --remove-orphans")?;
        if !status.success() {
            bail!(
                "docker compose up --remove-orphans failed with status: {}",
                status
            );
        }
        Ok(())
    }

    fn is_daemon_running(&self) -> bool {
        Command::new("/usr/bin/docker")
            .arg("ps")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn compose_pull_current(&self) -> Result<()> {
        let status = Command::new("/usr/bin/docker")
            .args(["compose", "pull"])
            .status()
            .context("Failed to spawn docker compose pull")?;
        if !status.success() {
            bail!("docker compose pull failed with status: {}", status);
        }
        Ok(())
    }
}

pub struct RealFirewallBackend;

#[async_trait]
impl FirewallBackend for RealFirewallBackend {
    fn is_active(&self) -> Result<bool> {
        let status = Command::new("/usr/sbin/ufw")
            .arg("status")
            .output()
            .context("Failed to check ufw status")?;
        let text = String::from_utf8_lossy(&status.stdout);
        Ok(text.contains("Status: active"))
    }

    fn allow_port(&self, port: u16, proto: &str) -> Result<()> {
        let port_rule = format!("{}/{}", port, proto);
        let status = Command::new("/usr/sbin/ufw")
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
        let status = Command::new("/usr/sbin/ufw")
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

    fn is_service_active(&self, service_name: &str) -> bool {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("is_service_active:{}", service_name)));
        false
    }

    fn stop_system_service(&self, service_name: &str) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("stop_system_service:{}", service_name)));
        Ok(())
    }

    fn vacuum_journal(&self, retention: &str) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push(format!("vacuum_journal:{}", retention)));
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

    fn compose_up_remove_orphans(&self) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push("compose_up_remove_orphans".to_string()));
        Ok(())
    }

    fn is_daemon_running(&self) -> bool {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push("is_daemon_running".to_string()));
        true
    }

    fn compose_pull_current(&self) -> Result<()> {
        let _ = self
            .calls
            .lock()
            .map(|mut c| c.push("compose_pull_current".to_string()));
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
