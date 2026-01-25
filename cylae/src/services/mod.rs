pub mod media;
pub mod arr;
pub mod infra;
pub mod download;
pub mod apps;

use crate::core::hardware::HardwareInfo;
use crate::core::secrets::Secrets;
use std::collections::HashMap;

pub trait Service: Send + Sync {
    fn name(&self) -> &'static str;
    fn image(&self) -> &'static str;

    fn ports(&self) -> Vec<String> { vec![] }
    fn env_vars(&self, _hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> { HashMap::new() }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> { vec![] }
    fn networks(&self) -> Vec<String> { vec!["cylae_net".to_string()] }
    fn devices(&self, _hw: &HardwareInfo) -> Vec<String> { vec![] }
    fn healthcheck(&self) -> Option<String> { None }
    fn depends_on(&self) -> Vec<String> { vec![] }
    fn security_opts(&self) -> Vec<String> { vec![] }
    fn labels(&self) -> HashMap<String, String> { HashMap::new() }
    fn cap_add(&self) -> Vec<String> { vec![] }
    fn sysctls(&self) -> Vec<String> { vec![] }
}

pub fn get_all_services() -> Vec<Box<dyn Service>> {
    vec![
        Box::new(media::PlexService),
        Box::new(media::TautulliService),
        Box::new(media::OverseerrService),
        Box::new(arr::SonarrService),
        Box::new(arr::RadarrService),
        Box::new(arr::ProwlarrService),
        Box::new(arr::JackettService),
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
    ]
}

#[cfg(test)]
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
