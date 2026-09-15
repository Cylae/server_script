use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// Persist a complete file with explicit permissions and file + directory fsync.
/// Callers must additionally lock an entire read-modify-write transaction.
pub fn atomic_write<P: AsRef<Path>>(path: P, content: &[u8], mode: u32) -> Result<()> {
    let dest = path.as_ref();
    let parent = dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = dest.file_name().context("Destination must name a file")?;
    fs::create_dir_all(parent).context("Failed to create output directory")?;
    let lock_path = parent.join(format!(".{}.lock", name.to_string_lossy()));
    let _lock = crate::core::lock::ProcessLock::acquire(lock_path, false)?;

    // Preserve timestamps on idempotent writes; never follow destination symlinks.
    if let Ok(meta) = fs::symlink_metadata(dest) {
        if meta.is_file() && fs::read(dest).is_ok_and(|old| old == content) {
            #[cfg(unix)]
            if meta.permissions().mode() & 0o777 != mode {
                let file = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW)
                    .open(dest)?;
                file.set_permissions(fs::Permissions::from_mode(mode))?;
                file.sync_all()?;
            }
            return Ok(());
        }
    }

    let tmp_path = parent.join(format!(
        ".tmp.{}.{:032x}",
        name.to_string_lossy(),
        rand::random::<u128>()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(mode);
    // Only remove a temporary file after this process successfully created it.
    let mut file = options
        .open(&tmp_path)
        .context("Failed to create temporary output")?;
    let result = (|| -> Result<()> {
        #[cfg(unix)]
        file.set_permissions(fs::Permissions::from_mode(mode))
            .context("Failed to set output permissions")?;
        file.write_all(content).context("Failed to write output")?;
        file.sync_all().context("Failed to fsync output")?;
        fs::rename(&tmp_path, dest).context("Failed to atomically replace output")?;
        #[cfg(unix)]
        fs::File::open(parent)?
            .sync_all()
            .context("Failed to fsync output directory")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

pub fn atomic_write_str<P: AsRef<Path>>(path: P, content: &str, mode: u32) -> Result<()> {
    atomic_write(path, content.as_bytes(), mode)
}
