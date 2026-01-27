use super::{Service, ResourceConfig};
use crate::core::hardware::{HardwareInfo, HardwareProfile};
use crate::core::secrets::Secrets;
use std::collections::HashMap;

macro_rules! define_arr_service {
    ($struct_name:ident, $name:expr, $image:expr, $port:expr) => {
        pub struct $struct_name;
        impl Service for $struct_name {
            fn name(&self) -> &'static str { $name }
            fn image(&self) -> &'static str { $image }
            fn ports(&self) -> Vec<String> { vec![format!("{}:{}", $port, $port)] }
            fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
                vec![
                    format!("./config/{}:/config", $name),
                    "./media:/media".to_string(),
                ]
            }
            fn env_vars(&self, hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
                let mut vars = HashMap::new();
                vars.insert("PUID".to_string(), hw.user_id.clone());
                vars.insert("PGID".to_string(), hw.group_id.clone());
                vars.insert("COMPlus_EnableDiagnostics".to_string(), "0".to_string());

                if let HardwareProfile::Low = hw.profile {
                    vars.insert("COMPlus_GCServer".to_string(), "0".to_string());
                }
                vars
            }
            fn healthcheck(&self) -> Option<String> {
                Some(format!("curl -f http://localhost:{}/ping || exit 1", $port))
            }

            fn resources(&self, hw: &HardwareInfo) -> Option<ResourceConfig> {
                 let memory_limit = match hw.profile {
                    HardwareProfile::High => "2G",
                    _ => "1G",
                };
                Some(ResourceConfig {
                    memory_limit: Some(memory_limit.to_string()),
                    memory_reservation: None,
                    cpu_limit: None,
                    cpu_reservation: None,
                })
            }
        }
    };
}

define_arr_service!(SonarrService, "sonarr", "lscr.io/linuxserver/sonarr:latest", 8989);
define_arr_service!(RadarrService, "radarr", "lscr.io/linuxserver/radarr:latest", 7878);
define_arr_service!(ProwlarrService, "prowlarr", "lscr.io/linuxserver/prowlarr:latest", 9696);
define_arr_service!(JackettService, "jackett", "lscr.io/linuxserver/jackett:latest", 9117);
