use server_manager::core::secrets::Secrets;
use server_manager::core::users::{Role, User};
use std::collections::HashSet;

#[test]
fn test_secrets_redaction_in_debug_format() {
    let raw_secret = "super_secret_password_12345";
    let secrets = Secrets {
        mysql_root_password: Some(raw_secret.to_string()),
        mysql_user_password: Some(raw_secret.to_string()),
        nextcloud_admin_password: Some(raw_secret.to_string()),
        nextcloud_db_password: Some(raw_secret.to_string()),
        mailserver_password: Some(raw_secret.to_string()),
        glpi_db_password: Some(raw_secret.to_string()),
        gitea_db_password: Some(raw_secret.to_string()),
        roundcube_db_password: Some(raw_secret.to_string()),
        yourls_admin_password: Some(raw_secret.to_string()),
        vaultwarden_admin_token: Some(raw_secret.to_string()),
        server_manager_admin_password: Some(raw_secret.to_string()),
    };

    let debug_output = format!("{:?}", secrets);

    // Verify plaintext secret does NOT appear in debug representation
    assert!(
        !debug_output.contains(raw_secret),
        "Plaintext secret must never appear in Debug output"
    );
    assert!(
        debug_output.contains("[REDACTED]"),
        "Secret fields must be formatted as [REDACTED]"
    );
}

#[test]
fn test_user_password_hash_redaction_in_debug_format() {
    let raw_hash = "$2b$12$e8O0e3wVvjT3v3h/5D3vauv6jS3.K7XG4Q5R1xZ3v1u5w9e7";
    let user = User {
        username: "admin".to_string(),
        password_hash: raw_hash.to_string(),
        role: Role::Admin,
        quota_gb: None,
        installed_apps: HashSet::new(),
    };

    let debug_output = format!("{:?}", user);

    assert!(
        !debug_output.contains(raw_hash),
        "Password hash must never appear in Debug output"
    );
    assert!(
        debug_output.contains("[REDACTED]"),
        "Password hash must be formatted as [REDACTED]"
    );
}
