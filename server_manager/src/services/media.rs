use super::{ResourceConfig, Service};
use crate::core::hardware::{HardwareInfo, HardwareProfile};
use crate::core::secrets::Secrets;
use std::collections::HashMap;

pub struct PlexService;

impl Service for PlexService {
    fn name(&self) -> &'static str {
        "plex"
    }
    fn image(&self) -> &'static str {
        "lscr.io/linuxserver/plex:latest"
    }

    fn ports(&self) -> Vec<String> {
        vec!["32400:32400".to_string()]
    }

    fn env_vars(&self, hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PUID".to_string(), hw.user_id.clone());
        vars.insert("PGID".to_string(), hw.group_id.clone());
        vars.insert("VERSION".to_string(), "docker".to_string());

        // Legacy requirement
        vars.insert(
            "PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS".to_string(),
            "2".to_string(),
        );

        if hw.has_nvidia {
            vars.insert("NVIDIA_VISIBLE_DEVICES".to_string(), "all".to_string());
            vars.insert(
                "NVIDIA_DRIVER_CAPABILITIES".to_string(),
                "compute,video,utility".to_string(),
            );
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

    fn resources(&self, hw: &HardwareInfo) -> Option<ResourceConfig> {
        let memory_limit = match hw.profile {
            HardwareProfile::High => "8G",
            HardwareProfile::Standard => "4G",
            HardwareProfile::Low => "2G",
        };
        Some(ResourceConfig {
            memory_limit: Some(memory_limit.to_string()),
            memory_reservation: None,
            cpu_limit: None,
            cpu_reservation: None,
        })
    }
}

pub struct JellyfinService;

impl Service for JellyfinService {
    fn name(&self) -> &'static str {
        "jellyfin"
    }
    fn image(&self) -> &'static str {
        "lscr.io/linuxserver/jellyfin:latest"
    }

    fn ports(&self) -> Vec<String> {
        vec!["8096:8096".to_string()]
    }

    fn env_vars(&self, hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PUID".to_string(), hw.user_id.clone());
        vars.insert("PGID".to_string(), hw.group_id.clone());

        if hw.has_nvidia {
            vars.insert("NVIDIA_VISIBLE_DEVICES".to_string(), "all".to_string());
            vars.insert(
                "NVIDIA_DRIVER_CAPABILITIES".to_string(),
                "compute,video,utility".to_string(),
            );
        }

        vars
    }

    fn volumes(&self, hw: &HardwareInfo) -> Vec<String> {
        let mut vols = vec![
            "./config/jellyfin:/config".to_string(),
            "./media/tv:/data/tvshows".to_string(),
            "./media/movies:/data/movies".to_string(),
        ];

        // Optimization: RAM Transcoding for High Profile
        match hw.profile {
            HardwareProfile::High => vols.push("/dev/shm:/transcode".to_string()),
            _ => vols.push("./transcode_jellyfin:/transcode".to_string()),
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
        Some("curl -f http://localhost:8096/health || exit 1".to_string())
    }

    fn resources(&self, hw: &HardwareInfo) -> Option<ResourceConfig> {
        let memory_limit = match hw.profile {
            HardwareProfile::High => "8G",
            HardwareProfile::Standard => "4G",
            HardwareProfile::Low => "2G",
        };
        Some(ResourceConfig {
            memory_limit: Some(memory_limit.to_string()),
            memory_reservation: None,
            cpu_limit: None,
            cpu_reservation: None,
        })
    }
}

pub struct JellyseerrService;
impl Service for JellyseerrService {
    fn name(&self) -> &'static str {
        "jellyseerr"
    }
    fn image(&self) -> &'static str {
        "fallenbagel/jellyseerr:latest"
    }
    // Internal port 5055, exposed as 5056 to avoid conflict with Overseerr
    fn ports(&self) -> Vec<String> {
        vec!["127.0.0.1:5056:5055".to_string()]
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/jellyseerr:/app/config".to_string()]
    }
    fn resources(&self, _hw: &HardwareInfo) -> Option<ResourceConfig> {
        Some(ResourceConfig {
            memory_limit: Some("1G".to_string()),
            memory_reservation: None,
            cpu_limit: None,
            cpu_reservation: None,
        })
    }
}

pub struct TautulliService;
impl Service for TautulliService {
    fn name(&self) -> &'static str {
        "tautulli"
    }
    fn image(&self) -> &'static str {
        "lscr.io/linuxserver/tautulli:latest"
    }
    fn ports(&self) -> Vec<String> {
        vec!["127.0.0.1:8181:8181".to_string()]
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/tautulli:/config".to_string()]
    }
    fn depends_on(&self) -> Vec<String> {
        vec!["plex".to_string()]
    }
    fn resources(&self, _hw: &HardwareInfo) -> Option<ResourceConfig> {
        Some(ResourceConfig {
            memory_limit: Some("512M".to_string()),
            memory_reservation: None,
            cpu_limit: None,
            cpu_reservation: None,
        })
    }
}

pub struct OverseerrService;
impl Service for OverseerrService {
    fn name(&self) -> &'static str {
        "overseerr"
    }
    fn image(&self) -> &'static str {
        "lscr.io/linuxserver/overseerr:latest"
    }
    fn ports(&self) -> Vec<String> {
        vec!["127.0.0.1:5055:5055".to_string()]
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/overseerr:/config".to_string()]
    }
    fn resources(&self, _hw: &HardwareInfo) -> Option<ResourceConfig> {
        Some(ResourceConfig {
            memory_limit: Some("1G".to_string()),
            memory_reservation: None,
            cpu_limit: None,
            cpu_reservation: None,
        })
    }
}
