pub mod core;
pub mod services;

// Re-export build_compose_structure from main (we need to move it or expose it)
// Since main.rs is a binary, we can't easily import from it in integration tests unless we use a lib.rs structure.
// The common pattern is to have the logic in lib.rs and main.rs just calls it.
// I will move build_compose_structure to lib.rs or make main.rs just a thin wrapper.

// Wait, I can't import functions from main.rs into integration tests directly if it's not a lib.
// I need to refactor slightly.
// Move `build_compose_structure` to a new module `src/orchestrator.rs` or keep it in `lib.rs`.

// Let's check where `main.rs` is. It is in `src/main.rs`.
// I should create `src/lib.rs` that exposes the modules and the orchestrator logic.

// Currently `main.rs` has `mod core; mod services;`.
// I will move these to `lib.rs` and make `main.rs` use the lib.

pub use crate::core::hardware;
pub use crate::core::secrets;

use anyhow::Result;
use std::collections::HashMap;

// I will duplicate the logic or move it. Moving is better.
// I'll put build_compose_structure here.

pub fn build_compose_structure(hw: &hardware::HardwareInfo, secrets: &secrets::Secrets) -> Result<serde_yaml::Mapping> {
    let services = services::get_all_services();

    let mut compose_services = HashMap::new();

    for service in services {
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
