use bollard::Docker;
use bollard::query_parameters::ListContainersOptions;
use std::collections::HashSet;
use anyhow::Result;
use tokio::sync::Mutex;
use log::{info, warn, error};

pub struct DockerManager {
    docker: Docker,
    network_lock: Mutex<()>,
}

impl DockerManager {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self {
            docker,
            network_lock: Mutex::new(()),
        })
    }

    pub async fn is_installed(&self, container_name: &str) -> Result<bool> {
        match self.docker.inspect_container(container_name, None).await {
            Ok(_) => Ok(true),
            Err(bollard::errors::Error::DockerResponseServerError { status_code: 404, .. }) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_all_container_names(&self) -> Result<HashSet<String>> {
        let options = ListContainersOptions {
            all: true,
            ..Default::default()
        };
        let containers = self.docker.list_containers(Some(options)).await?;
        let mut names = HashSet::new();
        for container in containers {
            if let Some(container_names) = container.names {
                for name in container_names {
                    names.insert(name.trim_start_matches('/').to_string());
                }
            }
        }
        Ok(names)
    }

    pub async fn ensure_network(&self, network_name: &str) -> Result<()> {
        let _lock = self.network_lock.lock().await;

        match self.docker.inspect_network(network_name, None).await {
            Ok(_) => Ok(()),
            Err(bollard::errors::Error::DockerResponseServerError { status_code: 404, .. }) => {
                info!("Creating Docker network: {}", network_name);

                // Stubbed for now
                warn!("TODO: Network creation implementation deferred.");
                Ok(())
            },
            Err(e) => Err(e.into()),
        }
    }

    pub async fn wait_for_health(&self, container_name: &str, retries: u32, delay: u64) -> Result<bool> {
        info!("Waiting for '{}' to become healthy...", container_name);

        for _ in 0..retries {
            match self.docker.inspect_container(container_name, None).await {
                Ok(container) => {
                    if let Some(state) = container.state {
                        if state.status == Some(bollard::models::ContainerStateStatusEnum::RUNNING) {
                            if let Some(health) = state.health {
                                if let Some(status) = health.status {
                                    if status == bollard::models::HealthStatusEnum::HEALTHY {
                                        info!("Container '{}' is healthy.", container_name);
                                        return Ok(true);
                                    }
                                }
                            } else {
                                // No healthcheck defined
                                info!("Container '{}' is running (no healthcheck defined).", container_name);
                                return Ok(true);
                            }
                        }
                    }
                },
                Err(e) => warn!("Error inspecting container '{}': {}", container_name, e),
            }
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        }

        warn!("Timeout waiting for '{}' to be healthy.", container_name);
        Ok(false)
    }

    pub async fn stop_and_remove(&self, container_name: &str) -> Result<()> {
        match self.docker.inspect_container(container_name, None).await {
            Ok(_) => {
                info!("Stopping container '{}'...", container_name);
                let _ = self.docker.stop_container(container_name, None).await;
                info!("Removing container '{}'...", container_name);
                self.docker.remove_container(container_name, None).await?;
            },
            Err(bollard::errors::Error::DockerResponseServerError { status_code: 404, .. }) => {
                // Already gone
            },
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}
