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
    pub fn load() -> Result<Self> {
        let path = Path::new("config.yaml");
        let fallback_path = Path::new("/opt/server_manager/config.yaml");

        let load_path = if path.exists() {
            Some(path)
        } else if fallback_path.exists() {
            Some(fallback_path)
        } else {
            None
        };

        if let Some(p) = load_path {
            let content = fs::read_to_string(p).context("Failed to read config.yaml")?;
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
                loaded_path: None,
            })
        });

        // Determine current priority path
        let path = Path::new("config.yaml");
        let fallback_path = Path::new("/opt/server_manager/config.yaml");

        // Check for existence asynchronously
        let current_path = if tokio::fs::try_exists(path).await.unwrap_or(false) {
            Some(path.to_path_buf())
        } else if tokio::fs::try_exists(fallback_path).await.unwrap_or(false) {
            Some(fallback_path.to_path_buf())
        } else {
            None
        };

        // Fast path: Optimistic read
        {
            let guard = cache.read().await;
            if let Some(p) = &current_path {
                // If we are looking at the same file as cached
                if guard.loaded_path.as_ref() == Some(p) {
                    if let Some(cached_mtime) = guard.last_mtime {
                        // Check if file still matches
                        if let Ok(metadata) = tokio::fs::metadata(p).await {
                            if let Ok(modified) = metadata.modified() {
                                if modified == cached_mtime {
                                    return Ok(guard.config.clone());
                                }
                            }
                        }
                    }
                }
            } else if guard.loaded_path.is_none() {
                // No file exists and cache has no file -> return default
                return Ok(guard.config.clone());
            }
        }

        // Slow path: Update cache
        let mut guard = cache.write().await;

        // Re-evaluate path under lock (though unlikely to race in this context, good practice)
        let current_path_2 = if tokio::fs::try_exists(path).await.unwrap_or(false) {
            Some(path.to_path_buf())
        } else if tokio::fs::try_exists(fallback_path).await.unwrap_or(false) {
            Some(fallback_path.to_path_buf())
        } else {
            None
        };

        if let Some(p) = current_path_2 {
            let metadata_res = tokio::fs::metadata(&p).await;
            match metadata_res {
                Ok(metadata) => {
                    let modified = metadata.modified().unwrap_or(SystemTime::now());

                    // Check if cache is already up to date for this path
                    if guard.loaded_path.as_ref() == Some(&p) {
                         if let Some(cached_mtime) = guard.last_mtime {
                            if modified == cached_mtime {
                                return Ok(guard.config.clone());
                            }
                        }
                    }

                    // Load file
                    match tokio::fs::read_to_string(&p).await {
                        Ok(content) => {
                            let config = if content.trim().is_empty() {
                                Config::default()
                            } else {
                                serde_yaml_ng::from_str(&content)
                                    .context("Failed to parse config.yaml")?
                            };

                            guard.config = config.clone();
                            guard.last_mtime = Some(modified);
                            guard.loaded_path = Some(p);
                            Ok(config)
                        }
                        Err(e) => Err(anyhow::Error::new(e).context("Failed to read config.yaml")),
                    }
                }
                Err(e) => Err(anyhow::Error::new(e).context("Failed to read config metadata")),
            }
        } else {
            // File not found -> Default
            guard.config = Config::default();
            guard.last_mtime = None;
            guard.loaded_path = None;
            Ok(guard.config.clone())
        }
    }

    pub fn save(&self) -> Result<()> {
        // Prefer saving to /opt/server_manager if it exists/is writable (checked by parent dir existence), else CWD
        let target = if Path::new("/opt/server_manager").exists() {
            Path::new("/opt/server_manager/config.yaml")
        } else {
            Path::new("config.yaml")
        };

        let content = serde_yaml_ng::to_string(self)?;
        fs::write(target, content).context("Failed to write config.yaml")?;

        // Invalidate cache implicitly?
        // Ideally we should update the cache here or invalidate it.
        // Since the cache checks mtime, it will pick up the change on next read.
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
