mod core;
mod services;

use log::{info, error, warn};
use services::registry::ServiceRegistry;
use services::media::plex::PlexService;
use crate::services::base::Service;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Cylae Manager (Rust Edition)");

    // Initialize Registry
    let registry = ServiceRegistry::new();

    // Register Plex
    info!("Registering Services...");
    registry.register(Box::new(PlexService)).await;

    // Verify Plex Configuration Generation
    if let Some(plex) = registry.get("plex").await {
        info!("Found Service: {}", plex.pretty_name());

        let compose_config = plex.generate_compose();
        info!("Generated Compose Config for Plex:");
        println!("{}", serde_json::to_string_pretty(&compose_config).unwrap());

        // Check GDHD impact
        let profile = core::hardware::HardwareManager::get_hardware_profile();
        info!("Current Hardware Profile: {:?}", profile);
    } else {
        error!("Plex service not found in registry!");
    }

    // Verify Docker Manager Connectivity
    info!("Verifying Docker Manager...");
    match core::docker::DockerManager::new() {
        Ok(docker) => {
            match docker.get_all_container_names().await {
                Ok(names) => {
                    info!("Docker Connectivity Verified. Found {} containers.", names.len());
                    for name in names {
                        info!(" - {}", name);
                    }
                },
                Err(e) => {
                    // This is expected in sandbox without docker daemon
                    warn!("Docker API Connection Failed (Expected in build env): {}", e);
                }
            }
        },
        Err(e) => {
             warn!("Failed to initialize Docker Manager (Expected in build env): {}", e);
        }
    }
}
