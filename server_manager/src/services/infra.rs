use super::{Service, ResourceConfig};
use crate::core::hardware::{HardwareInfo, HardwareProfile};
use crate::core::secrets::Secrets;
use std::collections::HashMap;
use anyhow::{Result, Context};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct MariaDBService;
impl Service for MariaDBService {
    fn name(&self) -> &'static str { "mariadb" }
    fn image(&self) -> &'static str { "lscr.io/linuxserver/mariadb:latest" }
    fn ports(&self) -> Vec<String> { vec![] } // Internal only

    fn configure(&self, hw: &HardwareInfo, secrets: &Secrets) -> Result<()> {
        let init_dir = Path::new("./config/mariadb/initdb.d");
        fs::create_dir_all(init_dir).context("Failed to create mariadb initdb.d")?;

        let mut sql = String::new();

        let escape = |s: &str| s.replace("'", "\\'");

        // Nextcloud
        if let Some(pass) = &secrets.nextcloud_db_password {
            sql.push_str("CREATE DATABASE IF NOT EXISTS nextcloud;\n");
            sql.push_str(&format!("CREATE USER IF NOT EXISTS 'nextcloud'@'%' IDENTIFIED BY '{}';\n", escape(pass)));
            sql.push_str("GRANT ALL PRIVILEGES ON nextcloud.* TO 'nextcloud'@'%';\n");
        }

        // GLPI
        if let Some(pass) = &secrets.glpi_db_password {
             sql.push_str("CREATE DATABASE IF NOT EXISTS glpi;\n");
             sql.push_str(&format!("CREATE USER IF NOT EXISTS 'glpi'@'%' IDENTIFIED BY '{}';\n", escape(pass)));
             sql.push_str("GRANT ALL PRIVILEGES ON glpi.* TO 'glpi'@'%';\n");
        }

        // Gitea
        if let Some(pass) = &secrets.gitea_db_password {
             sql.push_str("CREATE DATABASE IF NOT EXISTS gitea;\n");
             sql.push_str(&format!("CREATE USER IF NOT EXISTS 'gitea'@'%' IDENTIFIED BY '{}';\n", escape(pass)));
             sql.push_str("GRANT ALL PRIVILEGES ON gitea.* TO 'gitea'@'%';\n");
        }

        sql.push_str("FLUSH PRIVILEGES;\n");

        fs::write(init_dir.join("init.sql"), sql).context("Failed to write init.sql")?;

        // Optimization: Generate custom.cnf
        let (buffer_pool, log_file_size, max_connections) = match hw.profile {
            HardwareProfile::High => ("4G", "1G", "500"),
            HardwareProfile::Standard => ("1G", "256M", "100"),
            HardwareProfile::Low => ("256M", "64M", "50"),
        };

        let custom_cnf = format!(r#"[mysqld]
innodb_buffer_pool_size={}
innodb_log_file_size={}
max_connections={}
"#, buffer_pool, log_file_size, max_connections);

        // Parent dir is ./config/mariadb/
        let config_dir = init_dir.parent().context("Failed to determine parent directory for custom.cnf")?;
        fs::write(config_dir.join("custom.cnf"), custom_cnf).context("Failed to write custom.cnf")?;

        Ok(())
    }

    fn env_vars(&self, hw: &HardwareInfo, secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PUID".to_string(), hw.user_id.clone());
        vars.insert("PGID".to_string(), hw.group_id.clone());
        vars.insert("MYSQL_ROOT_PASSWORD".to_string(), secrets.mysql_root_password.clone().unwrap_or_default());
        vars.insert("MYSQL_DATABASE".to_string(), "server_manager".to_string());
        vars.insert("MYSQL_USER".to_string(), "server_manager".to_string());
        vars.insert("MYSQL_PASSWORD".to_string(), secrets.mysql_user_password.clone().unwrap_or_default());
        vars
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/mariadb:/config".to_string()]
    }
    fn networks(&self) -> Vec<String> { vec!["server_manager_net".to_string()] }

    fn resources(&self, hw: &HardwareInfo) -> Option<ResourceConfig> {
        let memory_limit = match hw.profile {
            HardwareProfile::High => "4G",
            HardwareProfile::Standard => "2G",
            HardwareProfile::Low => "512M",
        };
        Some(ResourceConfig {
            memory_limit: Some(memory_limit.to_string()),
            memory_reservation: None,
            cpu_limit: None,
            cpu_reservation: None,
        })
    }
}

pub struct RedisService;
impl Service for RedisService {
    fn name(&self) -> &'static str { "redis" }
    fn image(&self) -> &'static str { "redis:alpine" }
    fn ports(&self) -> Vec<String> { vec![] } // Internal only
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/redis:/data".to_string()]
    }
    fn resources(&self, hw: &HardwareInfo) -> Option<ResourceConfig> {
         let memory_limit = match hw.profile {
            HardwareProfile::High => "512M",
            HardwareProfile::Standard => "256M",
            HardwareProfile::Low => "128M",
        };
        Some(ResourceConfig {
            memory_limit: Some(memory_limit.to_string()),
            memory_reservation: None,
            cpu_limit: None,
            cpu_reservation: None,
        })
    }
}

pub struct NginxProxyService;
impl Service for NginxProxyService {
    fn name(&self) -> &'static str { "nginx-proxy" }
    fn image(&self) -> &'static str { "jc21/nginx-proxy-manager:latest" }
    fn ports(&self) -> Vec<String> { vec!["80:80".to_string(), "81:81".to_string(), "443:443".to_string()] }

    fn initialize(&self, _hw: &HardwareInfo, _secrets: &Secrets) -> Result<()> {
        let services = vec!["apache2", "nginx", "httpd"];
        for svc in services {
            // Stop and disable conflicting web servers
            let _ = Command::new("systemctl").args(["stop", svc]).status();
            let _ = Command::new("systemctl").args(["disable", svc]).status();
        }
        Ok(())
    }

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
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:9000:9000".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["/var/run/docker.sock:/var/run/docker.sock".to_string(), "./config/portainer:/data".to_string()]
    }
    fn security_opts(&self) -> Vec<String> { vec!["no-new-privileges:true".to_string()] }
}

pub struct NetdataService;
impl Service for NetdataService {
    fn name(&self) -> &'static str { "netdata" }
    fn image(&self) -> &'static str { "netdata/netdata:latest" }
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:19999:19999".to_string()] }
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
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:3001:3001".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/uptime-kuma:/app/data".to_string()]
    }
}
