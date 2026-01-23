use async_trait::async_trait;
use serde_yaml::Value;
use std::collections::HashMap;
use anyhow::Result;

#[async_trait]
pub trait Service {
    fn name(&self) -> &'static str;
    fn pretty_name(&self) -> &'static str;

    /// Checks if the service is currently installed (running or stopped).
    async fn is_installed(&self) -> Result<bool>;

    /// Optional hook to interactively prompt the user for configuration.
    async fn configure(&self) -> Result<()> {
        Ok(())
    }

    /// Optional hook to return a summary string after installation.
    fn get_install_summary(&self) -> Option<String> {
        None
    }

    /// Optional hook to return the primary URL or Subdomain for the service.
    fn get_url(&self) -> Option<String> {
        None
    }

    /// Returns a list of ports/protocols to open in the firewall.
    fn get_ports(&self) -> Vec<String> {
        vec![]
    }

    /// Installs or updates the service via Docker Compose.
    async fn install(&self) -> Result<()>;

    /// Polls the container status to check for health.
    async fn wait_for_health(&self, retries: u32, delay: u64) -> Result<bool>;

    /// Stops and removes the service container.
    async fn remove(&self) -> Result<()>;

    /// Generates the Docker Compose dictionary.
    fn generate_compose(&self) -> Value;
}
