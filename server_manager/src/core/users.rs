use crate::core::system;
use anyhow::{anyhow, Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use log::{info, warn};
use nix::unistd::Uid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedUsers {
    manager: UserManager,
    last_mtime: Option<SystemTime>,
    last_check: Option<SystemTime>,
}

static USERS_CACHE: OnceLock<RwLock<CachedUsers>> = OnceLock::new();

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
    fn get_active_path() -> PathBuf {
        let path = PathBuf::from("users.yaml");
        let fallback_path = PathBuf::from("/opt/server_manager/users.yaml");

        if fallback_path.exists() {
            fallback_path
        } else if path.exists() {
            path
        } else if Path::new("/opt/server_manager").exists() {
            fallback_path
        } else {
            path
        }
    }

    pub async fn load_async() -> Result<Self> {
        let cache = USERS_CACHE.get_or_init(|| {
            RwLock::new(CachedUsers {
                manager: UserManager::default(),
                last_mtime: None,
                last_check: None,
            })
        });

        // Fast path
        {
            let guard = cache.read().await;
            if let Some(last_check) = guard.last_check {
                if let Ok(elapsed) = SystemTime::now().duration_since(last_check) {
                    if elapsed < Duration::from_millis(500) {
                        return Ok(guard.manager.clone());
                    }
                }
            }
        }

        // Slow path
        let mut guard = cache.write().await;
        if let Some(last_check) = guard.last_check {
            if let Ok(elapsed) = SystemTime::now().duration_since(last_check) {
                if elapsed < Duration::from_millis(500) {
                    return Ok(guard.manager.clone());
                }
            }
        }

        Self::sync_cache(&mut guard).await?;
        guard.last_check = Some(SystemTime::now());
        Ok(guard.manager.clone())
    }

    async fn sync_cache(guard: &mut CachedUsers) -> Result<()> {
        let path = Self::get_active_path();
        let metadata_res = tokio::fs::metadata(&path).await;

        match metadata_res {
            Ok(metadata) => {
                let modified = metadata.modified().unwrap_or(SystemTime::now());
                let reload = match guard.last_mtime {
                    Some(cached) => modified != cached,
                    None => true,
                };

                if reload {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if content.trim().is_empty() {
                            guard.manager = UserManager::default();
                        } else {
                            guard.manager = serde_yaml_ng::from_str(&content)
                                .unwrap_or_else(|_| UserManager::default());
                        }
                    }
                    guard.last_mtime = Some(modified);
                }
            }
            Err(_) => {
                // If file doesn't exist, we might need to create default admin
                // But only if we are initializing?
                // For simplicity, if no file, we default.
                // But we must handle the "first run" logic of creating admin user.
                if guard.manager.users.is_empty() {
                    // Logic to create default admin if completely empty
                    // This is slightly tricky inside sync_cache which is a helper.
                    // But if we return empty manager, the caller might see it empty.
                    // Let's handle it here or in load().
                    // To match previous load() logic:
                    let pass = "admin";
                    // Blocking hash
                    let hash = tokio::task::spawn_blocking(move || hash(pass, DEFAULT_COST))
                        .await??;
                    guard.manager.users.insert(
                        "admin".to_string(),
                        User {
                            username: "admin".to_string(),
                            password_hash: hash,
                            role: Role::Admin,
                            quota_gb: None,
                        },
                    );
                    // Save immediately
                    let content = serde_yaml_ng::to_string(&guard.manager)?;
                    tokio::fs::write(&path, content).await?;
                    if let Ok(m) = tokio::fs::metadata(&path).await {
                        guard.last_mtime = m.modified().ok();
                    }
                    info!("Default user 'admin' created with password 'admin'. CHANGE THIS IMMEDIATELY!");
                }
            }
        }
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_active_path();

        let mut manager = if path.exists() {
            let content = fs::read_to_string(&path).context("Failed to read users.yaml")?;
            if content.trim().is_empty() {
                UserManager::default()
            } else {
                serde_yaml_ng::from_str(&content).context("Failed to parse users.yaml")?
            }
        } else {
            UserManager::default()
        };

        if manager.users.is_empty() {
            info!("No users found. Creating default 'admin' user.");
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
            let content = serde_yaml_ng::to_string(&manager)?;
            fs::write(&path, content)?;
            info!("Default user 'admin' created with password 'admin'. CHANGE THIS IMMEDIATELY!");
        }

        Ok(manager)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_active_path();
        let content = serde_yaml_ng::to_string(self)?;
        fs::write(path, content).context("Failed to write users.yaml")?;
        Ok(())
    }

    pub async fn add_user_async(
        username: String,
        password: String,
        role: Role,
        quota_gb: Option<u64>,
    ) -> Result<()> {
        let cache = USERS_CACHE.get_or_init(|| {
            RwLock::new(CachedUsers {
                manager: UserManager::default(),
                last_mtime: None,
                last_check: None,
            })
        });

        let mut guard = cache.write().await;
        Self::sync_cache(&mut guard).await?;

        if guard.manager.users.contains_key(&username) {
            return Err(anyhow!("User already exists"));
        }

        let u_clone = username.clone();
        let p_clone = password.clone();
        let q_clone = quota_gb;

        tokio::task::spawn_blocking(move || {
            if Uid::effective().is_root() {
                system::create_system_user(&u_clone, &p_clone)?;
                if let Some(gb) = q_clone {
                    system::set_system_quota(&u_clone, gb)?;
                }
            } else {
                warn!(
                    "Not running as root. Skipping system user creation for '{}'.",
                    u_clone
                );
            }
            Ok::<(), anyhow::Error>(())
        })
        .await??;

        let hash = tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST)).await??;

        guard.manager.users.insert(
            username.clone(),
            User {
                username: username.clone(),
                password_hash: hash,
                role,
                quota_gb,
            },
        );

        let content = serde_yaml_ng::to_string(&guard.manager)?;
        let path = Self::get_active_path();
        tokio::fs::write(&path, content).await?;

        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            guard.last_mtime = metadata.modified().ok();
        }

        Ok(())
    }

    pub async fn delete_user_async(username: String) -> Result<()> {
        let cache = USERS_CACHE.get_or_init(|| {
            RwLock::new(CachedUsers {
                manager: UserManager::default(),
                last_mtime: None,
                last_check: None,
            })
        });

        let mut guard = cache.write().await;
        Self::sync_cache(&mut guard).await?;

        if !guard.manager.users.contains_key(&username) {
            return Err(anyhow!("User not found"));
        }
        if username == "admin" && guard.manager.users.len() == 1 {
            return Err(anyhow!("Cannot delete the last admin user"));
        }

        let u_clone = username.clone();
        tokio::task::spawn_blocking(move || {
            if Uid::effective().is_root() {
                system::delete_system_user(&u_clone)?;
            } else {
                warn!(
                    "Not running as root. Skipping system user deletion for '{}'.",
                    u_clone
                );
            }
            Ok::<(), anyhow::Error>(())
        })
        .await??;

        guard.manager.users.remove(&username);

        let content = serde_yaml_ng::to_string(&guard.manager)?;
        let path = Self::get_active_path();
        tokio::fs::write(&path, content).await?;

        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            guard.last_mtime = metadata.modified().ok();
        }

        Ok(())
    }

    pub async fn list_users_async() -> Result<Vec<User>> {
        let manager = Self::load_async().await?;
        Ok(manager.users.values().cloned().collect())
    }

    pub async fn get_user_async(username: &str) -> Option<User> {
         if let Ok(manager) = Self::load_async().await {
             manager.users.get(username).cloned()
         } else {
             None
         }
    }

    pub async fn update_password_async(username: String, new_password: String) -> Result<()> {
        let cache = USERS_CACHE.get_or_init(|| {
            RwLock::new(CachedUsers {
                manager: UserManager::default(),
                last_mtime: None,
                last_check: None,
            })
        });

        let mut guard = cache.write().await;
        Self::sync_cache(&mut guard).await?;

        if let Some(user) = guard.manager.users.get_mut(&username) {
             let u_clone = username.clone();
             let p_clone = new_password.clone();

             tokio::task::spawn_blocking(move || {
                if Uid::effective().is_root() {
                    system::set_system_user_password(&u_clone, &p_clone)?;
                } else {
                    warn!(
                        "Not running as root. Skipping system password update for '{}'.",
                        u_clone
                    );
                }
                Ok::<(), anyhow::Error>(())
            }).await??;

            let hash = tokio::task::spawn_blocking(move || hash(&new_password, DEFAULT_COST)).await??;
            user.password_hash = hash;

            let content = serde_yaml_ng::to_string(&guard.manager)?;
            let path = Self::get_active_path();
            tokio::fs::write(&path, content).await?;

            if let Ok(metadata) = tokio::fs::metadata(&path).await {
                guard.last_mtime = metadata.modified().ok();
            }
            Ok(())
        } else {
            Err(anyhow!("User not found"))
        }
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
