pub mod core;
pub mod services;

pub use crate::core::hardware;
pub use crate::core::secrets;
pub use crate::core::config;
pub use crate::core::users;
pub mod interface;

use anyhow::Result;
use std::collections::HashMap;

/// Generates the docker-compose.yml structure based on hardware profile and secrets.
/// This acts as the "Compiler" for the infrastructure.
pub fn build_compose_structure(hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<serde_yaml::Mapping> {
    let services = services::get_all_services();

    let mut compose_services = HashMap::new();

    for service in services {
        if !config.is_enabled(service.name()) {
            continue;
        }

        let mut config = serde_yaml::Mapping::new();
        config.insert("image".into(), service.image().into());
        config.insert("container_name".into(), service.name().into());
        config.insert("restart".into(), "unless-stopped".into());

        // Ports
        let ports = service.ports();
        if !ports.is_empty() {
             let mut ports_seq = serde_yaml::Sequence::new();
             for p in ports {
                 ports_seq.push(p.into());
             }
             config.insert("ports".into(), serde_yaml::Value::Sequence(ports_seq));
        }

        // Env Vars
        let envs = service.env_vars(hw, secrets);
        if !envs.is_empty() {
             let mut env_seq = serde_yaml::Sequence::new();
             for (k, v) in envs {
                 env_seq.push(format!("{}={}", k, v).into());
             }
             config.insert("environment".into(), serde_yaml::Value::Sequence(env_seq));
        }

        // Volumes
        let vols = service.volumes(hw);
        if !vols.is_empty() {
             let mut vol_seq = serde_yaml::Sequence::new();
             for v in vols {
                 vol_seq.push(v.into());
             }
             config.insert("volumes".into(), serde_yaml::Value::Sequence(vol_seq));
        }

        // Devices
        let devs = service.devices(hw);
        if !devs.is_empty() {
             let mut dev_seq = serde_yaml::Sequence::new();
             for d in devs {
                 dev_seq.push(d.into());
             }
             config.insert("devices".into(), serde_yaml::Value::Sequence(dev_seq));
        }

        // Networks
        let nets = service.networks();
        if !nets.is_empty() {
             let mut net_seq = serde_yaml::Sequence::new();
             for n in nets {
                 net_seq.push(n.into());
             }
             config.insert("networks".into(), serde_yaml::Value::Sequence(net_seq));
        }

        // Healthcheck
        if let Some(cmd) = service.healthcheck() {
             let mut hc = serde_yaml::Mapping::new();
             let mut test_seq = serde_yaml::Sequence::new();
             test_seq.push("CMD-SHELL".into());
             test_seq.push(cmd.into());

             hc.insert("test".into(), serde_yaml::Value::Sequence(test_seq));
             hc.insert("interval".into(), "1m".into());
             hc.insert("retries".into(), 3.into());
             hc.insert("start_period".into(), "30s".into());
             hc.insert("timeout".into(), "10s".into());

             config.insert("healthcheck".into(), serde_yaml::Value::Mapping(hc));
        }

        // Depends On
        let deps = service.depends_on();
        if !deps.is_empty() {
             let mut dep_seq = serde_yaml::Sequence::new();
             for d in deps {
                 dep_seq.push(d.into());
             }
             config.insert("depends_on".into(), serde_yaml::Value::Sequence(dep_seq));
        }

        // Security Opts
        let sec_opts = service.security_opts();
        if !sec_opts.is_empty() {
             let mut sec_seq = serde_yaml::Sequence::new();
             for s in sec_opts {
                 sec_seq.push(s.into());
             }
             config.insert("security_opt".into(), serde_yaml::Value::Sequence(sec_seq));
        }

        // Labels
        let labels = service.labels();
        if !labels.is_empty() {
             let mut labels_seq = serde_yaml::Sequence::new();
             for (k, v) in labels {
                 labels_seq.push(format!("{}={}", k, v).into());
             }
             config.insert("labels".into(), serde_yaml::Value::Sequence(labels_seq));
        }

        // Cap Add
        let caps = service.cap_add();
        if !caps.is_empty() {
             let mut cap_seq = serde_yaml::Sequence::new();
             for c in caps {
                 cap_seq.push(c.into());
             }
             config.insert("cap_add".into(), serde_yaml::Value::Sequence(cap_seq));
        }

        // Sysctls
        let sysctls = service.sysctls();
        if !sysctls.is_empty() {
             let mut sys_seq = serde_yaml::Sequence::new();
             for s in sysctls {
                 sys_seq.push(s.into());
             }
             config.insert("sysctls".into(), serde_yaml::Value::Sequence(sys_seq));
        }

        // Resources (Deploy)
        if let Some(res) = service.resources(hw) {
            let mut deploy = serde_yaml::Mapping::new();
            let mut resources = serde_yaml::Mapping::new();
            let mut limits = serde_yaml::Mapping::new();
            let mut reservations = serde_yaml::Mapping::new();

            if let Some(mem) = res.memory_limit {
                limits.insert("memory".into(), mem.into());
            }
            if let Some(cpu) = res.cpu_limit {
                limits.insert("cpus".into(), cpu.into());
            }

            if let Some(mem) = res.memory_reservation {
                reservations.insert("memory".into(), mem.into());
            }
            if let Some(cpu) = res.cpu_reservation {
                reservations.insert("cpus".into(), cpu.into());
            }

            if !limits.is_empty() {
                resources.insert("limits".into(), serde_yaml::Value::Mapping(limits));
            }
            if !reservations.is_empty() {
                resources.insert("reservations".into(), serde_yaml::Value::Mapping(reservations));
            }

            if !resources.is_empty() {
                deploy.insert("resources".into(), serde_yaml::Value::Mapping(resources));
                config.insert("deploy".into(), serde_yaml::Value::Mapping(deploy));
            }
        }

        // Logging
        let logging = service.logging();
        let mut logging_map = serde_yaml::Mapping::new();
        logging_map.insert("driver".into(), logging.driver.into());

        let mut opts_map = serde_yaml::Mapping::new();
        for (k, v) in logging.options {
            opts_map.insert(k.into(), v.into());
        }
        if !opts_map.is_empty() {
             logging_map.insert("options".into(), serde_yaml::Value::Mapping(opts_map));
        }
        config.insert("logging".into(), serde_yaml::Value::Mapping(logging_map));

        compose_services.insert(service.name().to_string(), serde_yaml::Value::Mapping(config));
    }

    // Networks definition
    let mut networks_map = serde_yaml::Mapping::new();
    let mut server_manager_net = serde_yaml::Mapping::new();
    server_manager_net.insert("driver".into(), "bridge".into());
    networks_map.insert("server_manager_net".into(), serde_yaml::Value::Mapping(server_manager_net));

    // Top Level
    let mut top_level = serde_yaml::Mapping::new();
    top_level.insert("services".into(), serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(
        compose_services.into_iter().map(|(k, v)| (k.into(), v))
    )));
    top_level.insert("networks".into(), serde_yaml::Value::Mapping(networks_map));

    Ok(top_level)
}
