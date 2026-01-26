use server_manager::core::hardware::{HardwareInfo, HardwareProfile};
use server_manager::core::secrets::Secrets;
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

    // 2. Build Structure
    let result = build_compose_structure(&hw, &secrets);
    assert!(result.is_ok(), "Failed to build compose structure");
    let mapping = result.unwrap();

    // 3. Verify Top Level Keys
    assert!(mapping.contains_key(&serde_yaml::Value::from("services")));
    assert!(mapping.contains_key(&serde_yaml::Value::from("networks")));

    // 4. Verify Networks
    let networks = mapping.get(&serde_yaml::Value::from("networks")).unwrap().as_mapping().unwrap();
    assert!(networks.contains_key(&serde_yaml::Value::from("server_manager_net")));

    // 5. Verify Services Count (Should be 24)
    let services = mapping.get(&serde_yaml::Value::from("services")).unwrap().as_mapping().unwrap();
    assert_eq!(services.len(), 24, "Expected 24 services");

    // 6. Verify specific service (Plex)
    assert!(services.contains_key(&serde_yaml::Value::from("plex")));
    let plex = services.get(&serde_yaml::Value::from("plex")).unwrap().as_mapping().unwrap();
    assert_eq!(plex.get(&serde_yaml::Value::from("image")).unwrap().as_str().unwrap(), "lscr.io/linuxserver/plex:latest");

    // 7. Verify Network attachment
    let plex_nets = plex.get(&serde_yaml::Value::from("networks")).unwrap().as_sequence().unwrap();
    assert!(plex_nets.contains(&serde_yaml::Value::from("server_manager_net")));
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
    };
    let secrets = Secrets::default();

    let result = build_compose_structure(&hw, &secrets).unwrap();
    let services = result.get(&serde_yaml::Value::from("services")).unwrap().as_mapping().unwrap();

    let mail = services.get(&serde_yaml::Value::from("mailserver")).unwrap().as_mapping().unwrap();
    let envs = mail.get(&serde_yaml::Value::from("environment")).unwrap().as_sequence().unwrap();

    // Check for ENABLE_SPAMASSASSIN=0
    let has_disabled_spam = envs.iter().any(|v| v.as_str().unwrap() == "ENABLE_SPAMASSASSIN=0");
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
    };
    let secrets = Secrets::default();

    let result = build_compose_structure(&hw, &secrets).unwrap();
    let services = result.get(&serde_yaml::Value::from("services")).unwrap().as_mapping().unwrap();

    let mail = services.get(&serde_yaml::Value::from("mailserver")).unwrap().as_mapping().unwrap();
    let envs = mail.get(&serde_yaml::Value::from("environment")).unwrap().as_sequence().unwrap();

    // Check for ENABLE_SPAMASSASSIN=1
    let has_enabled_spam = envs.iter().any(|v| v.as_str().unwrap() == "ENABLE_SPAMASSASSIN=1");
    assert!(has_enabled_spam, "Standard profile should enable SpamAssassin");
}
