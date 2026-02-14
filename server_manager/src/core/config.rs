use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedConfig {
    config: Config,
    last_mtime: Option<SystemTime>,
    last_check: SystemTime,
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
                last_check: SystemTime::UNIX_EPOCH,
            })
        });

        let now = SystemTime::now();

        // 1. Highly optimistic path: if we checked recently, return cache
        // We throttle even if the file was missing (last_mtime is None)
        {
            let guard = cache.read().await;
            if now.duration_since(guard.last_check).unwrap_or_default() < Duration::from_millis(500) {
                return Ok(guard.config.clone());
            }
        }

        // 2. Perform metadata check (outside of lock to minimize contention)
        let metadata_res = tokio::fs::metadata("config.yaml").await;
        let mtime = match &metadata_res {
            Ok(m) => Some(m.modified().unwrap_or(SystemTime::now())),
            Err(_) => None,
        };

        // 3. Update cache if needed
        let mut guard = cache.write().await;

        // If file hasn't changed, just update last_check and return
        if guard.last_mtime == mtime {
            guard.last_check = now;
            return Ok(guard.config.clone());
        }

        // Actually reload
        match metadata_res {
            Ok(_) => match tokio::fs::read_to_string("config.yaml").await {
                Ok(content) => {
                    let config_res: Result<Config> = if content.trim().is_empty() {
                        Ok(Config::default())
                    } else {
                        serde_yaml_ng::from_str(&content).map_err(|e| anyhow::anyhow!(e))
                    };

                    match config_res {
                        Ok(config) => {
                            guard.config = config.clone();
                            guard.last_mtime = mtime;
                            guard.last_check = now;
                            Ok(config)
                        }
                        Err(e) => {
                            // On parse error, keep the old config but update last_check to prevent spamming
                            guard.last_check = now;
                            Err(e.context("Failed to parse config.yaml"))
                        }
                    }
                }
                Err(e) => {
                    guard.last_check = now;
                    Err(anyhow::Error::new(e).context("Failed to read config.yaml"))
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File not found -> Default
                guard.config = Config::default();
                guard.last_mtime = None;
                guard.last_check = now;
                Ok(guard.config.clone())
            }
            Err(e) => {
                guard.last_check = now;
                Err(anyhow::Error::new(e).context("Failed to read config metadata"))
            }
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
