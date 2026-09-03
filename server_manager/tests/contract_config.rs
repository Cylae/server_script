use server_manager::core::config::Config;
use std::collections::HashSet;

#[test]
fn test_config_default_and_service_toggles() {
    let mut config = Config::default();
    assert!(config.disabled_services.is_empty());
    assert!(config.is_enabled("plex"));
    assert!(config.is_enabled("sonarr"));

    config.disable_service("plex");
    assert!(!config.is_enabled("plex"));
    assert!(config.is_enabled("sonarr"));

    // Redundant disable is idempotent
    config.disable_service("plex");
    assert!(!config.is_enabled("plex"));

    // Enable restores it
    config.enable_service("plex");
    assert!(config.is_enabled("plex"));

    // Redundant enable is idempotent
    config.enable_service("plex");
    assert!(config.is_enabled("plex"));
}

#[test]
fn test_config_serialization_and_deserialization() {
    let mut config = Config::default();
    config.disable_service("radarr");
    config.disable_service("bazarr");

    let yaml = serde_yaml_ng::to_string(&config).expect("YAML serialization should succeed");
    let deserialized: Config =
        serde_yaml_ng::from_str(&yaml).expect("YAML deserialization should succeed");

    assert_eq!(deserialized.disabled_services.len(), 2);
    assert!(!deserialized.is_enabled("radarr"));
    assert!(!deserialized.is_enabled("bazarr"));
    assert!(deserialized.is_enabled("sonarr"));
}

#[tokio::test]
async fn test_async_config_service_updates() {
    let tmp_path =
        std::env::temp_dir().join(format!("test_config_{:08x}.yaml", rand::random::<u32>()));

    // Initially no file exists, default is loaded
    let mut set = HashSet::new();
    set.insert("qbittorrent".to_string());
    let initial_config = Config {
        disabled_services: set,
    };
    let yaml = serde_yaml_ng::to_string(&initial_config).expect("serialize");
    std::fs::write(&tmp_path, yaml).expect("write initial file");

    let loaded: Config =
        serde_yaml_ng::from_str(&std::fs::read_to_string(&tmp_path).expect("read"))
            .expect("deserialize");
    assert!(!loaded.is_enabled("qbittorrent"));
    assert!(loaded.is_enabled("plex"));

    let _ = std::fs::remove_file(&tmp_path);
}
