pub mod apps;
pub mod arr;
pub mod download;
pub mod infra;
pub mod media;

use crate::core::hardware::HardwareInfo;
use crate::core::secrets::Secrets;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ServiceCategory {
    Infrastructure,
    Media,
    Automation,
    Download,
    Apps,
}

impl std::fmt::Display for ServiceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceCategory::Infrastructure => write!(f, "Infrastructure"),
            ServiceCategory::Media => write!(f, "Media"),
            ServiceCategory::Automation => write!(f, "Automation & Arr"),
            ServiceCategory::Download => write!(f, "Download"),
            ServiceCategory::Apps => write!(f, "Apps & Utility"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_ip: Option<String>,
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
}

impl PortMapping {
    pub fn parse(s: &str) -> Result<Self> {
        let (main_part, protocol) = if let Some((p, proto)) = s.rsplit_once('/') {
            let proto = match proto.to_ascii_lowercase().as_str() {
                "tcp" => Protocol::Tcp,
                "udp" => Protocol::Udp,
                other => anyhow::bail!("Unsupported protocol '{}' in port spec: {}", other, s),
            };
            (p, proto)
        } else {
            (s, Protocol::Tcp)
        };

        let parts: Vec<&str> = main_part.split(':').collect();
        match parts.len() {
            1 => {
                let port: u16 = parts[0]
                    .parse()
                    .with_context(|| format!("Invalid port number in '{}'", s))?;
                Ok(Self {
                    host_ip: None,
                    host_port: port,
                    container_port: port,
                    protocol,
                })
            }
            2 => {
                let host_port: u16 = parts[0]
                    .parse()
                    .with_context(|| format!("Invalid host port number in '{}'", s))?;
                let container_port: u16 = parts[1]
                    .parse()
                    .with_context(|| format!("Invalid container port number in '{}'", s))?;
                Ok(Self {
                    host_ip: None,
                    host_port,
                    container_port,
                    protocol,
                })
            }
            3 => {
                let host_ip = parts[0].to_string();
                let host_port: u16 = parts[1]
                    .parse()
                    .with_context(|| format!("Invalid host port number in '{}'", s))?;
                let container_port: u16 = parts[2]
                    .parse()
                    .with_context(|| format!("Invalid container port number in '{}'", s))?;
                Ok(Self {
                    host_ip: Some(host_ip),
                    host_port,
                    container_port,
                    protocol,
                })
            }
            _ => anyhow::bail!("Invalid port format: '{}'", s),
        }
    }

    pub fn is_localhost(&self) -> bool {
        self.host_ip.as_deref() == Some("127.0.0.1")
    }

    pub fn is_public(&self) -> bool {
        !self.is_localhost()
    }

    pub fn host_binding_str(&self) -> String {
        match &self.host_ip {
            Some(ip) => format!("{}:{}", ip, self.host_port),
            None => format!("0.0.0.0:{}", self.host_port),
        }
    }

    pub fn security_tier_str(&self) -> &'static str {
        if self.is_localhost() {
            "Localhost Only"
        } else {
            "Public"
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub memory_limit: Option<String>,
    pub memory_reservation: Option<String>,
    pub cpu_limit: Option<String>,
    pub cpu_reservation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub driver: String,
    pub options: HashMap<String, String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut options = HashMap::new();
        options.insert("max-size".to_string(), "10m".to_string());
        options.insert("max-file".to_string(), "3".to_string());
        Self {
            driver: "json-file".to_string(),
            options,
        }
    }
}

pub trait Service: Send + Sync {
    fn name(&self) -> &'static str;
    fn image(&self) -> &'static str;

    fn category(&self) -> ServiceCategory {
        ServiceCategory::Apps
    }

    fn description(&self) -> &'static str {
        ""
    }

    /// Generates configuration files (safe to run without side effects on system services)
    fn configure(&self, _hw: &HardwareInfo, _secrets: &Secrets) -> Result<()> {
        Ok(())
    }

    /// Performs system initialization (e.g., stopping conflicting services). May require root.
    fn initialize(&self, _hw: &HardwareInfo, _secrets: &Secrets) -> Result<()> {
        Ok(())
    }

    fn ports(&self) -> Vec<String> {
        vec![]
    }

    fn parsed_ports(&self) -> Vec<PortMapping> {
        self.ports()
            .iter()
            .filter_map(|p| PortMapping::parse(p).ok())
            .collect()
    }

    fn env_vars(&self, _hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
        HashMap::new()
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![]
    }
    fn networks(&self) -> Vec<String> {
        vec!["server_manager_net".to_string()]
    }
    fn devices(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![]
    }
    fn healthcheck(&self) -> Option<String> {
        None
    }
    fn depends_on(&self) -> Vec<String> {
        vec![]
    }
    fn security_opts(&self) -> Vec<String> {
        vec![]
    }
    fn labels(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    fn cap_add(&self) -> Vec<String> {
        vec![]
    }
    fn sysctls(&self) -> Vec<String> {
        vec![]
    }

    /// Returns resource limits/reservations based on hardware
    fn resources(&self, _hw: &HardwareInfo) -> Option<ResourceConfig> {
        None
    }

    /// Returns logging configuration
    fn logging(&self) -> LoggingConfig {
        LoggingConfig::default()
    }
}

pub fn get_all_services() -> &'static [Box<dyn Service>] {
    static SERVICES: std::sync::OnceLock<Vec<Box<dyn Service>>> = std::sync::OnceLock::new();
    SERVICES.get_or_init(|| {
        vec![
            Box::new(media::PlexService),
            Box::new(media::TautulliService),
            Box::new(media::OverseerrService),
            Box::new(media::JellyfinService),
            Box::new(media::JellyseerrService),
            Box::new(arr::SonarrService),
            Box::new(arr::RadarrService),
            Box::new(arr::ProwlarrService),
            Box::new(arr::JackettService),
            Box::new(arr::BazarrService),
            Box::new(download::QBittorrentService),
            Box::new(infra::MariaDBService),
            Box::new(infra::RedisService),
            Box::new(infra::NginxProxyService),
            Box::new(infra::DNSCryptService),
            Box::new(infra::WireguardService),
            Box::new(infra::PortainerService),
            Box::new(infra::NetdataService),
            Box::new(infra::UptimeKumaService),
            Box::new(apps::VaultwardenService),
            Box::new(apps::FilebrowserService),
            Box::new(apps::YourlsService),
            Box::new(apps::GLPIService),
            Box::new(apps::GiteaService),
            Box::new(apps::RoundcubeService),
            Box::new(apps::NextcloudService),
            Box::new(apps::MailService),
            Box::new(apps::SyncthingService),
        ]
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCatalogEntry {
    pub name: &'static str,
    pub image: &'static str,
    pub category: ServiceCategory,
    pub description: &'static str,
    pub ports: Vec<PortMapping>,
}

pub fn get_service_catalog() -> Vec<ServiceCatalogEntry> {
    get_all_services()
        .iter()
        .map(|s| ServiceCatalogEntry {
            name: s.name(),
            image: s.image(),
            category: s.category(),
            description: s.description(),
            ports: s.parsed_ports(),
        })
        .collect()
}

pub fn generate_port_matrix_markdown() -> String {
    let mut out = String::new();
    out.push_str("# Port Matrix — Cylae/server_script\n\n");
    out.push_str("## Authoritative Specification\n");
    out.push_str("This document defines the normative port allocation matrix for all services managed by `server_manager`.\n");
    out.push_str("This matrix is derived directly from the typed service catalog in `src/services/` and verified\n");
    out.push_str(
        "programmatically by contract tests (`server_manager/tests/contract_port_matrix.rs`).\n\n",
    );

    out.push_str("## Security Tiers\n");
    out.push_str("- **Localhost Only (`127.0.0.1`)**: Internal microservice web UIs and management APIs bound strictly to loopback to prevent direct external exposure without reverse proxy authentication.\n");
    out.push_str("- **Public (`0.0.0.0` / All Interfaces)**: Ingress services requiring external exposure (Reverse Proxy, VPN, DNS, Mail, Media streaming, Git SSH, Torrent peer discovery).\n");
    out.push_str("- **Internal Only (`None`)**: Databases and cache instances attached strictly to the Docker bridge network (`server_manager_net`) with no exposed host ports.\n\n");

    out.push_str("## Port Matrix Table\n\n");
    out.push_str("| Category | Service | Host Port | Container Port | Protocol | Host Binding | Security Tier | Description |\n");
    out.push_str("|:---|:---|:---|:---|:---|:---|:---|:---|\n");

    // Server Manager core web dashboard
    out.push_str("| Infrastructure | server_manager | 8099 | 8099 | TCP | 127.0.0.1:8099 | Localhost Only | Server Manager Core Web Administration Interface |\n");

    let catalog = get_service_catalog();
    for entry in &catalog {
        if entry.ports.is_empty() {
            out.push_str(&format!(
                "| {} | {} | - | - | - | - | Internal Only | {} |\n",
                entry.category, entry.name, entry.description
            ));
        } else {
            for port in &entry.ports {
                out.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                    entry.category,
                    entry.name,
                    port.host_port,
                    port.container_port,
                    port.protocol,
                    port.host_binding_str(),
                    port.security_tier_str(),
                    entry.description
                ));
            }
        }
    }

    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_service_registry() {
        let services = get_all_services();
        assert!(!services.is_empty());

        let names: Vec<&str> = services.iter().map(|s| s.name()).collect();
        assert!(names.contains(&"plex"));
        assert!(names.contains(&"sonarr"));
        assert!(names.contains(&"mariadb"));
    }
}
