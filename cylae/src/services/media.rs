use super::Service;
use crate::core::hardware::{HardwareInfo, HardwareProfile};
use crate::core::secrets::Secrets;
use std::collections::HashMap;

pub struct PlexService;

impl Service for PlexService {
    fn name(&self) -> &'static str { "plex" }
    fn image(&self) -> &'static str { "lscr.io/linuxserver/plex:latest" }

    fn ports(&self) -> Vec<String> {
        vec!["32400:32400".to_string()]
    }

    fn env_vars(&self, hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PUID".to_string(), "1000".to_string());
        vars.insert("PGID".to_string(), "1000".to_string());
        vars.insert("VERSION".to_string(), "docker".to_string());

        // Legacy requirement
        vars.insert("PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS".to_string(), "2".to_string());

        if hw.has_nvidia {
             vars.insert("NVIDIA_VISIBLE_DEVICES".to_string(), "all".to_string());
             vars.insert("NVIDIA_DRIVER_CAPABILITIES".to_string(), "compute,video,utility".to_string());
        }

        vars
    }

    fn volumes(&self, hw: &HardwareInfo) -> Vec<String> {
        let mut vols = vec![
            "./config/plex:/config".to_string(),
            "./media/tv:/tv".to_string(),
            "./media/movies:/movies".to_string(),
        ];

        // Optimization: RAM Transcoding for High Profile
        match hw.profile {
            HardwareProfile::High => vols.push("/dev/shm:/transcode".to_string()),
            _ => vols.push("./transcode:/transcode".to_string()),
        }

        vols
    }

    fn devices(&self, hw: &HardwareInfo) -> Vec<String> {
        if hw.has_intel_quicksync {
            return vec!["/dev/dri:/dev/dri".to_string()];
        }
        vec![]
    }

    fn healthcheck(&self) -> Option<String> {
        Some("curl -f http://localhost:32400/identity || exit 1".to_string())
    }
}
