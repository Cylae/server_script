use async_trait::async_trait;
use crate::services::Service;
use crate::core::hardware::{HardwareProfile, HardwareManager, GpuType};
use crate::core::system::SystemManager;
use crate::core::firewall::FirewallManager;
use anyhow::Result;
use serde::{Serialize};
use std::collections::HashMap;
use log::{info};

pub struct PlexService;

#[derive(Serialize)]
struct DockerCompose {
    services: HashMap<String, ServiceConfig>,
}

#[derive(Serialize)]
struct ServiceConfig {
    image: String,
    container_name: String,
    restart: String,
    network_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logging: Option<LoggingConfig>,
    environment: HashMap<String, String>,
    volumes: Vec<String>,
    deploy: DeployConfig,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    devices: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime: Option<String>,
}

#[derive(Serialize)]
struct LoggingConfig {
    driver: String,
    options: HashMap<String, String>,
}

#[derive(Serialize)]
struct DeployConfig {
    resources: Resources,
}

#[derive(Serialize)]
struct Resources {
    limits: ResourceLimits,
}

#[derive(Serialize)]
struct ResourceLimits {
    memory: String,
    cpus: String,
}

#[async_trait]
impl Service for PlexService {
    fn name(&self) -> &'static str { "plex" }
    fn pretty_name(&self) -> &'static str { "Plex Media Server" }

    fn get_url(&self) -> Option<String> {
        Some("http://localhost:32400/web".to_string())
    }

    fn firewall_ports(&self) -> Vec<String> {
        vec!["32400/tcp".to_string()]
    }

    async fn install(&self, profile: &HardwareProfile) -> Result<()> {
        info!("Installing Plex Media Server...");

        // 1. Firewall
        for port in self.firewall_ports() {
            FirewallManager::allow_port(&port, "Plex");
        }

        // 2. Generate Compose
        let compose_content = self.generate_compose(profile);

        // 3. Write Compose File
        let data_dir = "/var/lib/cylae"; // hardcoded for now
        let service_dir = format!("{}/compose/plex", data_dir);
        std::fs::create_dir_all(&service_dir)?;

        let compose_path = format!("{}/docker-compose.yml", service_dir);
        let f = std::fs::File::create(&compose_path)?;
        serde_yaml::to_writer(f, &compose_content)?;

        // 4. Deploy
        SystemManager::run_command("docker", &["compose", "-f", &compose_path, "up", "-d"])?;

        info!("Plex installed successfully.");
        Ok(())
    }
}

impl PlexService {
    fn generate_compose(&self, profile: &HardwareProfile) -> DockerCompose {
        let is_low_spec = matches!(profile, HardwareProfile::Low);

        // Transcoding
        let transcode_vol = if !is_low_spec {
            "/tmp:/transcode".to_string() // RAM
        } else {
            "/var/lib/cylae/plex/transcode:/transcode".to_string() // Disk
        };

        // GPU
        let gpu_info = HardwareManager::detect_gpu();
        let mut devices = vec![];
        let mut runtime = None;
        let mut extra_env = HashMap::new();

        match gpu_info.gpu_type {
            GpuType::Intel => {
                devices.push("/dev/dri:/dev/dri".to_string());
            }
            GpuType::Nvidia => {
                runtime = Some("nvidia".to_string());
                extra_env.insert("NVIDIA_VISIBLE_DEVICES".to_string(), "all".to_string());
                extra_env.insert("NVIDIA_DRIVER_CAPABILITIES".to_string(), "compute,video,utility".to_string());
            }
            _ => {}
        }

        let mut env = HashMap::new();
        env.insert("VERSION".to_string(), "docker".to_string());
        env.insert("PLEX_CLAIM".to_string(), "claim-TOKEN".to_string());
        env.insert("PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS".to_string(), if !is_low_spec { "6".to_string() } else { "2".to_string() });

        // Merge extra_env
        for (k, v) in extra_env {
            env.insert(k, v);
        }

        // Common env
        let (uid, gid) = SystemManager::get_uid_gid();
        env.insert("PUID".to_string(), uid.to_string());
        env.insert("PGID".to_string(), gid.to_string());
        env.insert("TZ".to_string(), SystemManager::get_timezone());

        let logging = Some(LoggingConfig {
            driver: "json-file".to_string(),
            options: HashMap::from([
                ("max-size".to_string(), "10m".to_string()),
                ("max-file".to_string(), "3".to_string()),
            ]),
        });

        let resources = if !is_low_spec {
             ResourceLimits { memory: "8G".to_string(), cpus: "4.0".to_string() }
        } else {
             ResourceLimits { memory: "2G".to_string(), cpus: "1.0".to_string() }
        };

        let service_config = ServiceConfig {
            image: "lscr.io/linuxserver/plex:latest".to_string(),
            container_name: "plex".to_string(),
            restart: "unless-stopped".to_string(),
            network_mode: "host".to_string(),
            logging,
            environment: env,
            volumes: vec![
                "/var/lib/cylae/plex:/config".to_string(),
                "/media:/media".to_string(), // Simplified media root
                transcode_vol
            ],
            deploy: DeployConfig {
                resources: Resources { limits: resources }
            },
            devices,
            runtime,
        };

        let mut services = HashMap::new();
        services.insert("plex".to_string(), service_config);

        DockerCompose { services }
    }
}
