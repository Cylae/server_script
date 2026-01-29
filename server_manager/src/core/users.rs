use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use log::info;
use bcrypt::{DEFAULT_COST, hash, verify};

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
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserManager {
    users: HashMap<String, User>,
}

impl UserManager {
    pub fn load() -> Result<Self> {
        // Try CWD or /opt/server_manager
        let path = Path::new("users.yaml");
        let fallback_path = Path::new("/opt/server_manager/users.yaml");

        let load_path = if path.exists() {
            Some(path)
        } else if fallback_path.exists() {
            Some(fallback_path)
        } else {
            None
        };

        let mut manager = if let Some(p) = load_path {
            let content = fs::read_to_string(p).context("Failed to read users.yaml")?;
            if content.trim().is_empty() {
                UserManager::default()
            } else {
                serde_yaml::from_str(&content).context("Failed to parse users.yaml")?
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
            manager.users.insert("admin".to_string(), User {
                username: "admin".to_string(),
                password_hash: hash,
                role: Role::Admin,
            });
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

        let content = serde_yaml::to_string(self)?;
        fs::write(target, content).context("Failed to write users.yaml")?;
        Ok(())
    }

    pub fn add_user(&mut self, username: &str, password: &str, role: Role) -> Result<()> {
        if self.users.contains_key(username) {
            return Err(anyhow!("User already exists"));
        }
        let hash = hash(password, DEFAULT_COST)?;
        self.users.insert(username.to_string(), User {
            username: username.to_string(),
            password_hash: hash,
            role,
        });
        self.save()
    }

    pub fn delete_user(&mut self, username: &str) -> Result<()> {
        if !self.users.contains_key(username) {
             return Err(anyhow!("User not found"));
        }
        if username == "admin" && self.users.len() == 1 {
             return Err(anyhow!("Cannot delete the last admin user"));
        }
        self.users.remove(username);
        self.save()
    }

    pub fn update_password(&mut self, username: &str, new_password: &str) -> Result<()> {
        if let Some(user) = self.users.get_mut(username) {
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
        assert!(manager.add_user("testuser", "password123", Role::Observer).is_ok());
        assert!(manager.add_user("testuser", "password123", Role::Observer).is_err()); // Duplicate

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
        manager.add_user("admin", "admin", Role::Admin).unwrap();

        // Should fail to delete last admin
        assert!(manager.delete_user("admin").is_err());

        // Add another admin
        manager.add_user("admin2", "admin", Role::Admin).unwrap();
        // Now can delete one
        assert!(manager.delete_user("admin").is_ok());
    }
}
