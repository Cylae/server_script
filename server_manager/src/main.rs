use clap::{Parser, Subcommand};
use log::{info, error, LevelFilter};
use anyhow::{Result, Context};
use std::process::Command;
use std::fs;

// Use the lib crate
use server_manager::core::{system, hardware, firewall, docker, secrets};
use server_manager::services;
use server_manager::build_compose_structure;

#[derive(Parser)]
#[command(name = "server_manager")]
#[command(about = "Next-Gen Media Server Orchestrator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Full installation (Idempotent)
    Install,
    /// Show system status
    Status,
    /// Generate docker-compose.yml only
    Generate,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install => run_install().await?,
        Commands::Status => run_status(),
        Commands::Generate => run_generate().await?,
    }

    Ok(())
}

async fn run_install() -> Result<()> {
    info!("Starting Server Manager Installation...");

    // 1. Root Check
    system::check_root()?;

    // 1.0 Ensure Media User
    let (uid, gid) = system::ensure_media_user()?;

    // 1.1 Create Install Directory
    let install_dir = std::path::Path::new("/opt/server_manager");
    if !install_dir.exists() {
        info!("Creating installation directory at /opt/server_manager...");
        fs::create_dir_all(install_dir).context("Failed to create /opt/server_manager")?;
    }
    std::env::set_current_dir(install_dir).context("Failed to chdir to /opt/server_manager")?;

    // 1.2 Load Secrets
    let secrets = secrets::Secrets::load_or_create()?;

    // 2. Hardware Detection
    let mut hw = hardware::HardwareInfo::detect();
    hw.user_id = uid;
    hw.group_id = gid;

    // 3. System Dependencies & Optimization
    system::install_dependencies()?;
    system::apply_optimizations()?;

    // 4. Firewall
    firewall::configure()?;

    // 5. Docker
    docker::install()?;

    // 6. Initialize Services
    configure_services(&hw, &secrets)?;
    initialize_services(&hw, &secrets)?;

    // 7. Generate Compose
    generate_compose(&hw, &secrets).await?;

    // 7.1 Set Permissions
    system::chown_recursive(install_dir, uid, gid)?;

    // 8. Launch
    info!("Launching Services via Docker Compose...");
    let status = Command::new("docker")
        .args(&["compose", "up", "-d", "--remove-orphans"])
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
    println!("\n=================================================================================");
    println!("                           DEPLOYMENT SUMMARY 🚀");
    println!("=================================================================================");
    println!("{:<15} | {:<25} | {:<15} | {}", "Service", "URL", "User", "Password / Info");
    println!("{:<15} | {:<25} | {:<15} | {}", "-------", "---", "----", "---------------");

    let print_row = |service: &str, url: &str, user: &str, pass: &str| {
        println!("{:<15} | {:<25} | {:<15} | {}", service, url, user, pass);
    };

    // Helper to format Option<String>
    let pass = |opt: &Option<String>| opt.clone().unwrap_or_else(|| "ERROR".to_string());

    print_row("Nginx Proxy", "http://<IP>:81", "admin@example.com", "changeme");
    print_row("Portainer", "http://<IP>:9000", "admin", "Set on first login");
    print_row("Nextcloud", "https://<IP>:4443", "admin", &pass(&secrets.nextcloud_admin_password));
    print_row("Vaultwarden", "http://<IP>:8001/admin", "(Token)", &pass(&secrets.vaultwarden_admin_token));
    print_row("Gitea", "http://<IP>:3000", "Register", "DB pre-configured");
    print_row("GLPI", "http://<IP>:8088", "glpi", "glpi (Change immediately!)");
    print_row("Yourls", "http://<IP>:8003/admin", "admin", &pass(&secrets.yourls_admin_password));
    print_row("Roundcube", "http://<IP>:8090", "-", "Login with Mail creds");
    print_row("MailServer", "PORTS: 25, 143...", "CLI", "docker exec -ti mailserver setup ...");
    print_row("Plex", "http://<IP>:32400/web", "-", "Follow Web Setup");
    print_row("ArrStack", "http://<IP>:8989 (Sonarr)", "-", "No auth by default");

    println!("=================================================================================\n");
    println!("NOTE: Replace <IP> with your server's IP address.");
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
    if let Ok(true) = Command::new("docker").arg("ps").status().map(|s| s.success()) {
         println!("Docker is running.");
    } else {
         println!("Docker is NOT running.");
    }
}

async fn run_generate() -> Result<()> {
    let hw = hardware::HardwareInfo::detect();
    // For generate, we might not be in /opt/server_manager, but let's try to load secrets from CWD.
    // We propagate the error because generating a compose file with empty passwords is bad.
    let secrets = secrets::Secrets::load_or_create().context("Failed to load or create secrets.yaml")?;
    configure_services(&hw, &secrets)?;
    generate_compose(&hw, &secrets).await
}

fn configure_services(hw: &hardware::HardwareInfo, secrets: &secrets::Secrets) -> Result<()> {
    info!("Configuring services (generating config files)...");
    let services = services::get_all_services();
    for service in services {
        service.configure(hw, secrets).with_context(|| format!("Failed to configure service: {}", service.name()))?;
    }
    Ok(())
}

fn initialize_services(hw: &hardware::HardwareInfo, secrets: &secrets::Secrets) -> Result<()> {
    info!("Initializing services (system setup)...");
    let services = services::get_all_services();
    for service in services {
        service.initialize(hw, secrets).with_context(|| format!("Failed to initialize service: {}", service.name()))?;
    }
    Ok(())
}

async fn generate_compose(hw: &hardware::HardwareInfo, secrets: &secrets::Secrets) -> Result<()> {
    info!("Generating docker-compose.yml based on hardware profile...");
    let top_level = build_compose_structure(hw, secrets)?;
    let yaml_output = serde_yaml::to_string(&top_level)?;

    fs::write("docker-compose.yml", yaml_output).context("Failed to write docker-compose.yml")?;
    info!("docker-compose.yml generated.");

    Ok(())
}
