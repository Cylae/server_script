use server_manager::core::{
    atomic_io::atomic_write, config::Config, lock::ProcessLock, secrets::Secrets,
};
use std::{fs, path::PathBuf, sync::Arc};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "server-manager-regression-{:032x}",
            rand::random::<u128>()
        ));
        fs::create_dir_all(&path).expect("failed to create dir");
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn regression_a07_atomic_write_preserves_bytes_and_mtime_on_repetition() -> anyhow::Result<()> {
    let dir = Directory::new();
    let path = dir.0.join("data");
    atomic_write(&path, b"first", 0o600)?;
    let modified = fs::metadata(&path)?.modified()?;
    atomic_write(&path, b"first", 0o600)?;
    assert_eq!(modified, fs::metadata(&path)?.modified()?);
    atomic_write(&path, b"second", 0o600)?;
    assert_eq!(fs::read(&path)?, b"second");
    assert!(!fs::read_dir(&dir.0)?.any(|entry| entry
        .expect("read_dir failure")
        .file_name()
        .to_string_lossy()
        .starts_with(".tmp.")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn regression_a07_rejects_symlink_lock_without_touching_target() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;
    let dir = Directory::new();
    let victim = dir.0.join("victim");
    fs::write(&victim, "unchanged")?;
    let lock = dir.0.join("lock");
    symlink(&victim, &lock)?;
    assert!(ProcessLock::acquire(&lock, true).is_err());
    assert_eq!(fs::read_to_string(victim)?, "unchanged");
    Ok(())
}

#[cfg(unix)]
#[test]
fn regression_a02_existing_secrets_permissions_are_repaired_without_rotation() -> anyhow::Result<()>
{
    use std::os::unix::fs::PermissionsExt;
    let dir = Directory::new();
    let path = dir.0.join("secrets.yaml");
    let first = Secrets::load_or_create_at(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644))?;
    let second = Secrets::load_or_create_at(&path)?;
    assert_eq!(
        first.server_manager_admin_password,
        second.server_manager_admin_password
    );
    assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
    Ok(())
}

#[test]
fn regression_a07_corrupt_state_is_never_overwritten() -> anyhow::Result<()> {
    let dir = Directory::new();
    let path = dir.0.join("config.yaml");
    let corrupt = b"disabled_services: [unclosed";
    fs::write(&path, corrupt)?;
    assert!(Config::update_service_at(&path, "plex", |cfg, name| cfg
        .disabled_services
        .insert(name.into()))
    .is_err());
    assert_eq!(fs::read(&path)?, corrupt);
    let secret_path = dir.0.join("secrets.yaml");
    let secret = "mysql_root_password: [PRIVATE_SENTINEL";
    fs::write(&secret_path, secret)?;
    let error = Secrets::load_or_create_at(&secret_path).unwrap_err();
    assert!(!format!("{error:#}").contains("PRIVATE_SENTINEL"));
    assert_eq!(fs::read_to_string(&secret_path)?, secret);
    Ok(())
}

#[test]
fn regression_a07_concurrent_config_updates_do_not_lose_changes() -> anyhow::Result<()> {
    let dir = Directory::new();
    let path = Arc::new(dir.0.join("config.yaml"));
    let barrier = Arc::new(std::sync::Barrier::new(8));
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let path = Arc::clone(&path);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                Config::update_service_at(&path, &format!("service{i}"), |cfg, name| {
                    cfg.disabled_services.insert(name.into())
                })
                .expect("Failed to update config");
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap_or_else(|_| panic!("Thread panicked"));
    }
    assert_eq!(Config::load_from(&path)?.disabled_services.len(), 8);
    Ok(())
}

#[test]
fn regression_a11_config_reads_use_the_requested_file() -> anyhow::Result<()> {
    let dir = Directory::new();
    let first = dir.0.join("first.yaml");
    let second = dir.0.join("second.yaml");
    fs::write(&first, "disabled_services: [plex]\n")?;
    fs::write(&second, "disabled_services: [jellyfin]\n")?;
    assert!(!Config::load_from(&first)?.is_enabled("plex"));
    assert!(Config::load_from(&second)?.is_enabled("plex"));
    Ok(())
}
