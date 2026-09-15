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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_defaults() {
        let temp_dir = std::env::temp_dir().join(format!("test_cfg_{}", rand::random::<u64>()));
        let _ = fs::create_dir_all(&temp_dir);
        let non_existent = temp_dir.join("non_existent.yaml");
        let cfg = Config::load_from(&non_existent).unwrap();
        assert!(cfg.disabled_services.is_empty());

        let empty_file = temp_dir.join("empty.yaml");
        fs::write(&empty_file, "   \n  ").unwrap();
        let cfg2 = Config::load_from(&empty_file).unwrap();
        assert!(cfg2.disabled_services.is_empty());

        let invalid_file = temp_dir.join("invalid.yaml");
        fs::write(&invalid_file, ": : invalid yaml :::").unwrap();
        let err = Config::load_from(&invalid_file);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().to_string(), "Invalid config YAML");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_config_save_load_roundtrip_and_enable_disable() {
        let temp_dir = std::env::temp_dir().join(format!("test_cfg_rt_{}", rand::random::<u64>()));
        let _ = fs::create_dir_all(&temp_dir);
        let config_path = temp_dir.join("config.yaml");

        let mut cfg = Config::default();
        assert!(cfg.is_enabled("plex"));

        cfg.disable_service("plex");
        assert!(!cfg.is_enabled("plex"));
        // Disabling again returns false
        cfg.disable_service("plex");

        cfg.disable_service("radarr");
        cfg.disable_service("sonarr");

        cfg.save_to(&config_path).unwrap();

        let loaded = Config::load_from(&config_path).unwrap();
        assert!(!loaded.is_enabled("plex"));
        assert!(!loaded.is_enabled("radarr"));
        assert!(!loaded.is_enabled("sonarr"));
        assert!(loaded.is_enabled("jellyfin"));

        let content = fs::read_to_string(&config_path).unwrap();
        // Verify sorted serialization: plex, radarr, sonarr
        let plex_pos = content.find("plex").unwrap();
        let radarr_pos = content.find("radarr").unwrap();
        let sonarr_pos = content.find("sonarr").unwrap();
        assert!(plex_pos < radarr_pos && radarr_pos < sonarr_pos);

        let mut modified = loaded;
        modified.enable_service("plex");
        assert!(modified.is_enabled("plex"));
        // Enabling again returns false
        modified.enable_service("plex");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_update_service_at() {
        let temp_dir = std::env::temp_dir().join(format!("test_cfg_upd_{}", rand::random::<u64>()));
        let _ = fs::create_dir_all(&temp_dir);
        let config_path = temp_dir.join("config.yaml");

        // Invalid service name fails immediately
        let err = Config::update_service_at(&config_path, "../malicious", |cfg, name| {
            cfg.disable_service(name);
            true
        });
        assert!(err.is_err());

        // Valid update persists
        let res = Config::update_service_at(&config_path, "plex", |cfg, name| {
            cfg.disable_service(name);
            true
        });
        assert!(res.is_ok());

        let loaded = Config::load_from(&config_path).unwrap();
        assert!(!loaded.is_enabled("plex"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_config_path() {
        let path = Config::get_config_path();
        assert!(path.ends_with("config.yaml"));
    }
}
