use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::SystemTime;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedConfig {
    config: Config,
    last_mtime: Option<SystemTime>,
    loaded_path: Option<PathBuf>,
}

static CONFIG_CACHE: OnceLock<RwLock<CachedConfig>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub disabled_services: HashSet<String>,
}

impl Config {
    fn get_config_path() -> PathBuf {
        let local_path = Path::new("./config.yaml");
        let opt_path = Path::new("/opt/server_manager/config.yaml");
        if local_path.exists() {
            local_path.to_path_buf()
        } else if opt_path.exists() {
            opt_path.to_path_buf()
        } else {
            // Default to local if neither exists
            PathBuf::from("./config.yaml")
        }
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            if content.trim().is_empty() {
                return Ok(Config::default());
            }
            serde_yaml_ng::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))
        } else {
            Ok(Config::default())
        }
    }

    pub async fn load_async() -> Result<Self> {
        let cache = CONFIG_CACHE.get_or_init(|| {
            RwLock::new(CachedConfig {
                config: Config::default(),
                last_mtime: None,
                loaded_path: None,
            })
        });

        let path = Self::get_config_path();

        // Fast path: Optimistic read
        {
            let guard = cache.read().await;
            if let Some(cached_mtime) = guard.last_mtime {
                if let Ok(metadata) = tokio::fs::metadata(&path).await {
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

        let metadata_res = tokio::fs::metadata(&path).await;

        match metadata_res {
            Ok(metadata) => {
                let modified = metadata.modified().unwrap_or_else(|_| SystemTime::now());

                if let Some(cached_mtime) = guard.last_mtime {
                    if modified == cached_mtime {
                        return Ok(guard.config.clone());
                    }
                }

                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        let config = if content.trim().is_empty() {
                            Config::default()
                        } else {
                            serde_yaml_ng::from_str(&content)
                                .with_context(|| format!("Failed to parse {}", path.display()))?
                        };

                        guard.config = config.clone();
                        guard.last_mtime = Some(modified);
                        guard.loaded_path = Some(path);
                        Ok(config)
                    }
                    Err(e) => {
                        Err(anyhow::Error::new(e)
                            .context(format!("Failed to read {}", path.display())))
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                guard.config = Config::default();
                guard.last_mtime = None;
                guard.loaded_path = Some(path);
                Ok(guard.config.clone())
            }
            Err(e) => Err(anyhow::Error::new(e)
                .context(format!("Failed to read metadata for {}", path.display()))),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path();
        let content = serde_yaml_ng::to_string(self)?;
        crate::core::atomic_io::atomic_write_str(&path, &content, 0o644)
            .with_context(|| format!("Failed to write {}", path.display()))?;
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

    pub async fn update_service_async<F>(service_name: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut Self, &str) -> bool,
    {
        let cache = CONFIG_CACHE.get_or_init(|| {
            RwLock::new(CachedConfig {
                config: Config::default(),
                last_mtime: None,
                loaded_path: None,
            })
        });

        let mut guard = cache.write().await;
        let path = Self::get_config_path();

        // Reload if stale before applying modification
        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            let modified = metadata.modified().unwrap_or_else(|_| SystemTime::now());
            if guard.last_mtime != Some(modified) {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    guard.config = if content.trim().is_empty() {
                        Config::default()
                    } else {
                        serde_yaml_ng::from_str(&content).unwrap_or_default()
                    };
                }
            }
        }

        let changed = update_fn(&mut guard.config, service_name);

        if changed {
            let content = serde_yaml_ng::to_string(&guard.config)?;
            crate::core::atomic_io::atomic_write_str(&path, &content, 0o644)?;
            if let Ok(metadata) = tokio::fs::metadata(&path).await {
                guard.last_mtime = Some(metadata.modified().unwrap_or_else(|_| SystemTime::now()));
            }
        }

        Ok(())
    }

    pub async fn enable_service_async(service_name: &str) -> Result<()> {
        Self::update_service_async(service_name, |config, name| {
            if config.disabled_services.remove(name) {
                info!("Enabled service: {}", name);
                true
            } else {
                false
            }
        })
        .await
    }

    pub async fn disable_service_async(service_name: &str) -> Result<()> {
        Self::update_service_async(service_name, |config, name| {
            if config.disabled_services.insert(name.to_string()) {
                info!("Disabled service: {}", name);
                true
            } else {
                false
            }
        })
        .await
    }
}
