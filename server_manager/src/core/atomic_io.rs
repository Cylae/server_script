use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

/// Atomically writes content to a file.
///
/// Steps:
/// 1. Creates a temporary file in the same directory as `path` (guaranteeing same filesystem/mount).
/// 2. Sets explicit permissions on creation (e.g. 0600 or 0644 on Unix).
/// 3. Writes content and flushes buffers.
/// 4. Synchronizes to disk via `fsync` (`sync_all`).
/// 5. Atomically renames the temporary file to the destination path.
pub fn atomic_write<P: AsRef<Path>>(path: P, content: &[u8], mode: u32) -> Result<()> {
    let dest = path.as_ref();
    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory {}", parent.display()))?;
    }

    let file_name = dest
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "temp_file".to_string());

    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let tmp_name = format!(".tmp.{}.{}.{}", file_name, pid, nanos);
    let tmp_path = parent.join(tmp_name);

    let write_result = (|| -> Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);

        #[cfg(unix)]
        options.mode(mode);

        let mut file = options
            .open(&tmp_path)
            .with_context(|| format!("Failed to create temporary file {}", tmp_path.display()))?;

        file.write_all(content)
            .with_context(|| format!("Failed to write content to {}", tmp_path.display()))?;
        file.flush()
            .with_context(|| format!("Failed to flush temporary file {}", tmp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("Failed to fsync temporary file {}", tmp_path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(mode));
        }

        fs::rename(&tmp_path, dest).with_context(|| {
            format!(
                "Failed to atomically rename {} to {}",
                tmp_path.display(),
                dest.display()
            )
        })?;

        Ok(())
    })();

    if write_result.is_err() && tmp_path.exists() {
        let _ = fs::remove_file(&tmp_path);
    }

    write_result
}

/// Helper function to atomically write a string slice.
pub fn atomic_write_str<P: AsRef<Path>>(path: P, content: &str, mode: u32) -> Result<()> {
    atomic_write(path, content.as_bytes(), mode)
}
