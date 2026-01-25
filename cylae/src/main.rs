use clap::{Parser, Subcommand};
use env_logger::Env;
use log::info;
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
    /// Install system dependencies and optimize environment
    Install,
    /// Display hardware information
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Install => {
            info!("Starting Cylae Installation...");
            let sys = SystemManager::new();
            let hw = HardwareManager::new();
            let fw = FirewallManager::new();
            let docker = DockerManager::new();

            sys.check_root()?;
            sys.install_system_deps()?;
            fw.configure_firewall()?;
            docker.check_and_install_docker()?;
            sys.optimize_system()?;

            info!("Detecting Hardware for final configuration...");
            let hw_info = hw.detect_hardware()?;
            info!("Installation Complete! Profile: {:?}", hw_info.profile);
        }
        Commands::Info => {
            let hw = HardwareManager::new();
            let hw_info = hw.detect_hardware()?;
            println!("{:#?}", hw_info);
        }
    }

    Ok(())
}
