use super::Service;
use crate::core::hardware::{HardwareInfo};
use crate::core::secrets::Secrets;
use std::collections::HashMap;

pub struct QBittorrentService;

impl Service for QBittorrentService {
    fn name(&self) -> &'static str { "qbittorrent" }
    fn image(&self) -> &'static str { "lscr.io/linuxserver/qbittorrent:latest" }

    fn ports(&self) -> Vec<String> {
        vec!["8080:8080".to_string(), "6881:6881".to_string(), "6881:6881/udp".to_string()]
    }

    fn env_vars(&self, hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PUID".to_string(), hw.user_id.clone());
        vars.insert("PGID".to_string(), hw.group_id.clone());
        vars.insert("WEBUI_PORT".to_string(), "8080".to_string());
        vars
    }

    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![
            "./config/qbittorrent:/config".to_string(),
            "./media/downloads:/downloads".to_string(),
        ]
    }
}
