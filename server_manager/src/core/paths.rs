use std::path::{Path, PathBuf};

// This module handles standard paths for configuration, secrets, and users.
// It prioritizes /opt/server_manager if available, falling back to CWD for development.

pub const OPT_DIR: &str = "/opt/server_manager";

/// Returns the path to load a file from.
/// Priority: /opt/server_manager/<filename> > ./<filename>
/// If /opt version exists, returns it. Otherwise returns ./<filename> (even if it doesn't exist yet).
pub fn get_load_path(filename: &str) -> PathBuf {
    let opt_path = Path::new(OPT_DIR).join(filename);
    if opt_path.exists() {
        opt_path
    } else {
        PathBuf::from(filename)
    }
}

/// Returns the path to save a file to.
/// Priority: /opt/server_manager/<filename> (if /opt/server_manager dir exists) > ./<filename>
pub fn get_save_path(filename: &str) -> PathBuf {
    let opt_dir = Path::new(OPT_DIR);
    if opt_dir.exists() && opt_dir.is_dir() {
        opt_dir.join(filename)
    } else {
        PathBuf::from(filename)
    }
}

pub fn get_config_path() -> PathBuf {
    get_load_path("config.yaml")
}

pub fn get_users_path() -> PathBuf {
    get_load_path("users.yaml")
}

pub fn get_secrets_path() -> PathBuf {
    get_load_path("secrets.yaml")
}
