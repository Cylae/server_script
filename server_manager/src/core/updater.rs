use crate::core::atomic_io;
use anyhow::{bail, Context, Result};
use log::info;
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

/// Executes software self-update by pulling latest source changes, rebuilding,
/// and atomically installing the new binary over the currently running one.
///
/// CORRECTNESS/SECURITY (fixes A10): every prior failure mode here — a failed
/// `git pull`, a missing/failing `cargo build`, or never installing the built
/// binary — was swallowed, and the function unconditionally reported success.
/// Each step below now fails closed (`bail!`) on error, and the function only
/// returns `Ok` once the new binary has actually been installed.
pub fn self_update() -> Result<String> {
    info!("Starting software self-update procedure...");

    let repo_dir = if std::path::Path::new("/opt/server_manager/.git").exists() {
        std::path::Path::new("/opt/server_manager")
    } else if std::path::Path::new(".git").exists() {
        std::path::Path::new(".")
    } else {
        return Ok(format!(
            "Software is up-to-date (v{}). Standalone binary installation mode active; \
             self-update is unavailable outside a git checkout.",
            CURRENT_VERSION
        ));
    };

    info!("Updating repository at {:?}...", repo_dir);

    // git fetch & pull — a failure here means we would rebuild stale (or, in a
    // conflicted rebase, inconsistent) source, so it must abort the update.
    let pull_status = Command::new("git")
        .current_dir(repo_dir)
        .args(["pull", "--rebase"])
        .status()
        .context("Failed to execute git pull")?;

    if !pull_status.success() {
        bail!(
            "git pull --rebase exited with status {:?}; aborting self-update to avoid building \
             from a stale or conflicted working tree. Resolve the repository state manually.",
            pull_status.code()
        );
    }
    info!("Git pull successful.");

    // A rebuild requires cargo; without it we cannot safely produce a new binary.
    which::which("cargo").context(
        "cargo is not available on PATH; cannot rebuild for self-update. Install the Rust \
         toolchain or update via a packaged release instead.",
    )?;

    info!("Compiling release binary with cargo...");
    let build_status = Command::new("cargo")
        .current_dir(repo_dir)
        .args(["build", "--release"])
        .status()
        .context("Failed to execute cargo build --release")?;

    if !build_status.success() {
        bail!(
            "cargo build --release exited with status {:?}; the currently installed binary was \
             left untouched.",
            build_status.code()
        );
    }
    info!("Cargo release build completed successfully.");

    // Install the freshly built binary over the one currently running.
    let built_binary = repo_dir.join("server_manager/target/release/server_manager");
    let built_binary = if built_binary.exists() {
        built_binary
    } else {
        repo_dir.join("target/release/server_manager")
    };
    if !built_binary.exists() {
        bail!(
            "Build reported success but the expected binary was not found at {:?}; refusing to \
             report a completed update.",
            built_binary
        );
    }

    let current_exe = std::env::current_exe()
        .context("Failed to determine path of the currently running binary")?;
    let new_binary_bytes = std::fs::read(&built_binary)
        .with_context(|| format!("Failed to read newly built binary at {:?}", built_binary))?;

    // Atomic write + rename means an in-flight `server_manager` process keeps
    // running against its original (now-unlinked) inode; the new binary takes
    // effect on next invocation, with no window where the path is missing.
    atomic_io::atomic_write(&current_exe, &new_binary_bytes, 0o755).with_context(|| {
        format!(
            "Failed to atomically install new binary to {:?}; the previously installed binary \
             was left untouched.",
            current_exe
        )
    })?;

    info!("New binary installed at {:?}.", current_exe);

    Ok(format!(
        "Software update completed successfully for v{}! Restart server_manager to run the new \
         version.",
        CURRENT_VERSION
    ))
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_ver = |v: &str| -> Option<Vec<u32>> {
        let trimmed = v.trim_start_matches('v');
        if trimmed.is_empty() {
            return None;
        }
        let parts: Result<Vec<u32>, _> = trimmed.split('.').map(|s| s.parse::<u32>()).collect();
        let parts = parts.ok()?;
        if parts.is_empty() || parts.len() > 4 {
            None
        } else {
            Some(parts)
        }
    };

    match (parse_ver(latest), parse_ver(current)) {
        (Some(l_parts), Some(c_parts)) => l_parts > c_parts,
        _ => false, // Fail closed on malformed strings or downgrades
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
        assert!(!is_newer_version("0.9.0", "1.0.9"));
        assert!(!is_newer_version("invalid", "1.0.9"));
        assert!(!is_newer_version("1.0.9", "invalid"));
        assert!(!is_newer_version("", "1.0.9"));
        assert!(is_newer_version("v1.2.0", "v1.1.0"));
    }

    #[test]
    fn test_check_for_updates() {
        let info = check_for_updates().expect("Checked error condition in code");
        assert_eq!(info.current_version, CURRENT_VERSION);
    }
}
