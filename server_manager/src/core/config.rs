use anyhow::{Context, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedConfig {
    config: Config,
    last_mtime: Option<SystemTime>,
    last_check: Option<Instant>,
}

static CONFIG_CACHE: OnceLock<RwLock<CachedConfig>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub disabled_services: HashSet<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Path::new("config.yaml");
        if path.exists() {
            let content = fs::read_to_string(path).context("Failed to read config.yaml")?;
            if content.trim().is_empty() {
                return Ok(Config::default());
            }
            serde_yaml_ng::from_str(&content).context("Failed to parse config.yaml")
        } else {
            Ok(Config::default())
        }
    }

    pub async fn load_async() -> Result<Self> {
        let cache = CONFIG_CACHE.get_or_init(|| {
            RwLock::new(CachedConfig {
                config: Config::default(),
                last_mtime: None,
                last_check: None,
            })
        });

        {
            let guard = cache.read().await;
            if let Some(last_check) = guard.last_check {
                if last_check.elapsed() < Duration::from_millis(500) {
                    return Ok(guard.config.clone());
                }
            }
        }

        let mut guard = cache.write().await;
        if let Some(last_check) = guard.last_check {
            if last_check.elapsed() < Duration::from_millis(500) {
                return Ok(guard.config.clone());
            }
        }

        Self::reload_guard(&mut guard).await?;

        guard.last_check = Some(Instant::now());
        Ok(guard.config.clone())
    }

    async fn reload_guard(guard: &mut tokio::sync::RwLockWriteGuard<'_, CachedConfig>) -> Result<()> {
        let last_mtime = guard.last_mtime;

        // Use blocking IO inside spawn_blocking to be consistent with load() and robust
        let res = tokio::task::spawn_blocking(move || -> Result<Option<(Config, Option<SystemTime>)>> {
            let path = Path::new("config.yaml");
            let mtime = match std::fs::metadata(path) {
                Ok(m) => m.modified().ok(),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(anyhow::Error::new(e).context("Failed to read config metadata")),
            };

            if mtime == last_mtime && last_mtime.is_some() {
                 return Ok(None);
            }

            // Reload
            // load() handles file reading/parsing
            match Self::load() {
                Ok(cfg) => Ok(Some((cfg, mtime))),
                Err(e) => {
                     // If parsing fails, we might return error.
                     // But load_async behavior was to return cached config on error.
                     // Here we return Err. The caller (reload_guard) needs to handle this policy?
                     // Or load() should handle it? load() returns Err on parse failure.
                     // The previous load_async implementation swallowed errors.
                     // To match that:
                     warn!("Failed to reload config: {}. Preserving cache.", e);
                     Ok(None)
                }
            }
        }).await.map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;

        if let Some((cfg, mtime)) = res {
             guard.config = cfg;
             guard.last_mtime = mtime;
        }
        Ok(())
    }

    pub async fn enable_service_async(service_name: String) -> Result<()> {
        let cache = CONFIG_CACHE.get_or_init(|| {
            RwLock::new(CachedConfig {
                config: Config::default(),
                last_mtime: None,
                last_check: None,
            })
        });

        let mut guard = cache.write().await;
        Self::reload_guard(&mut guard).await?;

        let mut config = guard.config.clone();

        let (new_config, new_mtime) = tokio::task::spawn_blocking(move || -> Result<(Config, Option<SystemTime>)> {
            config.enable_service(&service_name);
            config.save()?;
            let mtime = std::fs::metadata("config.yaml").ok().and_then(|m| m.modified().ok());
            Ok((config, mtime))
        }).await.map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;

        guard.config = new_config;
        guard.last_mtime = new_mtime;
        guard.last_check = Some(Instant::now());

        Ok(())
    }

    pub async fn disable_service_async(service_name: String) -> Result<()> {
        let cache = CONFIG_CACHE.get_or_init(|| {
            RwLock::new(CachedConfig {
                config: Config::default(),
                last_mtime: None,
                last_check: None,
            })
        });

        let mut guard = cache.write().await;
        Self::reload_guard(&mut guard).await?;

        let mut config = guard.config.clone();

        let (new_config, new_mtime) = tokio::task::spawn_blocking(move || -> Result<(Config, Option<SystemTime>)> {
            config.disable_service(&service_name);
            config.save()?;
            let mtime = std::fs::metadata("config.yaml").ok().and_then(|m| m.modified().ok());
            Ok((config, mtime))
        }).await.map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;

        guard.config = new_config;
        guard.last_mtime = new_mtime;
        guard.last_check = Some(Instant::now());

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_yaml_ng::to_string(self)?;
        fs::write("config.yaml", content).context("Failed to write config.yaml")?;
        Ok(())
    }

    pub fn is_enabled(&self, service_name: &str) -> bool {
        !self.disabled_services.contains(service_name)
    }

    pub fn enable_service(&mut self, service_name: &str) {
        if self.disabled_services.remove(service_name) {
            info!("Enabled service: {}", service_name);
        }
    }

    pub fn disable_service(&mut self, service_name: &str) {
        if self.disabled_services.insert(service_name.to_string()) {
            info!("Disabled service: {}", service_name);
        }
    }
}
