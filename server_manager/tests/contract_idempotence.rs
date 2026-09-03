use server_manager::build_compose_structure;
use server_manager::core::config::Config;
use server_manager::core::hardware::{HardwareInfo, HardwareProfile};
use server_manager::core::secrets::Secrets;
use server_manager::core::users::{Role, UserManager};
use std::collections::HashSet;
use std::fs;

#[test]
fn test_config_save_idempotence() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_idem_cfg_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();
    let config_file = temp_dir.join("config.yaml");

    let config = Config {
        disabled_services: HashSet::from(["vaultwarden".to_string()]),
    };

    let yaml1 = serde_yaml_ng::to_string(&config).unwrap_or_default();
    server_manager::core::atomic_io::atomic_write_str(&config_file, &yaml1, 0o644)
        .unwrap_or_default();
    let bytes_run1 = fs::read(&config_file).unwrap_or_default();

    // Run 2: Write exact same config again
    let yaml2 = serde_yaml_ng::to_string(&config).unwrap_or_default();
    server_manager::core::atomic_io::atomic_write_str(&config_file, &yaml2, 0o644)
        .unwrap_or_default();
    let bytes_run2 = fs::read(&config_file).unwrap_or_default();

    assert_eq!(
        bytes_run1, bytes_run2,
        "Consecutive config writes must produce byte-identical files"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_user_manager_add_user_idempotent_duplicate_protection() {
    let mut manager = UserManager::default();
    assert!(manager
        .add_user("admin", "InitialPassword123!", Role::Admin, None)
        .is_ok());

    // Attempting to add the exact same user again should gracefully error without mutating
    let res = manager.add_user("admin", "InitialPassword123!", Role::Admin, None);
    assert!(res.is_err(), "Duplicate user addition must be rejected");

    let user = manager.get_user("admin");
    assert!(user.is_some(), "Existing user must remain intact");
}

#[test]
fn test_compose_generation_byte_determinism_run_twice() {
    let hw = HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 2,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };

    let secrets = Secrets {
        mysql_root_password: Some("rootpass123".to_string()),
        mysql_user_password: Some("userpass123".to_string()),
        nextcloud_admin_password: Some("adminpass123".to_string()),
        nextcloud_db_password: Some("dbpass123".to_string()),
        mailserver_password: Some("mailpass123".to_string()),
        glpi_db_password: Some("glpipass123".to_string()),
        gitea_db_password: Some("giteapass123".to_string()),
        roundcube_db_password: Some("rcpass123".to_string()),
        yourls_admin_password: Some("yourlspass123".to_string()),
        vaultwarden_admin_token: Some("token123".to_string()),
        server_manager_admin_password: Some("serveradmin123".to_string()),
    };

    let config = Config::default();

    let compose1 = build_compose_structure(&hw, &secrets, &config).unwrap_or_else(|e| {
        panic!("First compose generation failed: {}", e);
    });
    let yaml1 = serde_yaml_ng::to_string(&compose1).unwrap_or_default();

    let compose2 = build_compose_structure(&hw, &secrets, &config).unwrap_or_else(|e| {
        panic!("Second compose generation failed: {}", e);
    });
    let yaml2 = serde_yaml_ng::to_string(&compose2).unwrap_or_default();

    assert_eq!(
        yaml1, yaml2,
        "Consecutive compose generation must produce byte-identical YAML"
    );
}
