use clap::{Parser, Subcommand};
use env_logger::Env;
use log::{info, error};
use anyhow::Result;

use cylae::core::hardware::HardwareManager;
use cylae::core::system::SystemManager;
use cylae::core::firewall::FirewallManager;
use cylae::core::docker::DockerManager;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install system dependencies, docker, and apply optimizations
    Install,
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Install => {
            info!("Starting Cylae Installation...");

            if let Err(e) = SystemManager::check_root() {
                error!("{}", e);
                std::process::exit(1);
            }

            // Hardware Detection
            let profile = HardwareManager::get_hardware_profile();
            info!("Hardware Profile: {:?}", profile);
            let gpu = HardwareManager::detect_gpu();
            info!("GPU Type: {:?}", gpu);

            // Install Deps
            if let Err(e) = SystemManager::install_system_deps() {
                error!("Failed to install system dependencies: {}", e);
                std::process::exit(1);
            }

            // Firewall
            if let Err(e) = FirewallManager::configure_firewall() {
                error!("Failed to configure firewall: {}", e);
            }

            // Docker
            if let Err(e) = DockerManager::check_and_install_docker() {
                 error!("Failed to install Docker: {}", e);
                 std::process::exit(1);
            }

            // Optimize
            if let Err(e) = SystemManager::optimize_system() {
                 error!("Failed to apply optimizations: {}", e);
            }

            info!("Installation Complete! 🚀");
        }
    }

    Ok(())
}
