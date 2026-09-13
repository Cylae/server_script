use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default, serialize_with = "serialize_sorted")]
    pub disabled_services: HashSet<String>,
}

fn serialize_sorted<S: serde::Serializer>(
    set: &HashSet<String>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut values: Vec<_> = set.iter().collect();
    values.sort();
    values.serialize(serializer)
}

impl Config {
    pub fn get_config_path() -> PathBuf {
        let local = Path::new("config.yaml");
        let installed = Path::new("/opt/server_manager/config.yaml");
        if local.exists() || !installed.exists() {
            local.into()
        } else {
            installed.into()
        }
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(content) if content.trim().is_empty() => Ok(Self::default()),
            Ok(content) => serde_yaml_ng::from_str(&content)
                .map_err(|_| anyhow::anyhow!("Invalid config YAML")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).context("Failed to read config YAML"),
        }
    }

    pub fn load() -> Result<Self> {
        Self::load_from(&Self::get_config_path())
    }
    pub async fn load_async() -> Result<Self> {
        tokio::task::spawn_blocking(Self::load).await?
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        let content = serde_yaml_ng::to_string(self)?;
        crate::core::atomic_io::atomic_write_str(path, &content, 0o644)
    }
    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::get_config_path())
    }
    pub fn is_enabled(&self, name: &str) -> bool {
        !self.disabled_services.contains(name)
    }
    pub fn enable_service(&mut self, name: &str) {
        if self.disabled_services.remove(name) {
            info!("Enabled service: {}", name);
        }
    }
    pub fn disable_service(&mut self, name: &str) {
        if self.disabled_services.insert(name.into()) {
            info!("Disabled service: {}", name);
        }
    }

    pub fn update_service_at<F>(path: &Path, name: &str, update: F) -> Result<()>
    where
        F: FnOnce(&mut Self, &str) -> bool,
    {
        crate::core::validate::validate_service_name(name)?;
        let _lock = crate::core::lock::ProcessLock::acquire(
            path.with_extension("transaction.lock"),
            false,
        )?;
        let mut config = Self::load_from(path)?;
        if update(&mut config, name) {
            config.save_to(path)?;
        }
        Ok(())
    }

    pub async fn update_service_async<F>(name: &str, update: F) -> Result<()>
    where
        F: FnOnce(&mut Self, &str) -> bool + Send + 'static,
    {
        let path = Self::get_config_path();
        let name = name.to_owned();
        tokio::task::spawn_blocking(move || Self::update_service_at(&path, &name, update)).await?
    }
    pub async fn enable_service_async(name: &str) -> Result<()> {
        Self::update_service_async(name, |cfg, name| cfg.disabled_services.remove(name)).await
    }
    pub async fn disable_service_async(name: &str) -> Result<()> {
        Self::update_service_async(name, |cfg, name| cfg.disabled_services.insert(name.into()))
            .await
    }
}
