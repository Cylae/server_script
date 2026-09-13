use anyhow::{Context, Result};
use fs3::FileExt;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

/// Advisory inter-process lock to guard against concurrent mutating operations.
pub struct ProcessLock {
    _file: std::fs::File,
    path: PathBuf,
}

impl ProcessLock {
    /// Attempts to acquire an exclusive lock on the specified path.
    /// If `non_blocking` is true and the lock is already held, returns an error immediately.
    pub fn acquire<P: AsRef<Path>>(path: P, non_blocking: bool) -> Result<Self> {
        let target = path.as_ref();
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).context("Failed to create lock directory")?;
        }

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }

        let file = options
            .open(target)
            .with_context(|| format!("Failed to open lockfile {}", target.display()))?;

        anyhow::ensure!(file.metadata()?.is_file(), "Lock must be a regular file");

        if non_blocking {
            if let Err(err) = file.try_lock_exclusive() {
                anyhow::bail!(
                    "Advisory lock is already held by another process on {} (error: {})",
                    target.display(),
                    err
                );
            }
        } else {
            file.lock_exclusive()
                .with_context(|| format!("Failed to acquire lock on {}", target.display()))?;
        }

        Ok(Self {
            _file: file,
            path: target.to_path_buf(),
        })
    }

    /// Acquires the default server_manager advisory lock.
    pub fn acquire_default() -> Result<Self> {
        let lock_path = if Path::new("/var/lock").exists() {
            PathBuf::from("/var/lock/server_manager.lock")
        } else {
            std::env::temp_dir().join("server_manager.lock")
        };
        Self::acquire(&lock_path, true)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ProcessLock {
    fn drop(&mut self) {
        let _ = self._file.unlock();
    }
}
