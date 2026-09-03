use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

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
            let _ = std::fs::create_dir_all(parent);
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(target)
            .with_context(|| format!("Failed to open lockfile {}", target.display()))?;

        #[cfg(unix)]
        {
            let fd = file.as_raw_fd();
            let mut flag = libc::LOCK_EX;
            if non_blocking {
                flag |= libc::LOCK_NB;
            }
            // SAFETY: fd is valid and owned by file.
            let res = unsafe { libc::flock(fd, flag) };
            if res != 0 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EWOULDBLOCK)
                    || err.raw_os_error() == Some(libc::EAGAIN)
                {
                    anyhow::bail!(
                        "Advisory lock is already held by another process on {}",
                        target.display()
                    );
                }
                return Err(err)
                    .with_context(|| format!("Failed to acquire lock on {}", target.display()));
            }
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
        #[cfg(unix)]
        {
            let fd = self._file.as_raw_fd();
            // SAFETY: fd is valid until file is dropped after this block.
            unsafe {
                libc::flock(fd, libc::LOCK_UN);
            }
        }
    }
}
