use async_trait::async_trait;
use serde_yaml::Value;
use anyhow::Result;
use std::collections::HashMap;
use crate::services::base::Service;
use crate::core::hardware::{HardwareManager, HardwareProfile};

pub struct PlexService;

#[async_trait]
impl Service for PlexService {
    fn name(&self) -> &'static str {
        "plex"
    }

    fn pretty_name(&self) -> &'static str {
        "Plex Media Server"
    }

    async fn is_installed(&self) -> Result<bool> {
        // Stub for now or use DockerManager if we fixed it
        // let docker = crate::core::docker::DockerManager::new()?;
        // docker.is_installed(self.name()).await
        Ok(false)
    }

    async fn install(&self) -> Result<()> {
        println!("Installing Plex...");
        let compose = self.generate_compose();
        println!("Generated Compose: {:?}", compose);
        Ok(())
    }

    async fn wait_for_health(&self, _retries: u32, _delay: u64) -> Result<bool> {
        Ok(true)
    }

    async fn remove(&self) -> Result<()> {
         Ok(())
    }

    fn get_url(&self) -> Option<String> {
        Some("http://localhost:32400/web".to_string())
    }

    fn get_ports(&self) -> Vec<String> {
        vec!["32400/tcp".to_string()]
    }

    fn generate_compose(&self) -> Value {
        let profile = HardwareManager::get_hardware_profile();
        let is_low_spec = matches!(profile, HardwareProfile::Low);

        let transcode_vol = if !is_low_spec {
            "/tmp:/transcode"
        } else {
            "/opt/cylae/data/plex/transcode:/transcode"
        };

        let max_plugin_procs = if !is_low_spec { "6" } else { "2" };

        let mut env = HashMap::new();
        // Mocking get_common_env
        env.insert("PUID".to_string(), "1000".to_string());
        env.insert("PGID".to_string(), "1000".to_string());
        env.insert("TZ".to_string(), "UTC".to_string());
        env.insert("VERSION".to_string(), "docker".to_string());
        env.insert("PLEX_CLAIM".to_string(), "claim-TOKEN".to_string());
        env.insert("PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS".to_string(), max_plugin_procs.to_string());

        let (mem_limit, cpu_limit) = if !is_low_spec {
             ("8G", "4.0")
        } else {
             ("2G", "1.0")
        };

        serde_json::from_value(serde_json::json!({
            "services": {
                "plex": {
                    "image": "lscr.io/linuxserver/plex:latest",
                    "container_name": "plex",
                    "restart": "unless-stopped",
                    "network_mode": "host",
                    "logging": {
                        "driver": "json-file",
                        "options": {
                            "max-size": "10m",
                            "max-file": "3"
                        }
                    },
                    "environment": env,
                    "volumes": [
                        "/opt/cylae/data/plex:/config",
                        "/media/storage:/media",
                        transcode_vol
                    ],
                    "deploy": {
                        "resources": {
                            "limits": {
                                "memory": mem_limit,
                                "cpus": cpu_limit
                            }
                        }
                    }
                }
            }
        })).unwrap()
    }
}
