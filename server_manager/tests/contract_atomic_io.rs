use server_manager::core::atomic_io::{atomic_write, atomic_write_str};
use std::fs;

#[test]
fn test_atomic_write_creates_file_with_content() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_atomic_io_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();

    let target_file = temp_dir.join("test_atomic.txt");
    let content = "Hello, zero-defect atomic world!";

    assert!(atomic_write_str(&target_file, content, 0o600).is_ok());

    let read_back = fs::read_to_string(&target_file).unwrap_or_default();
    assert_eq!(read_back, content);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = fs::metadata(&target_file).unwrap_or_else(|_| panic!("Metadata exists"));
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "File permissions must be exactly 0600");
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_atomic_write_overwrites_existing_file() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_atomic_overwrite_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();

    let target_file = temp_dir.join("secrets.yaml");
    assert!(atomic_write_str(&target_file, "initial: data", 0o600).is_ok());
    assert!(atomic_write_str(&target_file, "updated: secure_data", 0o600).is_ok());

    let read_back = fs::read_to_string(&target_file).unwrap_or_default();
    assert_eq!(read_back, "updated: secure_data");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_atomic_write_nested_directory_creation() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_atomic_nested_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    let nested_file = temp_dir.join("sub1").join("sub2").join("config.yaml");
    assert!(atomic_write_str(&nested_file, "domain: example.com", 0o644).is_ok());

    let read_back = fs::read_to_string(&nested_file).unwrap_or_default();
    assert_eq!(read_back, "domain: example.com");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_atomic_write_raw_bytes() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_atomic_bytes_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();
    let target_file = temp_dir.join("binary.bin");
    let bytes = &[0xde, 0xad, 0xbe, 0xef];

    assert!(atomic_write(&target_file, bytes, 0o600).is_ok());
    let read_back = fs::read(&target_file).unwrap_or_default();
    assert_eq!(read_back, bytes);

    let _ = fs::remove_dir_all(&temp_dir);
}
