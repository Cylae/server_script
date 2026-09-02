use anyhow::{Context, Result};
use log::info;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    pub server_manager_admin_password: Option<String>,
}

impl Secrets {
    fn get_secrets_path() -> PathBuf {
        let opt_path = Path::new("/opt/server_manager/secrets.yaml");
        let local_path = Path::new("secrets.yaml");
        if opt_path.exists() {
            opt_path.to_path_buf()
        } else if local_path.exists() {
            local_path.to_path_buf()
        } else if Path::new("/opt/server_manager").exists() {
            opt_path.to_path_buf()
        } else {
            local_path.to_path_buf()
        }
    }

    pub fn load_or_create() -> Result<Self> {
        let path = Self::get_secrets_path();
        let mut secrets: Secrets = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            serde_yaml_ng::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))?
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
        if secrets.server_manager_admin_password.is_none() {
            secrets.server_manager_admin_password = Some(generate_hex(16)?);
            changed = true;
        }

        if changed {
            info!("Generated new secrets.");
            let content = serde_yaml_ng::to_string(&secrets)?;
            fs::write(&path, content)
                .with_context(|| format!("Failed to write {}", path.display()))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }

        Ok(secrets)
    }
}

fn generate_hex(bytes: usize) -> Result<String> {
    let mut buffer = vec![0u8; bytes];
    rand::rng().fill(&mut buffer[..]);

    const HEX_CHARS: &[u8] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes * 2);
    for b in buffer {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_generation() {
        let hex = generate_hex(16).expect("Value should exist");
        assert_eq!(hex.len(), 32); // 16 bytes = 32 hex chars
    }

    #[test]
    fn test_secrets_default() {
        let secrets = Secrets::default();
        assert!(secrets.mysql_root_password.is_none());
        assert!(secrets.server_manager_admin_password.is_none());
    }
}
