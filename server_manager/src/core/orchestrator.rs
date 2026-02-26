use crate::core::{config, hardware, secrets};
use crate::{build_compose_structure, services};
use anyhow::{Context, Result};
use log::{error, info};
use std::process::Stdio;
use tokio::process::Command;

pub async fn apply(
    hw: &hardware::HardwareInfo,
    secrets: &secrets::Secrets,
    config: &config::Config,
) -> Result<()> {
    // 1. Configure & Initialize (Blocking IO)
    let hw_clone = hw.clone();
    let secrets_clone = secrets.clone();
    let config_clone = config.clone();

    tokio::task::spawn_blocking(move || {
        configure_services(&hw_clone, &secrets_clone, &config_clone)?;
        initialize_services(&hw_clone, &secrets_clone, &config_clone)?;
        generate_compose_file(&hw_clone, &secrets_clone, &config_clone)?;
        Ok::<(), anyhow::Error>(())
    })
    .await
    .context("Failed to execute blocking configuration tasks")??;

    // 2. Docker Compose Up (Async)
    info!("Applying changes via Docker Compose...");
    let status = Command::new("docker")
        .args(["compose", "up", "-d", "--remove-orphans"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .context("Failed to run docker compose up")?;

    if status.success() {
        info!("Stack deployed successfully!");
    } else {
        error!("Docker Compose failed.");
        anyhow::bail!("Docker Compose failed");
    }

    Ok(())
}

pub async fn generate_only(
    hw: &hardware::HardwareInfo,
    secrets: &secrets::Secrets,
    config: &config::Config,
) -> Result<()> {
    let hw_clone = hw.clone();
    let secrets_clone = secrets.clone();
    let config_clone = config.clone();

    tokio::task::spawn_blocking(move || {
        configure_services(&hw_clone, &secrets_clone, &config_clone)?;
        generate_compose_file(&hw_clone, &secrets_clone, &config_clone)
    })
    .await
    .context("Failed to execute blocking generation tasks")??;

    Ok(())
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

fn generate_compose_file(
    hw: &hardware::HardwareInfo,
    secrets: &secrets::Secrets,
    config: &config::Config,
) -> Result<()> {
    info!("Generating docker-compose.yml based on hardware profile...");
    let top_level = build_compose_structure(hw, secrets, config)?;
    let yaml_output = serde_yaml_ng::to_string(&top_level)?;

    std::fs::write("docker-compose.yml", yaml_output)
        .context("Failed to write docker-compose.yml")?;
    info!("docker-compose.yml generated.");

    Ok(())
}
