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
