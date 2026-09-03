use crate::core::secrets::Secrets;
use crate::core::system;
use anyhow::{anyhow, Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::{info, warn};
use nix::unistd::Uid;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    Admin,
    Observer,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    #[serde(default)]
    pub quota_gb: Option<u64>,
    #[serde(default)]
    pub installed_apps: HashSet<String>,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("username", &self.username)
            .field("password_hash", &"[REDACTED]")
            .field("role", &self.role)
            .field("quota_gb", &self.quota_gb)
            .field("installed_apps", &self.installed_apps)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct UserManager {
    users: HashMap<String, User>,
}

impl UserManager {
    pub async fn load_async() -> Result<Self> {
        tokio::task::spawn_blocking(Self::load).await?
    }

    pub fn load() -> Result<Self> {
        // Try CWD or /opt/server_manager
        let path = Path::new("users.yaml");
        let fallback_path = Path::new("/opt/server_manager/users.yaml");

        // Priority: /opt/server_manager/users.yaml > ./users.yaml
        // This aligns with save() behavior which prefers /opt if available.
        let load_path = if fallback_path.exists() {
            Some(fallback_path)
        } else if path.exists() {
            Some(path)
        } else {
            None
        };

        let mut manager = if let Some(p) = load_path {
            let content = fs::read_to_string(p).context("Failed to read users.yaml")?;
            if content.trim().is_empty() {
                UserManager::default()
            } else {
                serde_yaml_ng::from_str(&content).context("Failed to parse users.yaml")?
            }
        } else {
            UserManager::default()
        };

        // Ensure default admin exists if no users
        if manager.users.is_empty() {
            info!("No users found. Creating default 'admin' user.");
            let initial_pass = match Secrets::load_or_create() {
                Ok(s) => s
                    .server_manager_admin_password
                    .unwrap_or_else(|| "admin".to_string()),
                Err(_) => "admin".to_string(),
            };

            let pass_hash = hash(&initial_pass, DEFAULT_COST)?;
            manager.users.insert(
                "admin".to_string(),
                User {
                    username: "admin".to_string(),
                    password_hash: pass_hash,
                    role: Role::Admin,
                    quota_gb: None,
                    installed_apps: HashSet::new(),
                },
            );
            manager.save()?;
            if initial_pass == "admin" {
                info!(
                    "Default user 'admin' created with password 'admin'. CHANGE THIS IMMEDIATELY!"
                );
            } else {
                info!("Default user 'admin' created with secret password from secrets.yaml.");
            }
        }

        Ok(manager)
    }

    pub fn save(&self) -> Result<()> {
        // Prefer saving to /opt/server_manager if it exists/is writable, else CWD
        let target = if Path::new("/opt/server_manager").exists() {
            Path::new("/opt/server_manager/users.yaml")
        } else {
            Path::new("users.yaml")
        };

        let content = serde_yaml_ng::to_string(self)?;
        crate::core::atomic_io::atomic_write_str(target, &content, 0o600)
            .context("Failed to write users.yaml")?;

        Ok(())
    }

    pub fn add_user(
        &mut self,
        username: &str,
        password: &str,
        role: Role,
        quota_gb: Option<u64>,
    ) -> Result<()> {
        crate::core::validate::validate_username(username)?;
        if self.users.contains_key(username) {
            return Err(anyhow!("User already exists"));
        }

        // System User Integration
        if Uid::effective().is_root() {
            system::create_system_user(username, password)?;
            if let Some(gb) = quota_gb {
                system::set_system_quota(username, gb)?;
            }
        } else {
            warn!(
                "Not running as root. Skipping system user creation for '{}'.",
                username
            );
        }

        let hash = hash(password, DEFAULT_COST)?;
        self.users.insert(
            username.to_string(),
            User {
                username: username.to_string(),
                password_hash: hash,
                role,
                quota_gb,
                installed_apps: HashSet::new(),
            },
        );
        self.save()
    }

    pub fn install_user_app(&mut self, username: &str, app_name: &str) -> Result<()> {
        crate::core::validate::validate_username(username)?;
        crate::core::validate::validate_service_name(app_name)?;
        if let Some(user) = self.users.get_mut(username) {
            user.installed_apps.insert(app_name.to_string());
            self.save()
        } else {
            Err(anyhow!("User not found"))
        }
    }

    pub fn update_user_role_and_quota(
        &mut self,
        username: &str,
        role: Role,
        quota_gb: Option<u64>,
    ) -> Result<()> {
        crate::core::validate::validate_username(username)?;
        if let Some(user) = self.users.get_mut(username) {
            if Uid::effective().is_root() {
                if let Some(gb) = quota_gb {
                    let _ = system::set_system_quota(username, gb);
                }
            }
            user.role = role;
            user.quota_gb = quota_gb;
            self.save()
        } else {
            Err(anyhow!("User not found"))
        }
    }

    pub fn uninstall_user_app(&mut self, username: &str, app_name: &str) -> Result<()> {
        crate::core::validate::validate_username(username)?;
        crate::core::validate::validate_service_name(app_name)?;
        if let Some(user) = self.users.get_mut(username) {
            user.installed_apps.remove(app_name);
            self.save()
        } else {
            Err(anyhow!("User not found"))
        }
    }

    pub fn delete_user(&mut self, username: &str) -> Result<()> {
        crate::core::validate::validate_username(username)?;
        if !self.users.contains_key(username) {
            return Err(anyhow!("User not found"));
        }
        if username == "admin" && self.users.len() == 1 {
            return Err(anyhow!("Cannot delete the last admin user"));
        }

        // System User Deletion
        if Uid::effective().is_root() {
            system::delete_system_user(username)?;
        } else {
            warn!(
                "Not running as root. Skipping system user deletion for '{}'.",
                username
            );
        }

        self.users.remove(username);
        self.save()
    }

    pub fn update_password(&mut self, username: &str, new_password: &str) -> Result<()> {
        if let Some(user) = self.users.get_mut(username) {
            // System Password Update
            if Uid::effective().is_root() {
                system::set_system_user_password(username, new_password)?;
            } else {
                warn!(
                    "Not running as root. Skipping system password update for '{}'.",
                    username
                );
            }

            user.password_hash = hash(new_password, DEFAULT_COST)?;
            self.save()
        } else {
            Err(anyhow!("User not found"))
        }
    }

    pub fn verify(&self, username: &str, password: &str) -> Option<User> {
        if let Some(user) = self.users.get(username) {
            if verify(password, &user.password_hash).unwrap_or(false) {
                return Some(user.clone());
            }
        }
        None
    }

    pub async fn verify_async(&self, username: &str, password: &str) -> Option<User> {
        if let Some(user) = self.users.get(username) {
            let hash = user.password_hash.clone();
            let password = password.to_string();
            let user_clone = user.clone();

            let is_valid =
                tokio::task::spawn_blocking(move || verify(&password, &hash).unwrap_or(false))
                    .await
                    .unwrap_or(false);

            if is_valid {
                return Some(user_clone);
            }
        }
        None
    }

    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }

    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_user_management() {
        let mut manager = UserManager::default();

        // Add User
        assert!(manager
            .add_user("testuser", "password123", Role::Observer, None)
            .is_ok());
        assert!(manager
            .add_user("testuser", "password123", Role::Observer, None)
            .is_err()); // Duplicate

        // Verify
        let user = manager.verify("testuser", "password123");
        assert!(user.is_some());
        assert_eq!(user.expect("Value should exist").role, Role::Observer);

        assert!(manager.verify("testuser", "wrongpass").is_none());

        // Update Password
        assert!(manager.update_password("testuser", "newpass").is_ok());
        assert!(manager.verify("testuser", "password123").is_none());
        assert!(manager.verify("testuser", "newpass").is_some());

        // Delete
        assert!(manager.delete_user("testuser").is_ok());
        assert!(manager.verify("testuser", "newpass").is_none());
    }

    #[test]
    fn test_user_app_management() {
        let mut manager = UserManager::default();
        assert!(manager
            .add_user("appuser", "pass123", Role::Observer, None)
            .is_ok());

        assert!(manager.install_user_app("appuser", "plex").is_ok());
        let u = manager.get_user("appuser").expect("User exists");
        assert!(u.installed_apps.contains("plex"));

        assert!(manager.uninstall_user_app("appuser", "plex").is_ok());
        let u2 = manager.get_user("appuser").expect("User exists");
        assert!(!u2.installed_apps.contains("plex"));
    }

    #[test]
    fn test_admin_protection() {
        let mut manager = UserManager::default();
        manager
            .add_user("admin", "admin", Role::Admin, None)
            .expect("Value should exist");

        // Should fail to delete last admin
        assert!(manager.delete_user("admin").is_err());

        // Add another admin
        manager
            .add_user("admin2", "admin", Role::Admin, None)
            .expect("Value should exist");
        // Now can delete one
        assert!(manager.delete_user("admin").is_ok());
    }

    #[test]
    fn test_update_user_role_and_quota() {
        let mut manager = UserManager::default();
        manager
            .add_user("user1", "pass123", Role::Observer, Some(10))
            .expect("User creation failed");

        let u = manager.get_user("user1").expect("User exists");
        assert_eq!(u.role, Role::Observer);
        assert_eq!(u.quota_gb, Some(10));

        assert!(manager
            .update_user_role_and_quota("user1", Role::Admin, Some(50))
            .is_ok());

        let updated_u = manager.get_user("user1").expect("User exists");
        assert_eq!(updated_u.role, Role::Admin);
        assert_eq!(updated_u.quota_gb, Some(50));
    }
}
