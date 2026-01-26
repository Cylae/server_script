use super::Service;
use crate::core::hardware::{HardwareInfo, HardwareProfile};
use crate::core::secrets::Secrets;
use crate::core::system;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use log::info;

pub struct MariaDBService;
impl Service for MariaDBService {
    fn name(&self) -> &'static str { "mariadb" }
    fn image(&self) -> &'static str { "lscr.io/linuxserver/mariadb:latest" }

    fn initialize(&self, hw: &HardwareInfo, install_dir: &Path) -> Result<()> {
        let buffer_size = match hw.profile {
            HardwareProfile::Low => "256M",
            HardwareProfile::Standard => "1G",
            HardwareProfile::High => "4G",
        };

        info!("Generating MariaDB optimization config (innodb_buffer_pool_size={})", buffer_size);

        let config_content = format!(
            "[mysqld]\ninnodb_buffer_pool_size={}\n",
            buffer_size
        );

        let config_dir = install_dir.join("config/mariadb");
        fs::create_dir_all(&config_dir).context("Failed to create mariadb config directory")?;

        let config_path = config_dir.join("custom.cnf");
        fs::write(config_path, config_content).context("Failed to write custom.cnf")?;

        Ok(())
    }

    fn ports(&self) -> Vec<String> { vec!["3306:3306".to_string()] }
    fn env_vars(&self, _hw: &HardwareInfo, secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PUID".to_string(), "1000".to_string());
        vars.insert("PGID".to_string(), "1000".to_string());
        vars.insert("MYSQL_ROOT_PASSWORD".to_string(), secrets.mysql_root_password.clone().unwrap_or_default());
        vars.insert("MYSQL_DATABASE".to_string(), "cylae".to_string());
        vars.insert("MYSQL_USER".to_string(), "cylae".to_string());
        vars.insert("MYSQL_PASSWORD".to_string(), secrets.mysql_user_password.clone().unwrap_or_default());
        vars
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/mariadb:/config".to_string()]
    }
}

pub struct RedisService;
impl Service for RedisService {
    fn name(&self) -> &'static str { "redis" }
    fn image(&self) -> &'static str { "redis:alpine" }
    fn ports(&self) -> Vec<String> { vec!["6379:6379".to_string()] }
}

pub struct NginxProxyService;
impl Service for NginxProxyService {
    fn name(&self) -> &'static str { "nginx-proxy" }
    fn image(&self) -> &'static str { "jc21/nginx-proxy-manager:latest" }

    fn initialize(&self, _hw: &HardwareInfo, _install_dir: &Path) -> Result<()> {
        for service in ["apache2", "nginx", "caddy", "httpd"] {
            system::stop_service(service)?;
        }
        Ok(())
    }

    fn ports(&self) -> Vec<String> { vec!["80:80".to_string(), "81:81".to_string(), "443:443".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/npm:/data".to_string(), "./config/npm/letsencrypt:/etc/letsencrypt".to_string()]
    }
}

pub struct DNSCryptService;
impl Service for DNSCryptService {
    fn name(&self) -> &'static str { "dnscrypt-proxy" }
    fn image(&self) -> &'static str { "klutchell/dnscrypt-proxy:latest" }
    fn ports(&self) -> Vec<String> { vec!["5300:5053/tcp".to_string(), "5300:5053/udp".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/dnscrypt:/config".to_string()]
    }
}

pub struct WireguardService;
impl Service for WireguardService {
    fn name(&self) -> &'static str { "wireguard" }
    fn image(&self) -> &'static str { "lscr.io/linuxserver/wireguard:latest" }
    fn ports(&self) -> Vec<String> { vec!["51820:51820/udp".to_string()] }
    fn cap_add(&self) -> Vec<String> { vec!["NET_ADMIN".to_string(), "SYS_MODULE".to_string()] }
    fn sysctls(&self) -> Vec<String> { vec!["net.ipv4.conf.all.src_valid_mark=1".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/wireguard:/config".to_string(), "/lib/modules:/lib/modules".to_string()]
    }
}

pub struct PortainerService;
impl Service for PortainerService {
    fn name(&self) -> &'static str { "portainer" }
    fn image(&self) -> &'static str { "portainer/portainer-ce:latest" }
    fn ports(&self) -> Vec<String> { vec!["9000:9000".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["/var/run/docker.sock:/var/run/docker.sock".to_string(), "./config/portainer:/data".to_string()]
    }
    fn security_opts(&self) -> Vec<String> { vec!["no-new-privileges:true".to_string()] }
}

pub struct NetdataService;
impl Service for NetdataService {
    fn name(&self) -> &'static str { "netdata" }
    fn image(&self) -> &'static str { "netdata/netdata" }
    fn ports(&self) -> Vec<String> { vec!["19999:19999".to_string()] }
    fn cap_add(&self) -> Vec<String> { vec!["SYS_PTRACE".to_string()] }
    fn security_opts(&self) -> Vec<String> { vec!["apparmor:unconfined".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![
            "/proc:/host/proc:ro".to_string(),
            "/sys:/host/sys:ro".to_string(),
            "/var/run/docker.sock:/var/run/docker.sock:ro".to_string()
        ]
    }
}

pub struct UptimeKumaService;
impl Service for UptimeKumaService {
    fn name(&self) -> &'static str { "uptime-kuma" }
    fn image(&self) -> &'static str { "louislam/uptime-kuma:1" }
    fn ports(&self) -> Vec<String> { vec!["3001:3001".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/uptime-kuma:/app/data".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_mariadb_config_generation() {
        // Setup
        let service = MariaDBService;
        let install_dir = std::env::temp_dir().join("cylae_test_mariadb");
        let _ = fs::remove_dir_all(&install_dir);
        fs::create_dir_all(&install_dir).unwrap();

        // Case 1: Low Profile
        let hw_low = HardwareInfo {
            profile: HardwareProfile::Low,
            ram_gb: 2,
            cpu_cores: 2,
            has_nvidia: false,
            has_intel_quicksync: false,
            disk_gb: 50,
            swap_gb: 0,
        };

        service.initialize(&hw_low, &install_dir).unwrap();
        let config_path = install_dir.join("config/mariadb/custom.cnf");
        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("innodb_buffer_pool_size=256M"));

        // Case 2: Standard Profile
        let hw_std = HardwareInfo {
            profile: HardwareProfile::Standard,
            ram_gb: 8,
            cpu_cores: 4,
            has_nvidia: false,
            has_intel_quicksync: false,
            disk_gb: 100,
            swap_gb: 2,
        };
        service.initialize(&hw_std, &install_dir).unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("innodb_buffer_pool_size=1G"));

         // Case 3: High Profile
        let hw_high = HardwareInfo {
            profile: HardwareProfile::High,
            ram_gb: 32,
            cpu_cores: 8,
            has_nvidia: false,
            has_intel_quicksync: false,
            disk_gb: 500,
            swap_gb: 0,
        };
        service.initialize(&hw_high, &install_dir).unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("innodb_buffer_pool_size=4G"));

        // Cleanup
        fs::remove_dir_all(&install_dir).unwrap();
    }
}
