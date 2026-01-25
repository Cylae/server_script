use crate::core::system::SystemManager;
use log::{info, debug};
use which::which;
use std::collections::HashSet;
use anyhow::{Result};

pub struct DockerManager;

impl DockerManager {
    pub fn ensure_docker_installed() -> Result<()> {
        if which("docker").is_ok() {
            return Ok(());
        }

        info!("Docker not found. Installing...");

        // Curl the script
        SystemManager::run_command("curl", &["-fsSL", "https://get.docker.com", "-o", "get-docker.sh"])?;

        // Run the script
        SystemManager::run_command("sh", &["get-docker.sh"])?;

        // Cleanup
        let _ = std::fs::remove_file("get-docker.sh");

        if which("docker").is_err() {
            anyhow::bail!("Docker installation failed.");
        }

        info!("Docker installed successfully.");
        Ok(())
    }

    pub fn get_containers() -> HashSet<String> {
        let mut containers = HashSet::new();
        // docker ps -a --format '{{.Names}}'
        match SystemManager::run_command("docker", &["ps", "-a", "--format", "{{.Names}}"]) {
            Ok(output) => {
                for line in output.lines() {
                    let name = line.trim();
                    if !name.is_empty() {
                        containers.insert(name.to_string());
                    }
                }
            },
            Err(e) => {
                // If docker is not running or installed, this might fail.
                debug!("Failed to list containers (Docker might not be running): {}", e);
            }
        }
        containers
    }

    pub fn ensure_network(network_name: &str) -> Result<()> {
         let output = SystemManager::run_command("docker", &["network", "ls", "--format", "{{.Name}}"])?;
         if output.lines().any(|line| line.trim() == network_name) {
             return Ok(());
         }

         info!("Creating Docker network: {}", network_name);
         SystemManager::run_command("docker", &["network", "create", "--driver", "bridge", network_name])?;
         Ok(())
    }
}
