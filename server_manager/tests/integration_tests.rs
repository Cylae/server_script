use server_manager::core::hardware::{HardwareInfo, HardwareProfile};
use server_manager::core::secrets::Secrets;
use server_manager::core::config::Config;
use server_manager::build_compose_structure;

#[test]
fn test_generate_compose_structure() {
    // 1. Mock Hardware and Secrets
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
        mysql_root_password: Some("rootpass".to_string()),
        mysql_user_password: Some("userpass".to_string()),
        nextcloud_db_password: Some("nextcloudpass".to_string()),
        glpi_db_password: Some("glpipass".to_string()),
        gitea_db_password: Some("giteapass".to_string()),
        yourls_admin_password: Some("yourlspass".to_string()),
        mailserver_password: Some("mailpass".to_string()),
        nextcloud_admin_password: Some("nextcloudadmin".to_string()),
        roundcube_db_password: Some("roundcubepass".to_string()),
        vaultwarden_admin_token: Some("token".to_string()),
    };
    let config = Config::default();

    // 2. Build Structure
    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();

    // 3. Verify Top Level Keys (Struct fields exist by definition)

    // 4. Verify Networks
    assert!(compose.networks.contains_key("server_manager_net"));

    // 5. Verify Services Count (Should be 27)
    assert_eq!(compose.services.len(), 27, "Expected 27 services");

    // 6. Verify specific service (Plex, Jellyfin, Bazarr)
    assert!(compose.services.contains_key("plex"));
    assert!(compose.services.contains_key("jellyfin"));
    assert!(compose.services.contains_key("bazarr"));

    let plex = compose.services.get("plex").unwrap();
    assert_eq!(plex.image, "lscr.io/linuxserver/plex:latest");

    // 7. Verify Network attachment
    let plex_nets = plex.networks.as_ref().unwrap();
    assert!(plex_nets.contains(&"server_manager_net".to_string()));
}

#[test]
fn test_security_bindings() {
    // Test that sensitive services are bound to localhost and internal DBs have no ports
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
    let secrets = Secrets::default();
    let config = Config::default();

    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();

    // 1. MariaDB should have NO ports
    let mariadb = compose.services.get("mariadb").unwrap();
    assert!(mariadb.ports.is_none(), "MariaDB should not expose ports");

    // 2. Sonarr should be bound to 127.0.0.1
    let sonarr = compose.services.get("sonarr").unwrap();
    let ports = sonarr.ports.as_ref().unwrap();
    let port_str = &ports[0];
    assert!(port_str.starts_with("127.0.0.1:"), "Sonarr port should be bound to localhost: {}", port_str);

    // 3. Plex should still be exposed (host mapping implied or explicit 0.0.0.0)
    let plex = compose.services.get("plex").unwrap();
    let ports = plex.ports.as_ref().unwrap();
    let port_str = &ports[0];
    assert!(!port_str.starts_with("127.0.0.1:"), "Plex port should be exposed: {}", port_str);
}

#[test]
fn test_profile_logic_low() {
    // Test that Low profile disables SpamAssassin in MailService
    let hw = HardwareInfo {
        profile: HardwareProfile::Low,
        ram_gb: 2,
        cpu_cores: 2,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 100,
        swap_gb: 0,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };
    let secrets = Secrets::default();
    let config = Config::default();

    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();
    let mail = compose.services.get("mailserver").unwrap();
    let envs = mail.environment.as_ref().unwrap();

    // Check for ENABLE_SPAMASSASSIN=0
    let has_disabled_spam = envs.iter().any(|v| v == "ENABLE_SPAMASSASSIN=0");
    assert!(has_disabled_spam, "Low profile should disable SpamAssassin");
}

#[test]
fn test_profile_logic_standard() {
    // Test that Standard profile enables SpamAssassin
    let hw = HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 4,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };
    let secrets = Secrets::default();
    let config = Config::default();

    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();
    let mail = compose.services.get("mailserver").unwrap();
    let envs = mail.environment.as_ref().unwrap();

    // Check for ENABLE_SPAMASSASSIN=1
    let has_enabled_spam = envs.iter().any(|v| v == "ENABLE_SPAMASSASSIN=1");
    assert!(has_enabled_spam, "Standard profile should enable SpamAssassin");
}

#[test]
fn test_resource_generation() {
    // Test that resources are generated correctly for MariaDB on High Profile
    let hw = HardwareInfo {
        profile: HardwareProfile::High,
        ram_gb: 32,
        cpu_cores: 16,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 1000,
        swap_gb: 4,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };
    let secrets = Secrets::default();
    let config = Config::default();

    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();
    let mariadb = compose.services.get("mariadb").unwrap();

    // Check deploy key exists
    assert!(mariadb.deploy.is_some());

    let deploy = mariadb.deploy.as_ref().unwrap();
    let resources = deploy.resources.as_ref().unwrap();
    let limits = resources.limits.as_ref().unwrap();

    let memory = limits.memory.as_ref().unwrap();
    assert_eq!(memory, "4G", "MariaDB should have 4G limit on High profile");
}

#[test]
fn test_disabled_service_filtering() {
    let hw = HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 4,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };
    let secrets = Secrets::default();

    // Disable "plex"
    let mut config = Config::default();
    config.disable_service("plex");

    let compose = build_compose_structure(&hw, &secrets, &config).unwrap();

    assert!(!compose.services.contains_key("plex"), "Plex should be disabled");
    assert!(compose.services.contains_key("jellyfin"), "Jellyfin should still be enabled");
}
