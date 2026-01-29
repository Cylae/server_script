use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use log::info;

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
            serde_yaml::from_str(&content).context("Failed to parse config.yaml")
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
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
