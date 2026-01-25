use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use env_logger::Builder;
use cylae::core::hardware::HardwareManager;
use cylae::core::system::SystemManager;
use cylae::core::firewall::FirewallManager;
use cylae::core::docker::DockerManager;
use cylae::services::{ServiceRegistry, plex::PlexService};
use std::sync::Arc;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "cylae")]
#[command(about = "Cylae Server Manager: The Ultimate Optimized Media Server Stack", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install the full stack or specific services
    Install,
    /// Check system status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Info);
    builder.parse_default_env();
    builder.init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install => {
            install().await?;
        }
        Commands::Status => {
            status();
        }
    }

    Ok(())
}

async fn install() -> Result<()> {
    // 1. Root Check
    SystemManager::require_root()?;

    info!("Starting Cylae installation...");

    // 2. Hardware Detection
    let profile = HardwareManager::get_hardware_profile();

    // 3. System Dependencies
    SystemManager::install_dependencies()?;
    SystemManager::apply_optimizations()?;

    // 4. Firewall
    FirewallManager::setup_firewall();

    // 5. Docker
    DockerManager::ensure_docker_installed()?;
    DockerManager::ensure_network("cylae_net")?;

    // 6. Services
    let mut registry = ServiceRegistry::new();
    registry.register(Arc::new(PlexService));

    // For now, just install Plex as a demo
    // In a real scenario, we might ask user or have a config
    if let Some(plex) = registry.get("plex") {
        plex.install(&profile).await?;
    }

    info!("Installation complete!");
    Ok(())
}

fn status() {
    let profile = HardwareManager::get_hardware_profile();
    let gpu = HardwareManager::detect_gpu();
    println!("Hardware Profile: {:?}", profile);
    println!("GPU: {:?}", gpu);

    // Containers
    let containers = DockerManager::get_containers();
    println!("Containers: {:?}", containers);
}
