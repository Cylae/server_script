use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::Builder;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Atomically writes content to a file using secure temporary file creation.
///
/// Steps:
/// 1. Creates a secure temporary file in the same directory as `path` using `tempfile::Builder` (guaranteeing same filesystem/mount and unguessable filename).
/// 2. Sets explicit permissions on creation (e.g. 0600 or 0644 on Unix).
/// 3. Writes content and flushes buffers.
/// 4. Synchronizes to disk via `fsync` (`sync_all`).
/// 5. Atomically persists/renames the temporary file to the destination path.
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

    let prefix = format!(".tmp.{}.", file_name);

    let mut temp_file = Builder::new()
        .prefix(&prefix)
        .tempfile_in(parent)
        .with_context(|| {
            format!(
                "Failed to create secure temporary file in {}",
                parent.display()
            )
        })?;

    #[cfg(unix)]
    {
        let perms = fs::Permissions::from_mode(mode);
        fs::set_permissions(temp_file.path(), perms).with_context(|| {
            format!(
                "Failed to set permissions on temporary file {}",
                temp_file.path().display()
            )
        })?;
    }

    temp_file.write_all(content).with_context(|| {
        format!(
            "Failed to write content to temporary file {}",
            temp_file.path().display()
        )
    })?;
    temp_file.flush().with_context(|| {
        format!(
            "Failed to flush temporary file {}",
            temp_file.path().display()
        )
    })?;
    temp_file.as_file().sync_all().with_context(|| {
        format!(
            "Failed to fsync temporary file {}",
            temp_file.path().display()
        )
    })?;

    temp_file.persist(dest).with_context(|| {
        format!(
            "Failed to atomically persist temporary file to {}",
            dest.display()
        )
    })?;

    Ok(())
}

/// Helper function to atomically write a string slice.
pub fn atomic_write_str<P: AsRef<Path>>(path: P, content: &str, mode: u32) -> Result<()> {
    atomic_write(path, content.as_bytes(), mode)
}
