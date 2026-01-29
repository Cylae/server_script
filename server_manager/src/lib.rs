pub mod core;
pub mod services;

pub use crate::core::hardware;
pub use crate::core::secrets;
pub use crate::core::config;
pub use crate::core::users;
// Import the new module structs
use crate::core::compose::{ComposeFile, Service, Network, Logging, HealthCheck, Deploy, Resources, ResourceLimits};

pub mod interface;

use anyhow::Result;
use std::collections::HashMap;

/// Generates the docker-compose.yml structure based on hardware profile and secrets.
/// This acts as the "Compiler" for the infrastructure.
pub fn build_compose_structure(hw: &hardware::HardwareInfo, secrets: &secrets::Secrets, config: &config::Config) -> Result<ComposeFile> {
    let services_list = services::get_all_services();
    let mut compose_services = HashMap::new();

    for service_impl in services_list {
        if !config.is_enabled(service_impl.name()) {
            continue;
        }

        let name = service_impl.name().to_string();

        let ports = service_impl.ports();
        let ports = if ports.is_empty() { None } else { Some(ports) };

        let envs = service_impl.env_vars(hw, secrets);
        let environment = if envs.is_empty() { None } else {
            Some(envs.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect())
        };

        let vols = service_impl.volumes(hw);
        let volumes = if vols.is_empty() { None } else { Some(vols) };

        let devs = service_impl.devices(hw);
        let devices = if devs.is_empty() { None } else { Some(devs) };

        let nets = service_impl.networks();
        let networks = if nets.is_empty() { None } else { Some(nets) };

        let healthcheck = if let Some(cmd) = service_impl.healthcheck() {
            Some(HealthCheck {
                test: vec!["CMD-SHELL".to_string(), cmd],
                interval: "1m".to_string(),
                retries: 3,
                start_period: "30s".to_string(),
                timeout: "10s".to_string(),
            })
        } else {
            None
        };

        let deps = service_impl.depends_on();
        let depends_on = if deps.is_empty() { None } else { Some(deps) };

        let sec_opts = service_impl.security_opts();
        let security_opt = if sec_opts.is_empty() { None } else { Some(sec_opts) };

        let labels_map = service_impl.labels();
        let labels = if labels_map.is_empty() { None } else {
            Some(labels_map.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect())
        };

        let caps = service_impl.cap_add();
        let cap_add = if caps.is_empty() { None } else { Some(caps) };

        let sys = service_impl.sysctls();
        let sysctls = if sys.is_empty() { None } else { Some(sys) };

        let deploy = if let Some(res) = service_impl.resources(hw) {
             let mut limits = None;
             if res.memory_limit.is_some() || res.cpu_limit.is_some() {
                 limits = Some(ResourceLimits {
                     memory: res.memory_limit,
                     cpus: res.cpu_limit,
                 });
             }

             let mut reservations = None;
             if res.memory_reservation.is_some() || res.cpu_reservation.is_some() {
                 reservations = Some(ResourceLimits {
                     memory: res.memory_reservation,
                     cpus: res.cpu_reservation,
                 });
             }

             if limits.is_some() || reservations.is_some() {
                 Some(Deploy {
                     resources: Some(Resources {
                         limits,
                         reservations,
                     })
                 })
             } else {
                 None
             }
        } else {
            None
        };

        let logging_info = service_impl.logging();
        let logging_options = if logging_info.options.is_empty() { None } else { Some(logging_info.options) };
        let logging = Some(Logging {
            driver: logging_info.driver,
            options: logging_options,
        });

        let service = Service {
            image: service_impl.image().to_string(),
            container_name: service_impl.name().to_string(),
            restart: "unless-stopped".to_string(),
            ports,
            environment,
            volumes,
            devices,
            networks,
            healthcheck,
            depends_on,
            security_opt,
            labels,
            cap_add,
            sysctls,
            deploy,
            logging,
        };

        compose_services.insert(name, service);
    }

    let mut networks = HashMap::new();
    networks.insert("server_manager_net".to_string(), Network {
        driver: "bridge".to_string(),
    });

    Ok(ComposeFile {
        version: None,
        services: compose_services,
        networks,
    })
}
