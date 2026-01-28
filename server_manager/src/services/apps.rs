use super::Service;
use crate::core::hardware::{HardwareInfo, HardwareProfile};
use crate::core::secrets::Secrets;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

pub struct VaultwardenService;
impl Service for VaultwardenService {
    fn name(&self) -> &'static str { "vaultwarden" }
    fn image(&self) -> &'static str { "vaultwarden/server:latest" }
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:8001:80".to_string()] }
    fn env_vars(&self, _hw: &HardwareInfo, secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("ADMIN_TOKEN".to_string(), secrets.vaultwarden_admin_token.clone().unwrap_or_default());
        vars
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/vaultwarden:/data".to_string()]
    }
}

pub struct FilebrowserService;
impl Service for FilebrowserService {
    fn name(&self) -> &'static str { "filebrowser" }
    fn image(&self) -> &'static str { "filebrowser/filebrowser:latest" }
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:8002:80".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![
            "./config/filebrowser:/config".to_string(),
            "./media:/srv".to_string(),
            "./config/filebrowser/filebrowser.db:/database.db".to_string()
        ]
    }
}

pub struct YourlsService;
impl Service for YourlsService {
    fn name(&self) -> &'static str { "yourls" }
    fn image(&self) -> &'static str { "yourls/yourls:latest" }
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:8003:80".to_string()] }
    fn env_vars(&self, _hw: &HardwareInfo, secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("YOURLS_DB_HOST".to_string(), "mariadb".to_string());
        vars.insert("YOURLS_DB_USER".to_string(), "server_manager".to_string());
        vars.insert("YOURLS_DB_PASS".to_string(), secrets.mysql_user_password.clone().unwrap_or_default());
        vars.insert("YOURLS_DB_NAME".to_string(), "yourls".to_string());
        vars.insert("YOURLS_USER".to_string(), "admin".to_string());
        vars.insert("YOURLS_PASS".to_string(), secrets.yourls_admin_password.clone().unwrap_or_default());
        vars
    }
    fn depends_on(&self) -> Vec<String> { vec!["mariadb".to_string()] }
}

pub struct GLPIService;
impl Service for GLPIService {
    fn name(&self) -> &'static str { "glpi" }
    fn image(&self) -> &'static str { "diouxx/glpi:latest" } // Common community image, official docker-library is scarce
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:8088:80".to_string()] }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/glpi:/var/www/html/glpi".to_string()]
    }
    fn security_opts(&self) -> Vec<String> { vec!["no-new-privileges:true".to_string()] }
}

pub struct GiteaService;
impl Service for GiteaService {
    fn name(&self) -> &'static str { "gitea" }
    fn image(&self) -> &'static str { "gitea/gitea:latest" }
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:3000:3000".to_string(), "2222:22".to_string()] }
    fn env_vars(&self, _hw: &HardwareInfo, secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("GITEA__database__DB_TYPE".to_string(), "mysql".to_string());
        vars.insert("GITEA__database__HOST".to_string(), "mariadb:3306".to_string());
        vars.insert("GITEA__database__NAME".to_string(), "gitea".to_string());
        vars.insert("GITEA__database__USER".to_string(), "gitea".to_string());
        vars.insert("GITEA__database__PASSWD".to_string(), secrets.gitea_db_password.clone().unwrap_or_default());
        vars
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![
            "./config/gitea:/data".to_string(),
            "/etc/timezone:/etc/timezone:ro".to_string(),
            "/etc/localtime:/etc/localtime:ro".to_string()
        ]
    }
    fn security_opts(&self) -> Vec<String> { vec!["no-new-privileges:true".to_string()] }
}

pub struct RoundcubeService;
impl Service for RoundcubeService {
    fn name(&self) -> &'static str { "roundcube" }
    fn image(&self) -> &'static str { "roundcube/roundcubemail:latest" }
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:8090:80".to_string()] }
    fn env_vars(&self, _hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("ROUNDCUBEMAIL_DB_TYPE".to_string(), "sqlite".to_string()); // Defaulting to sqlite as per memory hints or keeping simple.
        // Memory says: "uses SQLite."
        vars.insert("ROUNDCUBEMAIL_SKIN".to_string(), "elastic".to_string());
        vars
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![
            "./config/roundcube/db:/var/roundcube/db".to_string(),
            "./config/roundcube/config:/var/roundcube/config".to_string()
        ]
    }
    fn security_opts(&self) -> Vec<String> { vec!["no-new-privileges:true".to_string()] }
    fn depends_on(&self) -> Vec<String> { vec!["mailserver".to_string()] }
}

pub struct NextcloudService;
impl Service for NextcloudService {
    fn name(&self) -> &'static str { "nextcloud" }
    fn image(&self) -> &'static str { "lscr.io/linuxserver/nextcloud:latest" }
    fn ports(&self) -> Vec<String> { vec!["127.0.0.1:4443:443".to_string()] }
    fn configure(&self, _hw: &HardwareInfo, secrets: &Secrets) -> Result<()> {
        let config_dir = Path::new("./config/nextcloud");
        fs::create_dir_all(config_dir).context("Failed to create nextcloud config dir")?;

        let escape_php = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        let db_pass = escape_php(&secrets.nextcloud_db_password.clone().unwrap_or_default());
        let admin_pass = escape_php(&secrets.nextcloud_admin_password.clone().unwrap_or_default());

        let php_config = format!(r#"<?php
$AUTOCONFIG = array(
  "dbtype"        => "mysql",
  "dbname"        => "nextcloud",
  "dbuser"        => "nextcloud",
  "dbpass"        => "{}",
  "dbhost"        => "mariadb",
  "directory"     => "/data",
  "adminlogin"    => "admin",
  "adminpass"     => "{}",
);
"#, db_pass, admin_pass);

        fs::write(config_dir.join("autoconfig.php"), php_config).context("Failed to write autoconfig.php")?;
        Ok(())
    }

    fn env_vars(&self, hw: &HardwareInfo, secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PUID".to_string(), hw.user_id.clone());
        vars.insert("PGID".to_string(), hw.group_id.clone());
        vars.insert("MYSQL_HOST".to_string(), "mariadb".to_string());
        vars.insert("MYSQL_DATABASE".to_string(), "nextcloud".to_string());
        vars.insert("MYSQL_USER".to_string(), "nextcloud".to_string());
        vars.insert("MYSQL_PASSWORD".to_string(), secrets.nextcloud_db_password.clone().unwrap_or_default());
        vars.insert("REDIS_HOST".to_string(), "redis".to_string());
        vars
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec!["./config/nextcloud:/config".to_string(), "./media:/data".to_string()]
    }
    fn depends_on(&self) -> Vec<String> { vec!["mariadb".to_string(), "redis".to_string()] }
}

pub struct MailService;
impl Service for MailService {
    fn name(&self) -> &'static str { "mailserver" }
    fn image(&self) -> &'static str { "mailserver/docker-mailserver:latest" }
    fn ports(&self) -> Vec<String> {
        vec![
            "25:25".to_string(),
            "143:143".to_string(),
            "587:587".to_string(),
            "993:993".to_string(),
        ]
    }
    fn env_vars(&self, hw: &HardwareInfo, _secrets: &Secrets) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("DMS_DEBUG".to_string(), "0".to_string());
        vars.insert("ENABLE_POSTGREY".to_string(), "0".to_string());
        vars.insert("ONE_DIR".to_string(), "1".to_string());
        vars.insert("POSTMASTER_ADDRESS".to_string(), "postmaster@example.com".to_string());

        if let HardwareProfile::Low = hw.profile {
             vars.insert("ENABLE_CLAMAV".to_string(), "0".to_string());
             vars.insert("ENABLE_SPAMASSASSIN".to_string(), "0".to_string());
             vars.insert("ENABLE_FAIL2BAN".to_string(), "0".to_string());
        } else {
             vars.insert("ENABLE_CLAMAV".to_string(), "1".to_string());
             vars.insert("ENABLE_SPAMASSASSIN".to_string(), "1".to_string());
             vars.insert("ENABLE_FAIL2BAN".to_string(), "1".to_string());
        }

        vars
    }
    fn volumes(&self, _hw: &HardwareInfo) -> Vec<String> {
        vec![
            "./config/mailserver/mail-data:/var/mail".to_string(),
            "./config/mailserver/mail-state:/var/mail-state".to_string(),
            "./config/mailserver/mail-logs:/var/log/mail".to_string(),
            "./config/mailserver/config:/tmp/docker-mailserver".to_string(),
            "/etc/localtime:/etc/localtime:ro".to_string()
        ]
    }
    fn security_opts(&self) -> Vec<String> { vec!["no-new-privileges:true".to_string()] }
    fn cap_add(&self) -> Vec<String> { vec!["NET_ADMIN".to_string()] }
}
