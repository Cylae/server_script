use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{error, info, warn};
use std::fmt::Write;
use std::fs;
use std::io::{self, Write as IoWrite};
use std::process::Command;

use crate::build_compose_structure;
use crate::core::{config, docker, firewall, hardware, secrets, system, users};
use crate::services;

#[derive(Parser)]
#[command(name = "server_manager")]
#[command(
    about = "Next-Gen Media Server Orchestrator & Management Platform",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Full installation (Idempotent)
    Install,
    /// Apply configuration and deploy services without re-running system installations
    Apply,
    /// Pull latest service Docker images and recreate containers
    Update,
    /// Clean Docker unused resources, caches, and system log artifacts
    Clean,
    /// Fix system permissions, UFW firewall rules, and container runtime issues
    Fix,
    /// Launch interactive TUI menu for quick server management
    Interactive,
    /// Show comprehensive system status, telemetry, and container runtime state
    Status,
    /// Generate docker-compose.yml only
    Generate,
    /// Enable a service
    Enable { service: String },
    /// Disable a service
    Disable { service: String },
    /// Start the Web Administration Interface
    Web {
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
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
        Commands::Apply => run_apply().await?,
        Commands::Update => run_update().await?,
        Commands::Clean => run_clean().await?,
        Commands::Fix => run_fix().await?,
        Commands::Interactive => run_interactive().await?,
        Commands::Status => run_status().await?,
        Commands::Generate => run_generate().await?,
        Commands::Enable { service } => run_toggle_service(service, true).await?,
        Commands::Disable { service } => run_toggle_service(service, false).await?,
        Commands::Web { bind, port } => crate::interface::web::start_server(&bind, port).await?,
        Commands::User { action } => run_user_management(action).await?,
    }

    Ok(())
}

async fn run_user_management(action: UserCommands) -> Result<()> {
    let mut user_manager = users::UserManager::load_async().await?;

    match action {
        UserCommands::Add {
            username,
            role,
            quota,
        } => {
            let role_enum = match role.to_lowercase().as_str() {
                "admin" => users::Role::Admin,
                "operator" => users::Role::Operator,
                "observer" => users::Role::Observer,
                "auditor" => users::Role::Auditor,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid role '{}'. Valid roles are: Admin, Operator, Observer, Auditor",
                        role
                    ))
                }
            };

            let password =
                rpassword::prompt_password(format!("Enter password for {}: ", username))?;
            let password = password.trim().to_string();

            if password.is_empty() {
                return Err(anyhow::anyhow!("Password cannot be empty"));
            }

            user_manager.add_user(&username, &password, role_enum, quota)?;
            info!("User '{}' added successfully.", username);
        }
        UserCommands::Delete { username } => {
            user_manager.delete_user(&username)?;
            info!("User '{}' deleted successfully.", username);
        }
        UserCommands::List => {
            println!("┌──────────────────────┬─────────────────┐");
            println!("│ {:<20} │ {:<15} │", "Username", "Role");
            println!("├──────────────────────┼─────────────────┤");
            for user in user_manager.list_users() {
                println!("│ {:<20} │ {:<15?} │", user.username, user.role);
            }
            println!("└──────────────────────┴─────────────────┘");
        }
        UserCommands::Passwd { username } => {
            if user_manager.get_user(&username).is_none() {
                return Err(anyhow::anyhow!("User not found"));
            }
            let password =
                rpassword::prompt_password(format!("Enter new password for {}: ", username))?;
            let password = password.trim().to_string();

            if password.is_empty() {
                return Err(anyhow::anyhow!("Password cannot be empty"));
            }

            user_manager.update_password(&username, &password)?;
            info!("Password for '{}' updated successfully.", username);
        }
    }
    Ok(())
}

async fn run_clean() -> Result<()> {
    info!("Cleaning system caches and unused Docker resources...");

    info!("Running Docker system prune (removing stopped containers & dangling images)...");
    let prune_status = Command::new("docker")
        .args(["system", "prune", "-f"])
        .status();

    if let Ok(status) = prune_status {
        if status.success() {
            info!("Docker system prune completed successfully.");
        } else {
            warn!("Docker system prune exited with non-zero status.");
        }
    }

    info!("Vacuuming systemd journal logs (> 100M)...");
    let _ = Command::new("journalctl")
        .args(["--vacuum-size=100M"])
        .status();

    info!("Cleaning temporary files in /tmp/server_manager...");
    let temp_dir = std::path::Path::new("/tmp/server_manager");
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(temp_dir);
    }

    info!("System cleanup complete! ✨");
    Ok(())
}

async fn run_fix() -> Result<()> {
    info!("Diagnosing and repairing system state & permissions...");

    system::check_root()?;

    let opt_dir = std::path::Path::new("/opt/server_manager");
    if opt_dir.exists() {
        info!("Ensuring correct permissions on /opt/server_manager...");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(entries) = fs::read_dir(opt_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|s| s.ends_with(".yaml"))
                    {
                        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
                    }
                }
            }
        }
    }

    info!("Re-applying UFW firewall rules...");
    if let Err(e) = firewall::configure() {
        warn!("Failed to re-configure firewall: {}", e);
    }

    info!("Verifying Docker service status...");
    if let Err(e) = docker::install() {
        warn!("Docker verification returned an error: {}", e);
    }

    info!("System repair complete! 🛠️");
    Ok(())
}

async fn run_interactive() -> Result<()> {
    println!("\n=======================================================");
    println!("        🚀 Server Manager Interactive Console         ");
    println!("=======================================================");
    println!("  1. View System Status");
    println!("  2. Apply Configuration & Deploy");
    println!("  3. Update Stack Images");
    println!("  4. Clean System & Docker Caches");
    println!("  5. Repair & Fix System Permissions");
    println!("  6. List System Users");
    println!("  7. Start Web Dashboard (Port 8099)");
    println!("  8. Exit");
    println!("=======================================================");
    print!("Select an option [1-8]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => run_status().await?,
        "2" => run_apply().await?,
        "3" => run_update().await?,
        "4" => run_clean().await?,
        "5" => run_fix().await?,
        "6" => run_user_management(UserCommands::List).await?,
        "7" => crate::interface::web::start_server("127.0.0.1", 8099).await?,
        "8" | "" => println!("Exiting interactive mode."),
        _ => println!("Invalid option selected."),
    }

    Ok(())
}

async fn run_toggle_service(service_name: String, enable: bool) -> Result<()> {
    if !std::path::Path::new("config.yaml").exists()
        && std::path::Path::new("/opt/server_manager/config.yaml").exists()
    {
        std::env::set_current_dir("/opt/server_manager")?;
    }

    let mut config = config::Config::load()?;

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

    let secrets = secrets::Secrets::load_or_create()?;
    let hw = hardware::HardwareInfo::detect();

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

    system::check_root()?;

    let install_dir = std::path::Path::new("/opt/server_manager");
    if !install_dir.exists() {
        info!("Creating installation directory at /opt/server_manager...");
        fs::create_dir_all(install_dir).context("Failed to create /opt/server_manager")?;
    }
    std::env::set_current_dir(install_dir).context("Failed to chdir to /opt/server_manager")?;

    let secrets = secrets::Secrets::load_or_create()?;
    let config = config::Config::load()?;

    let hw = hardware::HardwareInfo::detect();

    system::install_dependencies()?;
    system::apply_optimizations(&hw)?;

    firewall::configure()?;

    docker::install()?;

    configure_services(&hw, &secrets, &config)?;
    initialize_services(&hw, &secrets, &config)?;

    generate_compose(&hw, &secrets, &config).await?;

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
    summary.push_str(
        "\n╔══════════════════════════════════════════════════════════════════════════════════════════════════╗\n",
    );
    summary.push_str(
        "║                                    DEPLOYMENT SUMMARY 🚀                                         ║\n",
    );
    summary.push_str(
        "╠═════════════════┬═════════════════════════┬═══════════════════┬══════════════════════════════════╣\n",
    );
    let _ = writeln!(
        summary,
        "║ {:<15} │ {:<23} │ {:<17} │ {:<32} ║",
        "Service", "URL", "User", "Password / Info"
    );
    summary.push_str(
        "╠─────────────────┼─────────────────────────┼───────────────────┼──────────────────────────────────╣\n",
    );

    let mut append_row = |service: &str, url: &str, user: &str, pass: &str| {
        let truncated_pass = if pass.len() > 32 { &pass[..32] } else { pass };
        let _ = writeln!(
            summary,
            "║ {:<15} │ {:<23} │ {:<17} │ {:<32} ║",
            service, url, user, truncated_pass
        );
    };

    let pass = |opt: &Option<String>| opt.as_deref().unwrap_or("ERROR").to_string();

    append_row(
        "Server Manager",
        "http://<IP>:8099",
        "admin",
        &pass(&secrets.server_manager_admin_password),
    );
    append_row(
        "Nginx Proxy",
        "http://<IP>:81",
        "admin@example.com",
        "changeme",
    );
    append_row(
        "Portainer",
        "http://<IP>:9000",
        "admin",
        "Set on first login",
    );
    append_row(
        "Nextcloud",
        "https://<IP>:4443",
        "admin",
        &pass(&secrets.nextcloud_admin_password),
    );
    append_row(
        "Vaultwarden",
        "http://<IP>:8001/admin",
        "(Token)",
        &pass(&secrets.vaultwarden_admin_token),
    );
    append_row("Gitea", "http://<IP>:3000", "Register", "DB pre-configured");
    append_row("GLPI", "http://<IP>:8088", "glpi", "glpi (Change!)");
    append_row(
        "Yourls",
        "http://<IP>:8003/admin",
        "admin",
        &pass(&secrets.yourls_admin_password),
    );
    append_row("Roundcube", "http://<IP>:8090", "-", "Login Mail creds");
    append_row("MailServer", "PORTS: 25, 143...", "CLI", "docker exec ...");
    append_row("Plex", "http://<IP>:32400/web", "-", "Follow Web Setup");
    append_row("ArrStack", "http://<IP>:8989", "-", "No auth default");

    summary.push_str(
        "╚═════════════════╧═════════════════════════╧═══════════════════╧══════════════════════════════════╝\n\n",
    );
    summary.push_str("NOTE: Replace <IP> with your server's IP address.");

    println!("{}", summary);

    if nix::unistd::Uid::effective().is_root() {
        if let Err(e) = std::fs::write("/root/credentials.txt", &summary) {
            error!("Failed to save credentials to /root/credentials.txt: {}", e);
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    "/root/credentials.txt",
                    std::fs::Permissions::from_mode(0o600),
                );
            }
            info!("Credentials saved to /root/credentials.txt (restricted permissions 0600)");
        }
    }
}

async fn run_status() -> Result<()> {
    let config = config::Config::load_async().await?;
    let hw = hardware::HardwareInfo::detect();
    let services = services::get_all_services();
    let total_services = services.len();
    let enabled_count = services
        .iter()
        .filter(|s| config.is_enabled(s.name()))
        .count();

    println!("\n╔═════════════════════════════════════════════════════════════╗");
    println!("║                    SYSTEM TELEMETRY                         ║");
    println!("╠═════════════════════════════════════════════════════════════╣");
    println!("║ RAM Total:       {:<42} ║", format!("{} GB", hw.ram_gb));
    println!("║ Swap Total:      {:<42} ║", format!("{} GB", hw.swap_gb));
    println!("║ Disk Root (/):   {:<42} ║", format!("{} GB", hw.disk_gb));
    println!("║ CPU Cores:       {:<42} ║", hw.cpu_cores);
    println!("║ Hardware Profile:{:<42?} ║", hw.profile);
    println!(
        "║ Nvidia GPU:      {:<42} ║",
        if hw.has_nvidia {
            "Enabled 🟢"
        } else {
            "Disabled"
        }
    );
    println!(
        "║ Intel QuickSync: {:<42} ║",
        if hw.has_intel_quicksync {
            "Enabled 🟢"
        } else {
            "Disabled"
        }
    );
    println!("╠═════════════════════════════════════════════════════════════╣");
    println!("║                    STACK TELEMETRY                          ║");
    println!("╠═════════════════════════════════════════════════════════════╣");
    println!(
        "║ Active Stack:    {:<42} ║",
        format!("{} / {} Services Enabled", enabled_count, total_services)
    );

    if let Ok(true) = tokio::process::Command::new("docker")
        .arg("ps")
        .status()
        .await
        .map(|s| s.success())
    {
        println!("║ Docker Daemon:   Active 🟢                                  ║");
    } else {
        println!("║ Docker Daemon:   Inactive 🔴                                ║");
    }
    println!("╚═════════════════════════════════════════════════════════════╝\n");
    Ok(())
}

async fn run_update() -> Result<()> {
    info!("Updating Server Manager Stack...");

    if !std::path::Path::new("docker-compose.yml").exists()
        && std::path::Path::new("/opt/server_manager/docker-compose.yml").exists()
    {
        std::env::set_current_dir("/opt/server_manager")?;
    }

    info!("Pulling latest Docker images...");
    let pull_status = tokio::process::Command::new("docker")
        .args(["compose", "pull"])
        .status()
        .await
        .context("Failed to run docker compose pull")?;

    if !pull_status.success() {
        log::warn!("Some images failed to pull or docker compose pull returned non-zero status.");
    }

    info!("Re-deploying updated services...");
    let up_status = tokio::process::Command::new("docker")
        .args(["compose", "up", "-d", "--remove-orphans"])
        .status()
        .await
        .context("Failed to run docker compose up")?;

    if up_status.success() {
        info!("Server Manager Stack updated successfully! 🚀");
    } else {
        error!("Failed to re-deploy stack via Docker Compose.");
    }

    Ok(())
}

async fn run_apply() -> Result<()> {
    info!("Applying Server Manager Configuration...");

    let secrets = secrets::Secrets::load_or_create()?;
    let config = config::Config::load_async().await?;
    let hw = hardware::HardwareInfo::detect();

    configure_services(&hw, &secrets, &config)?;
    initialize_services(&hw, &secrets, &config)?;
    generate_compose(&hw, &secrets, &config).await?;

    info!("Applying changes via Docker Compose...");
    let status = tokio::process::Command::new("docker")
        .args(["compose", "up", "-d", "--remove-orphans"])
        .status()
        .await
        .context("Failed to run docker compose up")?;

    if status.success() {
        info!("Server Manager Configuration Applied Successfully! 🚀");
    } else {
        error!("Failed to apply changes via Docker Compose.");
    }

    Ok(())
}

async fn run_generate() -> Result<()> {
    let hw = hardware::HardwareInfo::detect();
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

    crate::core::atomic_io::atomic_write_str("docker-compose.yml", &yaml_output, 0o644)
        .context("Failed to write docker-compose.yml")?;
    info!("docker-compose.yml generated.");

    Ok(())
}
