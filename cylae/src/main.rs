mod core;
mod services;

use clap::{Parser, Subcommand};
use log::{info, error, LevelFilter};
use std::collections::HashMap;
use anyhow::{Result, Context};
use std::process::Command;
use std::fs;

use crate::core::{system, hardware, firewall, docker, secrets};

#[derive(Parser)]
#[command(name = "cylae")]
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
    info!("Starting Cylae Installation...");

    // 1. Root Check
    system::check_root()?;

    // 1.1 Create Install Directory
    let install_dir = std::path::Path::new("/opt/cylae");
    if !install_dir.exists() {
        info!("Creating installation directory at /opt/cylae...");
        fs::create_dir_all(install_dir).context("Failed to create /opt/cylae")?;
    }
    std::env::set_current_dir(install_dir).context("Failed to chdir to /opt/cylae")?;

    // 1.2 Load Secrets
    let secrets = secrets::Secrets::load_or_create()?;

    // 2. Hardware Detection
    let hw = hardware::HardwareInfo::detect();

    // 3. System Dependencies & Optimization
    system::install_dependencies()?;
    system::apply_optimizations()?;

    // 4. Firewall
    firewall::configure()?;

    // 5. Docker
    docker::install()?;

    // 6. Initialize Services
    let services = services::get_all_services();
    for service in &services {
        service.initialize(&hw, install_dir).context(format!("Failed to initialize service {}", service.name()))?;
    }

    // 7. Generate Compose
    generate_compose(&hw, &secrets, &services).await?;

    // 8. Launch
    info!("Launching Services via Docker Compose...");
    let status = Command::new("docker")
        .args(&["compose", "up", "-d", "--remove-orphans"])
        .status()
        .context("Failed to run docker compose up")?;

    if status.success() {
        info!("Cylae Stack Deployed Successfully! 🚀");
    } else {
        error!("Docker Compose failed.");
    }

    Ok(())
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
    // For generate, we might not be in /opt/cylae, but let's try to load secrets from CWD.
    // We propagate the error because generating a compose file with empty passwords is bad.
    let secrets = secrets::Secrets::load_or_create().context("Failed to load or create secrets.yaml")?;
    let services = services::get_all_services();
    generate_compose(&hw, &secrets, &services).await
}

async fn generate_compose(hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, services: &[Box<dyn services::Service>]) -> Result<()> {
    info!("Generating docker-compose.yml based on hardware profile...");

    let mut compose_services = HashMap::new();

    for service in services {
        let mut config = serde_yaml::Mapping::new();
        config.insert("image".into(), service.image().into());
        config.insert("container_name".into(), service.name().into());
        config.insert("restart".into(), "unless-stopped".into());

        // Ports
        let ports = service.ports();
        if !ports.is_empty() {
             let mut ports_seq = serde_yaml::Sequence::new();
             for p in ports {
                 ports_seq.push(p.into());
             }
             config.insert("ports".into(), serde_yaml::Value::Sequence(ports_seq));
        }

        // Env Vars
        let envs = service.env_vars(hw, secrets);
        if !envs.is_empty() {
             let mut env_seq = serde_yaml::Sequence::new();
             for (k, v) in envs {
                 env_seq.push(format!("{}={}", k, v).into());
             }
             config.insert("environment".into(), serde_yaml::Value::Sequence(env_seq));
        }

        // Volumes
        let vols = service.volumes(hw);
        if !vols.is_empty() {
             let mut vol_seq = serde_yaml::Sequence::new();
             for v in vols {
                 vol_seq.push(v.into());
             }
             config.insert("volumes".into(), serde_yaml::Value::Sequence(vol_seq));
        }

        // Devices
        let devs = service.devices(hw);
        if !devs.is_empty() {
             let mut dev_seq = serde_yaml::Sequence::new();
             for d in devs {
                 dev_seq.push(d.into());
             }
             config.insert("devices".into(), serde_yaml::Value::Sequence(dev_seq));
        }

        // Networks
        let nets = service.networks();
        if !nets.is_empty() {
             let mut net_seq = serde_yaml::Sequence::new();
             for n in nets {
                 net_seq.push(n.into());
             }
             config.insert("networks".into(), serde_yaml::Value::Sequence(net_seq));
        }

        // Healthcheck
        if let Some(cmd) = service.healthcheck() {
             let mut hc = serde_yaml::Mapping::new();
             // curl command needs to be split if using ["CMD-SHELL", ...] or just string.
             // Docker compose supports string or list.
             // We'll use list for safety: ["CMD-SHELL", cmd]
             let mut test_seq = serde_yaml::Sequence::new();
             test_seq.push("CMD-SHELL".into());
             test_seq.push(cmd.into());

             hc.insert("test".into(), serde_yaml::Value::Sequence(test_seq));
             hc.insert("interval".into(), "1m".into());
             hc.insert("retries".into(), 3.into());
             hc.insert("start_period".into(), "30s".into());
             hc.insert("timeout".into(), "10s".into());

             config.insert("healthcheck".into(), serde_yaml::Value::Mapping(hc));
        }

        // Depends On
        let deps = service.depends_on();
        if !deps.is_empty() {
             let mut dep_seq = serde_yaml::Sequence::new();
             for d in deps {
                 dep_seq.push(d.into());
             }
             config.insert("depends_on".into(), serde_yaml::Value::Sequence(dep_seq));
        }

        // Security Opts
        let sec_opts = service.security_opts();
        if !sec_opts.is_empty() {
             let mut sec_seq = serde_yaml::Sequence::new();
             for s in sec_opts {
                 sec_seq.push(s.into());
             }
             config.insert("security_opt".into(), serde_yaml::Value::Sequence(sec_seq));
        }

        // Labels
        let labels = service.labels();
        if !labels.is_empty() {
             let mut labels_seq = serde_yaml::Sequence::new();
             for (k, v) in labels {
                 labels_seq.push(format!("{}={}", k, v).into());
             }
             config.insert("labels".into(), serde_yaml::Value::Sequence(labels_seq));
        }

        // Cap Add
        let caps = service.cap_add();
        if !caps.is_empty() {
             let mut cap_seq = serde_yaml::Sequence::new();
             for c in caps {
                 cap_seq.push(c.into());
             }
             config.insert("cap_add".into(), serde_yaml::Value::Sequence(cap_seq));
        }

        // Sysctls
        let sysctls = service.sysctls();
        if !sysctls.is_empty() {
             let mut sys_seq = serde_yaml::Sequence::new();
             for s in sysctls {
                 sys_seq.push(s.into());
             }
             config.insert("sysctls".into(), serde_yaml::Value::Sequence(sys_seq));
        }

        // Logging
        if let Some(logging_opts) = service.logging() {
            let mut logging = serde_yaml::Mapping::new();
            logging.insert("driver".into(), "json-file".into());

            let mut opts = serde_yaml::Mapping::new();
            for (k, v) in logging_opts {
                opts.insert(k.into(), v.into());
            }
            logging.insert("options".into(), serde_yaml::Value::Mapping(opts));
            config.insert("logging".into(), serde_yaml::Value::Mapping(logging));
        }

        compose_services.insert(service.name().to_string(), serde_yaml::Value::Mapping(config));
    }

    // Networks definition
    let mut networks_map = serde_yaml::Mapping::new();
    let mut cylae_net = serde_yaml::Mapping::new();
    cylae_net.insert("driver".into(), "bridge".into());
    networks_map.insert("cylae_net".into(), serde_yaml::Value::Mapping(cylae_net));

    // Top Level
    let mut top_level = serde_yaml::Mapping::new();
    top_level.insert("services".into(), serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(
        compose_services.into_iter().map(|(k, v)| (k.into(), v))
    )));
    top_level.insert("networks".into(), serde_yaml::Value::Mapping(networks_map));

    let yaml_output = serde_yaml::to_string(&top_level)?;

    fs::write("docker-compose.yml", yaml_output).context("Failed to write docker-compose.yml")?;
    info!("docker-compose.yml generated.");

    Ok(())
}
