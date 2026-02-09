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
        if path.exists() {
            let content = fs::read_to_string(path).context("Failed to read config.yaml")?;
            // If empty file, return default
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
            })
        });

        // Fast path: Optimistic read
        {
            let guard = cache.read().await;
            if let Some(cached_mtime) = guard.last_mtime {
                // Check if file still matches
                if let Ok(metadata) = tokio::fs::metadata("config.yaml").await {
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
        let metadata_res = tokio::fs::metadata("config.yaml").await;

        match metadata_res {
            Ok(metadata) => {
                let modified = metadata.modified().unwrap_or(SystemTime::now());

                if let Some(cached_mtime) = guard.last_mtime {
                    if modified == cached_mtime {
                        return Ok(guard.config.clone());
                    }
                }

                // Load file
                match tokio::fs::read_to_string("config.yaml").await {
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
