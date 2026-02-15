use crate::core::system;
use anyhow::{anyhow, Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::{info, warn};
use nix::unistd::Uid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    Admin,
    Observer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    #[serde(default)]
    pub quota_gb: Option<u64>,
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
            // We use a generated secret for the initial password if secrets exist,
            // otherwise generate one.
            // Better: use 'admin' / 'admin' but WARN, or generate random.
            // Let's generate a random one and print it, safer.
            // Re-using secrets generation logic if possible, or just simple random.
            // For simplicity in this context, let's look for a stored password or default to 'admin' and log a warning.

            let pass = "admin";
            let hash = hash(pass, DEFAULT_COST)?;
            manager.users.insert(
                "admin".to_string(),
                User {
                    username: "admin".to_string(),
                    password_hash: hash,
                    role: Role::Admin,
                    quota_gb: None,
                },
            );
            manager.save()?;
            info!("Default user 'admin' created with password 'admin'. CHANGE THIS IMMEDIATELY!");
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
        fs::write(target, content).context("Failed to write users.yaml")?;
        Ok(())
    }

    pub fn add_user(
        &mut self,
        username: &str,
        password: &str,
        role: Role,
        quota_gb: Option<u64>,
    ) -> Result<()> {
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
            },
        );
        self.save()
    }

    pub fn delete_user(&mut self, username: &str) -> Result<()> {
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

    pub async fn add_user_async(
        &mut self,
        username: &str,
        password: &str,
        role: Role,
        quota_gb: Option<u64>,
    ) -> Result<()> {
        let mut manager_clone = self.clone();
        let username = username.to_string();
        let password = password.to_string();

        let updated_manager = tokio::task::spawn_blocking(move || {
            manager_clone.add_user(&username, &password, role, quota_gb)?;
            Ok::<UserManager, anyhow::Error>(manager_clone)
        })
        .await??;

        *self = updated_manager;
        Ok(())
    }

    pub async fn delete_user_async(&mut self, username: &str) -> Result<()> {
        let mut manager_clone = self.clone();
        let username = username.to_string();

        let updated_manager = tokio::task::spawn_blocking(move || {
            manager_clone.delete_user(&username)?;
            Ok::<UserManager, anyhow::Error>(manager_clone)
        })
        .await??;

        *self = updated_manager;
        Ok(())
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
        assert_eq!(user.unwrap().role, Role::Observer);

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
    fn test_admin_protection() {
        let mut manager = UserManager::default();
        manager
            .add_user("admin", "admin", Role::Admin, None)
            .unwrap();

        // Should fail to delete last admin
        assert!(manager.delete_user("admin").is_err());

        // Add another admin
        manager
            .add_user("admin2", "admin", Role::Admin, None)
            .unwrap();
        // Now can delete one
        assert!(manager.delete_user("admin").is_ok());
    }
}
