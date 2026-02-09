use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{error, info};
use std::fs;
use std::io::{self, Write};
use std::process::Command;

use crate::build_compose_structure;
use crate::core::{config, docker, firewall, hardware, secrets, system, users};
use crate::services;

#[derive(Parser)]
#[command(name = "server_manager")]
#[command(about = "Next-Gen Media Server Orchestrator", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Full installation (Idempotent)
    Install,
    /// Show system status
    Status,
    /// Generate docker-compose.yml only
    Generate,
    /// Enable a service
    Enable { service: String },
    /// Disable a service
    Disable { service: String },
    /// Start the Web Administration Interface
    Web {
        #[arg(long, default_value_t = 8099)]
        port: u16,
    },
    /// Manage Users
    User {
        #[command(subcommand)]
        action: UserCommands,
    },
}

#[derive(Subcommand)]
pub enum UserCommands {
    /// Add a new user
    Add {
        username: String,
        #[arg(long, default_value = "Observer")]
        role: String, // "Admin" or "Observer"
        #[arg(long)]
        quota: Option<u64>,
    },
    /// Delete a user
    Delete { username: String },
    /// List users
    List,
    /// Change user password
    Passwd { username: String },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install => run_install().await?,
        Commands::Status => run_status(),
        Commands::Generate => run_generate().await?,
        Commands::Enable { service } => run_toggle_service(service, true).await?,
        Commands::Disable { service } => run_toggle_service(service, false).await?,
        Commands::Web { port } => crate::interface::web::start_server(port).await?,
        Commands::User { action } => run_user_management(action)?,
    }

    Ok(())
}

fn run_user_management(action: UserCommands) -> Result<()> {
    let mut user_manager = users::UserManager::load()?;

    match action {
        UserCommands::Add {
            username,
            role,
            quota,
        } => {
            let role_enum = match role.to_lowercase().as_str() {
                "admin" => users::Role::Admin,
                "observer" => users::Role::Observer,
                _ => return Err(anyhow::anyhow!("Invalid role. Use 'Admin' or 'Observer'")),
            };

            print!("Enter password for {}: ", username);
            io::stdout().flush()?;
            let mut password = String::new();
            io::stdin().read_line(&mut password)?;
            let password = password.trim();

            if password.is_empty() {
                return Err(anyhow::anyhow!("Password cannot be empty"));
            }

            user_manager.add_user(&username, password, role_enum, quota)?;
            info!("User '{}' added successfully.", username);
        }
        UserCommands::Delete { username } => {
            user_manager.delete_user(&username)?;
            info!("User '{}' deleted successfully.", username);
        }
        UserCommands::List => {
            println!("{:<20} | {:<15}", "Username", "Role");
            println!("{:<20} | {:<15}", "--------", "----");
            for user in user_manager.list_users() {
                println!("{:<20} | {:?}", user.username, user.role);
            }
        }
        UserCommands::Passwd { username } => {
            // Check existence first
            if user_manager.get_user(&username).is_none() {
                return Err(anyhow::anyhow!("User not found"));
            }
            print!("Enter new password for {}: ", username);
            io::stdout().flush()?;
            let mut password = String::new();
            io::stdin().read_line(&mut password)?;
            let password = password.trim();

            if password.is_empty() {
                return Err(anyhow::anyhow!("Password cannot be empty"));
            }

            user_manager.update_password(&username, password)?;
            info!("Password for '{}' updated successfully.", username);
        }
    }
    Ok(())
}

async fn run_toggle_service(service_name: String, enable: bool) -> Result<()> {
    // 1. Load Config
    // We assume we are in the install directory or user provides it.
    // For safety, let's try to switch to /opt/server_manager if config not found locally
    if !std::path::Path::new("config.yaml").exists()
        && std::path::Path::new("/opt/server_manager/config.yaml").exists()
    {
        std::env::set_current_dir("/opt/server_manager")?;
    }

    let mut config = config::Config::load()?;

    // Check if service exists
    let services = services::get_all_services();
    if !services.iter().any(|s| s.name() == service_name) {
        error!("Service '{}' not found!", service_name);
        return Ok(());
    }

    if enable {
        config.enable_service(&service_name);
    } else {
        config.disable_service(&service_name);
    }

    config.save()?;

    info!("Configuration updated. Re-running generation...");

    // 2. Re-run generation logic (similar to run_generate/run_install subset)
    // We need secrets for this
    let secrets = secrets::Secrets::load_or_create()?;
    let hw = hardware::HardwareInfo::detect();

    // Only configure/generate, don't necessarily fully install dependencies again
    // But we should probably trigger docker compose up to apply changes
    configure_services(&hw, &secrets, &config)?;
    initialize_services(&hw, &secrets, &config)?;
    generate_compose(&hw, &secrets, &config).await?;

    info!("Applying changes via Docker Compose...");
    let status = Command::new("docker")
        .args(["compose", "up", "-d", "--remove-orphans"])
        .status()
        .context("Failed to run docker compose up")?;

    if status.success() {
        info!(
            "Service '{}' {} successfully!",
            service_name,
            if enable { "enabled" } else { "disabled" }
        );
    } else {
        error!("Failed to apply changes via Docker Compose.");
    }

    Ok(())
}

async fn run_install() -> Result<()> {
    info!("Starting Server Manager Installation...");

    // 1. Root Check
    system::check_root()?;

    // 1.1 Create Install Directory
    let install_dir = std::path::Path::new("/opt/server_manager");
    if !install_dir.exists() {
        info!("Creating installation directory at /opt/server_manager...");
        fs::create_dir_all(install_dir).context("Failed to create /opt/server_manager")?;
    }
    std::env::set_current_dir(install_dir).context("Failed to chdir to /opt/server_manager")?;

    // 1.2 Load Secrets & Config
    let secrets = secrets::Secrets::load_or_create()?;
    let config = config::Config::load()?;

    // 2. Hardware Detection
    let hw = hardware::HardwareInfo::detect();

    // 3. System Dependencies & Optimization
    system::install_dependencies()?;
    system::apply_optimizations(&hw)?;

    // 4. Firewall
    firewall::configure()?;

    // 5. Docker
    docker::install()?;

    // 6. Initialize Services
    configure_services(&hw, &secrets, &config)?;
    initialize_services(&hw, &secrets, &config)?;

    // 7. Generate Compose
    generate_compose(&hw, &secrets, &config).await?;

    // 8. Launch
    info!("Launching Services via Docker Compose...");
    let status = Command::new("docker")
        .args(["compose", "up", "-d", "--remove-orphans"])
        .status()
        .context("Failed to run docker compose up")?;

    if status.success() {
        info!("Server Manager Stack Deployed Successfully! 🚀");
        print_deployment_summary(&secrets);
    } else {
        error!("Docker Compose failed.");
    }

    Ok(())
}

fn print_deployment_summary(secrets: &secrets::Secrets) {
    let mut summary = String::new();
    summary.push_str("\n=================================================================================\n");
    summary.push_str("                           DEPLOYMENT SUMMARY 🚀\n");
    summary.push_str("=================================================================================\n");
    summary.push_str(&format!("{:<15} | {:<25} | {:<15} | Password / Info\n", "Service", "URL", "User"));
    summary.push_str(&format!("{:<15} | {:<25} | {:<15} | ---------------\n", "-------", "---", "----"));

    let mut append_row = |service: &str, url: &str, user: &str, pass: &str| {
        summary.push_str(&format!("{:<15} | {:<25} | {:<15} | {}\n", service, url, user, pass));
    };

    // Helper to format Option<String>
    let pass = |opt: &Option<String>| opt.clone().unwrap_or_else(|| "ERROR".to_string());

    append_row("Nginx Proxy", "http://<IP>:81", "admin@example.com", "changeme");
    append_row("Portainer", "http://<IP>:9000", "admin", "Set on first login");
    append_row("Nextcloud", "https://<IP>:4443", "admin", &pass(&secrets.nextcloud_admin_password));
    append_row("Vaultwarden", "http://<IP>:8001/admin", "(Token)", &pass(&secrets.vaultwarden_admin_token));
    append_row("Gitea", "http://<IP>:3000", "Register", "DB pre-configured");
    append_row("GLPI", "http://<IP>:8088", "glpi", "glpi (Change immediately!)");
    append_row("Yourls", "http://<IP>:8003/admin", "admin", &pass(&secrets.yourls_admin_password));
    append_row("Roundcube", "http://<IP>:8090", "-", "Login with Mail creds");
    append_row("MailServer", "PORTS: 25, 143...", "CLI", "docker exec -ti mailserver setup ...");
    append_row("Plex", "http://<IP>:32400/web", "-", "Follow Web Setup");
    append_row("ArrStack", "http://<IP>:8989 (Sonarr)", "-", "No auth by default");

    summary.push_str("=================================================================================\n\n");
    summary.push_str("NOTE: Replace <IP> with your server's IP address.");

    println!("{}", summary);

    // Save to /root/credentials.txt if running as root
    if nix::unistd::Uid::effective().is_root() {
        if let Err(e) = std::fs::write("/root/credentials.txt", &summary) {
            error!("Failed to save credentials to /root/credentials.txt: {}", e);
        } else {
            info!("Credentials saved to /root/credentials.txt");
        }
    }
}

fn run_status() {
    let hw = hardware::HardwareInfo::detect();
    println!("=== System Status ===");
    println!("RAM: {} GB", hw.ram_gb);
    println!("Swap: {} GB", hw.swap_gb);
    println!("Disk: {} GB", hw.disk_gb);
    println!("Cores: {}", hw.cpu_cores);
    println!("Profile: {:?}", hw.profile);
    println!("Nvidia GPU: {}", hw.has_nvidia);
    println!("Intel QuickSync: {}", hw.has_intel_quicksync);

    println!("\n=== Docker Status ===");
    if let Ok(true) = Command::new("docker")
        .arg("ps")
        .status()
        .map(|s| s.success())
    {
        println!("Docker is running.");
    } else {
        println!("Docker is NOT running.");
    }
}

async fn run_generate() -> Result<()> {
    let hw = hardware::HardwareInfo::detect();
    // For generate, we might not be in /opt/server_manager, but let's try to load secrets from CWD.
    // We propagate the error because generating a compose file with empty passwords is bad.
    let secrets =
        secrets::Secrets::load_or_create().context("Failed to load or create secrets.yaml")?;
    let config = config::Config::load()?;
    configure_services(&hw, &secrets, &config)?;
    generate_compose(&hw, &secrets, &config).await
}

fn configure_services(
    hw: &hardware::HardwareInfo,
    secrets: &secrets::Secrets,
    config: &config::Config,
) -> Result<()> {
    info!("Configuring services (generating config files)...");
    let services = services::get_all_services();
    for service in services {
        if !config.is_enabled(service.name()) {
            continue;
        }
        service
            .configure(hw, secrets)
            .with_context(|| format!("Failed to configure service: {}", service.name()))?;
    }
    Ok(())
}

fn initialize_services(
    hw: &hardware::HardwareInfo,
    secrets: &secrets::Secrets,
    config: &config::Config,
) -> Result<()> {
    info!("Initializing services (system setup)...");
    let services = services::get_all_services();
    for service in services {
        if !config.is_enabled(service.name()) {
            continue;
        }
        service
            .initialize(hw, secrets)
            .with_context(|| format!("Failed to initialize service: {}", service.name()))?;
    }
    Ok(())
}

async fn generate_compose(
    hw: &hardware::HardwareInfo,
    secrets: &secrets::Secrets,
    config: &config::Config,
) -> Result<()> {
    info!("Generating docker-compose.yml based on hardware profile...");
    let top_level = build_compose_structure(hw, secrets, config)?;
    let yaml_output = serde_yaml_ng::to_string(&top_level)?;

    fs::write("docker-compose.yml", yaml_output).context("Failed to write docker-compose.yml")?;
    info!("docker-compose.yml generated.");

    Ok(())
}
