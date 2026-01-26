use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use log::info;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Secrets {
    pub mysql_root_password: Option<String>,
    pub mysql_user_password: Option<String>,
    pub nextcloud_admin_password: Option<String>,
    pub nextcloud_db_password: Option<String>,
    pub mailserver_password: Option<String>,
    pub glpi_db_password: Option<String>,
    pub gitea_db_password: Option<String>,
    pub roundcube_db_password: Option<String>,
    pub yourls_admin_password: Option<String>,
    pub vaultwarden_admin_token: Option<String>,
}

impl Secrets {
    pub fn load_or_create() -> Result<Self> {
        let path = Path::new("secrets.yaml");
        let mut secrets: Secrets = if path.exists() {
            let content = fs::read_to_string(path).context("Failed to read secrets.yaml")?;
            serde_yaml::from_str(&content).context("Failed to parse secrets.yaml")?
        } else {
            Secrets::default()
        };

        let mut changed = false;
        if secrets.mysql_root_password.is_none() {
            secrets.mysql_root_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.mysql_user_password.is_none() {
            secrets.mysql_user_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.nextcloud_admin_password.is_none() {
            secrets.nextcloud_admin_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.nextcloud_db_password.is_none() {
            secrets.nextcloud_db_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.mailserver_password.is_none() {
            secrets.mailserver_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.glpi_db_password.is_none() {
            secrets.glpi_db_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.gitea_db_password.is_none() {
            secrets.gitea_db_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.roundcube_db_password.is_none() {
            secrets.roundcube_db_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.yourls_admin_password.is_none() {
            secrets.yourls_admin_password = Some(generate_hex(16)?);
            changed = true;
        }
        if secrets.vaultwarden_admin_token.is_none() {
            secrets.vaultwarden_admin_token = Some(generate_hex(16)?);
            changed = true;
        }

        if changed {
            info!("Generated new secrets.");
            let content = serde_yaml::to_string(&secrets)?;
            fs::write(path, content).context("Failed to write secrets.yaml")?;
        }

        Ok(secrets)
    }
}

fn generate_hex(bytes: usize) -> Result<String> {
    let mut file = File::open("/dev/urandom").context("Failed to open /dev/urandom")?;
    let mut buffer = vec![0u8; bytes];
    file.read_exact(&mut buffer).context("Failed to read from /dev/urandom")?;
    Ok(buffer.iter().map(|b| format!("{:02x}", b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_generation() {
        let hex = generate_hex(16).unwrap();
        assert_eq!(hex.len(), 32); // 16 bytes = 32 hex chars
    }

    #[test]
    fn test_secrets_default() {
        let secrets = Secrets::default();
        assert!(secrets.mysql_root_password.is_none());
    }
}
