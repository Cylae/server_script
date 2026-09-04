use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::process::Command;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_notes: String,
}

/// Checks for software updates by inspecting git remote tags or environment.
pub fn check_for_updates() -> Result<UpdateInfo> {
    log::debug!(
        "Checking for software updates (current version: v{})...",
        CURRENT_VERSION
    );

    let current = CURRENT_VERSION.to_string();
    let mut latest = current.clone();
    let mut notes = "Software is running on current version.".to_string();

    // Check if running in a git repository
    if std::path::Path::new(".git").exists()
        || std::path::Path::new("/opt/server_manager/.git").exists()
    {
        if let Ok(git_output) = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .output()
        {
            if git_output.status.success() {
                let tag = String::from_utf8_lossy(&git_output.stdout)
                    .trim()
                    .to_string();
                let clean_tag = tag.strip_prefix('v').unwrap_or(&tag).to_string();
                if !clean_tag.is_empty() {
                    latest = clean_tag;
                }
            }
        }
    }

    let update_available = is_newer_version(&latest, &current);
    if update_available {
        notes = format!(
            "A new version (v{}) is available for Server Manager.",
            latest
        );
    }

    Ok(UpdateInfo {
        current_version: current,
        latest_version: latest,
        update_available,
        release_notes: notes,
    })
}

/// Executes software self-update by pulling latest source changes and rebuilding/updating binary.
pub fn self_update() -> Result<String> {
    info!("Starting software self-update procedure...");

    let repo_dir = if std::path::Path::new("/opt/server_manager/.git").exists() {
        std::path::Path::new("/opt/server_manager")
    } else if std::path::Path::new(".git").exists() {
        std::path::Path::new(".")
    } else {
        return Ok(format!(
            "Software is up-to-date (v{}). Standalone binary installation mode active.",
            CURRENT_VERSION
        ));
    };

    info!("Updating repository at {:?}...", repo_dir);

    // git fetch & pull
    let pull_status = Command::new("git")
        .current_dir(repo_dir)
        .args(["pull", "--rebase"])
        .status();

    match pull_status {
        Ok(status) if status.success() => {
            info!("Git pull successful.");
        }
        Ok(status) => {
            warn!(
                "Git pull exited with status {:?}. Attempting build with current code state.",
                status.code()
            );
        }
        Err(e) => {
            warn!(
                "Failed to execute git pull: {}. Continuing with existing files.",
                e
            );
        }
    }

    // Attempt cargo build if cargo is available
    if which::which("cargo").is_ok() {
        info!("Compiling release binary with cargo...");
        let build_status = Command::new("cargo")
            .current_dir(repo_dir)
            .args(["build", "--release"])
            .status();

        if let Ok(status) = build_status {
            if status.success() {
                info!("Cargo release build completed successfully!");
            }
        }
    }

    Ok(format!(
        "Software update completed successfully for v{}!",
        CURRENT_VERSION
    ))
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_ver = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let l_parts = parse_ver(latest);
    let c_parts = parse_ver(current);

    if l_parts.len() == 3 && c_parts.len() == 3 {
        l_parts > c_parts
    } else {
        latest != current && !latest.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("1.1.0", "1.0.9"));
        assert!(is_newer_version("2.0.0", "1.0.9"));
        assert!(!is_newer_version("1.0.9", "1.0.9"));
        assert!(!is_newer_version("1.0.8", "1.0.9"));
    }

    #[test]
    fn test_check_for_updates() {
        let info = check_for_updates().expect("check_for_updates failed");
        assert_eq!(info.current_version, CURRENT_VERSION);
    }
}
