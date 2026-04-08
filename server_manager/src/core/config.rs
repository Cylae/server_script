use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::SystemTime;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedConfig {
    config: Config,
    last_mtime: Option<SystemTime>,
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
        let fallback_path = Path::new("/opt/server_manager/config.yaml");
        let load_path = if path.exists() {
            path
        } else if fallback_path.exists() {
            fallback_path
        } else {
            return Ok(Config::default());
        };

        let content = fs::read_to_string(load_path).context("Failed to read config.yaml")?;
        // If empty file, return default
        if content.trim().is_empty() {
            return Ok(Config::default());
        }
        serde_yaml_ng::from_str(&content).context("Failed to parse config.yaml")
    }

    pub async fn load_async() -> Result<Self> {
        let cache = CONFIG_CACHE.get_or_init(|| {
            RwLock::new(CachedConfig {
                config: Config::default(),
                last_mtime: None,
            })
        });

        let path = Path::new("config.yaml");
        let fallback_path = Path::new("/opt/server_manager/config.yaml");
        let load_path = if path.exists() {
            path
        } else {
            fallback_path
        };

        // Fast path: Optimistic read
        {
            let guard = cache.read().await;
            if let Some(cached_mtime) = guard.last_mtime {
                // Check if file still matches
                if let Ok(metadata) = tokio::fs::metadata(load_path).await {
                    if let Ok(modified) = metadata.modified() {
                        if modified == cached_mtime {
                            return Ok(guard.config.clone());
                        }
                    }
                }
            }
        }

        // Slow path: Update cache
        let mut guard = cache.write().await;

        // Check metadata again (double-checked locking pattern)
        let metadata_res = tokio::fs::metadata(load_path).await;

        match metadata_res {
            Ok(metadata) => {
                let modified = metadata.modified().unwrap_or(SystemTime::now());

                if let Some(cached_mtime) = guard.last_mtime {
                    if modified == cached_mtime {
                        return Ok(guard.config.clone());
                    }
                }

                // Load file
                match tokio::fs::read_to_string(load_path).await {
                    Ok(content) => {
                        let config = if content.trim().is_empty() {
                            Config::default()
                        } else {
                            serde_yaml_ng::from_str(&content)
                                .context("Failed to parse config.yaml")?
                        };

                        guard.config = config.clone();
                        guard.last_mtime = Some(modified);
                        Ok(config)
                    }
                    Err(e) => Err(anyhow::Error::new(e).context("Failed to read config.yaml")),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File not found -> Default
                guard.config = Config::default();
                guard.last_mtime = None;
                Ok(guard.config.clone())
            }
            Err(e) => Err(anyhow::Error::new(e).context("Failed to read config metadata")),
        }
    }

    pub fn save(&self) -> Result<()> {
        let target = if Path::new("/opt/server_manager").exists() {
            Path::new("/opt/server_manager/config.yaml")
        } else {
            Path::new("config.yaml")
        };
        let content = serde_yaml_ng::to_string(self)?;
        fs::write(target, content).context("Failed to write config.yaml")?;
        Ok(())
    }

    pub async fn update_service_async<F>(service_name: &str, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Config, &str) + Send,
    {
        let cache = CONFIG_CACHE.get_or_init(|| {
            RwLock::new(CachedConfig {
                config: Config::default(),
                last_mtime: None,
            })
        });

        let mut guard = cache.write().await;

        let path = Path::new("config.yaml");
        let fallback_path = Path::new("/opt/server_manager/config.yaml");
        let load_path = if path.exists() { path } else { fallback_path };

        // Reload if stale before applying update
        if let Ok(metadata) = tokio::fs::metadata(load_path).await {
            let modified = metadata.modified().unwrap_or_else(|_| SystemTime::now());
            if guard.last_mtime != Some(modified) {
                if let Ok(content) = tokio::fs::read_to_string(load_path).await {
                    if !content.trim().is_empty() {
                        if let Ok(cfg) = serde_yaml_ng::from_str(&content) {
                            guard.config = cfg;
                        }
                    }
                }
            }
        }

        f(&mut guard.config, service_name);

        guard.config.save()?;

        if let Ok(metadata) = tokio::fs::metadata(load_path).await {
            guard.last_mtime = metadata.modified().ok();
        }

        Ok(())
    }

    pub async fn enable_service_async(service_name: &str) -> Result<()> {
        Self::update_service_async(service_name, |config, name| {
            config.enable_service(name);
        }).await
    }

    pub async fn disable_service_async(service_name: &str) -> Result<()> {
        Self::update_service_async(service_name, |config, name| {
            config.disable_service(name);
        }).await
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
