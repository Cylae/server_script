use super::Service;
use crate::core::hardware::{HardwareInfo};
use crate::core::secrets::Secrets;
use std::collections::HashMap;

pub struct MariaDBService;
impl Service for MariaDBService {
    fn name(&self) -> &'static str { "mariadb" }
    fn image(&self) -> &'static str { "lscr.io/linuxserver/mariadb:latest" }
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
