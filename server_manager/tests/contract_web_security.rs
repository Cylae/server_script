use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use clap::Parser;
use server_manager::{
    core::{
        config::Config,
        users::{hash_password, verify_password, Role, UserManager},
        validate::{validate_ip, validate_port},
    },
    interface::{
        cli::{Cli, Commands},
        web::{build_app, AppState},
    },
};
use tower::ServiceExt;

#[test]
fn test_argon2id_hash_format_and_parameters() {
    // REQ-SEC-005: 64 MiB (m=65536), 3 iterations (t=3), 4 lanes (p=4)
    let password = "UltraSecureZeroDefect2026!";
    let hashed = hash_password(password).expect("Argon2id hashing should succeed");

    assert!(
        hashed.starts_with("$argon2id$v=19$m=65536,t=3,p=4$"),
        "Hash must contain exact Argon2id parameters: {}",
        hashed
    );

    // Verify correct password succeeds
    let is_valid = verify_password(password, &hashed);
    assert!(is_valid, "Valid password must verify successfully");

    // Verify wrong password fails
    let is_invalid = verify_password("WrongPassword123", &hashed);
    assert!(!is_invalid, "Wrong password must fail verification");
}

#[test]
fn test_transparent_bcrypt_migration_to_argon2id() {
    // REQ-SEC-005: In-flight transparent upgrade of legacy bcrypt hashes to Argon2id
    let password = "LegacyBcryptPassword123!";
    let legacy_bcrypt_hash = bcrypt::hash(password, 4).expect("bcrypt hash generation failed");
    assert!(
        legacy_bcrypt_hash.starts_with("$2"),
        "Must be legacy bcrypt hash"
    );

    let yaml = format!(
        r#"
users:
  legacy_bob:
    username: legacy_bob
    password_hash: "{}"
    role: Observer
    quota_gb: null
    installed_apps: []
"#,
        legacy_bcrypt_hash
    );
    let mut user_mgr: UserManager =
        serde_yaml_ng::from_str(&yaml).expect("Deserialization of legacy user");

    // Initial check: legacy verify works
    let initial_valid = verify_password(password, &legacy_bcrypt_hash);
    assert!(initial_valid, "Legacy hash must verify");

    // Test verify_and_migrate with wrong password: no migration
    let wrong_result = user_mgr.verify_and_migrate("legacy_bob", "BadPassword");
    assert!(wrong_result.is_none(), "Bad password should return None");
    assert_eq!(
        user_mgr.get_user("legacy_bob").unwrap().password_hash,
        legacy_bcrypt_hash,
        "Hash must not change on failed login"
    );

    // Test verify_and_migrate with correct password: transparent upgrade to Argon2id
    let migrated_result = user_mgr.verify_and_migrate("legacy_bob", password);
    assert!(
        migrated_result.is_some(),
        "Correct password must verify and return Some(User)"
    );

    let updated_user = user_mgr.get_user("legacy_bob").expect("User must exist");
    assert!(
        updated_user
            .password_hash
            .starts_with("$argon2id$v=19$m=65536,t=3,p=4$"),
        "Hash must be transparently upgraded to Argon2id: {}",
        updated_user.password_hash
    );

    // Verify subsequent login works with the newly migrated Argon2id hash
    let subsequent_valid = verify_password(password, &updated_user.password_hash);
    assert!(subsequent_valid, "New Argon2id hash must verify correctly");
}

#[test]
fn test_four_role_rbac_capability_matrix() {
    // REQ-SEC-008: Strict 4-role RBAC matrix (Admin, Operator, Observer, Auditor)
    let admin = Role::Admin;
    let operator = Role::Operator;
    let observer = Role::Observer;
    let auditor = Role::Auditor;

    // Admin: Full operational & administrative privileges
    assert!(admin.can_manage_users(), "Admin must manage users");
    assert!(admin.can_manage_services(), "Admin must manage services");
    assert!(admin.can_view_secrets(), "Admin must view secrets");
    assert!(admin.can_view_audit_logs(), "Admin must view audit logs");
    assert!(admin.can_trigger_updates(), "Admin must trigger updates");

    // Operator: Operational privileges (services, updates), no user admin or secret exposure
    assert!(
        !operator.can_manage_users(),
        "Operator must NOT manage users"
    );
    assert!(
        operator.can_manage_services(),
        "Operator must manage services"
    );
    assert!(
        !operator.can_view_secrets(),
        "Operator must NOT view secrets"
    );
    assert!(
        !operator.can_view_audit_logs(),
        "Operator must NOT view audit logs"
    );
    assert!(
        operator.can_trigger_updates(),
        "Operator must trigger updates"
    );

    // Observer: Read-only telemetry, no mutating actions
    assert!(
        !observer.can_manage_users(),
        "Observer must NOT manage users"
    );
    assert!(
        !observer.can_manage_services(),
        "Observer must NOT manage services"
    );
    assert!(
        !observer.can_view_secrets(),
        "Observer must NOT view secrets"
    );
    assert!(
        !observer.can_view_audit_logs(),
        "Observer must NOT view audit logs"
    );
    assert!(
        !observer.can_trigger_updates(),
        "Observer must NOT trigger updates"
    );

    // Auditor: Compliance & audit log inspection, no operational or administrative control
    assert!(!auditor.can_manage_users(), "Auditor must NOT manage users");
    assert!(
        !auditor.can_manage_services(),
        "Auditor must NOT manage services"
    );
    assert!(!auditor.can_view_secrets(), "Auditor must NOT view secrets");
    assert!(
        auditor.can_view_audit_logs(),
        "Auditor MUST view audit logs"
    );
    assert!(
        !auditor.can_trigger_updates(),
        "Auditor must NOT trigger updates"
    );
}

#[test]
fn test_cli_bind_flag_defaults_to_localhost() {
    // REQ-SEC-007: Default localhost binding 127.0.0.1:8099 with --bind override
    let cli_default =
        Cli::try_parse_from(["server_manager", "web"]).expect("Default web command parse");
    if let Commands::Web { bind, .. } = cli_default.command {
        assert_eq!(bind, "127.0.0.1", "Web command must default to 127.0.0.1");
    } else {
        panic!("Expected Commands::Web");
    }

    let cli_override = Cli::try_parse_from(["server_manager", "web", "--bind", "192.168.1.100"])
        .expect("Overridden bind parse");
    if let Commands::Web { bind, .. } = cli_override.command {
        assert_eq!(
            bind, "192.168.1.100",
            "--bind flag must override bind address"
        );
    } else {
        panic!("Expected Commands::Web");
    }

    // Input validation checks
    assert!(validate_ip("127.0.0.1").is_ok());
    assert!(validate_ip("0.0.0.0").is_ok());
    assert!(validate_ip("::1").is_ok());
    assert!(validate_ip("999.999.999.999").is_err());
    assert!(validate_ip("127.0.0.1; rm -rf /").is_err());
    assert!(validate_port(8099).is_ok());
    assert!(validate_port(0).is_err());
}

#[test]
fn test_four_roles_serialization_and_deserialization() {
    // Ensure all 4 roles round-trip faithfully through YAML and JSON
    let roles = vec![Role::Admin, Role::Operator, Role::Observer, Role::Auditor];

    for role in roles {
        let serialized = serde_yaml_ng::to_string(&role).expect("YAML serialization");
        let deserialized: Role =
            serde_yaml_ng::from_str(&serialized).expect("YAML deserialization");
        assert_eq!(
            role, deserialized,
            "YAML round-trip mismatch for {:?}",
            role
        );

        let json_serialized = serde_json::to_string(&role).expect("JSON serialization");
        let json_deserialized: Role =
            serde_json::from_str(&json_serialized).expect("JSON deserialization");
        assert_eq!(
            role, json_deserialized,
            "JSON round-trip mismatch for {:?}",
            role
        );
    }
}

#[tokio::test]
async fn test_http_security_headers_middleware() {
    // REQ-SEC-006: Hardened HTTP headers middleware
    let app_state = AppState::new_test(Config::default(), UserManager::default());
    let app = build_app(app_state);

    let req = Request::builder()
        .uri("/login")
        .method("GET")
        .body(Body::empty())
        .expect("Valid request");

    let response = app.oneshot(req).await.expect("App should respond");
    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert_eq!(
        headers
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    assert_eq!(
        headers.get("x-frame-options").and_then(|v| v.to_str().ok()),
        Some("DENY")
    );
    assert_eq!(
        headers
            .get("x-xss-protection")
            .and_then(|v| v.to_str().ok()),
        Some("1; mode=block")
    );
    assert_eq!(
        headers.get("referrer-policy").and_then(|v| v.to_str().ok()),
        Some("strict-origin-when-cross-origin")
    );
    assert_eq!(
        headers
            .get("strict-transport-security")
            .and_then(|v| v.to_str().ok()),
        Some("max-age=31536000; includeSubDomains")
    );
    assert!(
        headers
            .get("content-security-policy")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .contains("default-src 'self'"),
        "CSP header must restrict default sources to 'self'"
    );
}

#[tokio::test]
async fn test_unauthenticated_requests_redirected_to_login() {
    // REQ-SEC-006 & REQ-SEC-008: Protected routes require valid authenticated session
    let app_state = AppState::new_test(Config::default(), UserManager::default());
    let app = build_app(app_state);

    let protected_routes = vec!["/", "/users", "/updates", "/audit", "/user/profile"];

    for route in protected_routes {
        let req = Request::builder()
            .uri(route)
            .method("GET")
            .body(Body::empty())
            .expect("Valid request");

        let response = app.clone().oneshot(req).await.expect("App response");
        assert!(
            response.status().is_redirection(),
            "Route '{}' must redirect unauthenticated users to /login, got {}",
            route,
            response.status()
        );
        let location = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok());
        assert_eq!(
            location,
            Some("/login"),
            "Redirect destination for '{}' must be /login",
            route
        );
    }
}

#[tokio::test]
async fn test_mutating_requests_reject_missing_csrf() {
    // REQ-SEC-006: Mutating operations without valid CSRF must be rejected
    let app_state = AppState::new_test(Config::default(), UserManager::default());
    let app = build_app(app_state);

    // Attempting POST /logout without session/CSRF
    let req = Request::builder()
        .uri("/logout")
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("csrf_token=invalid_token_12345"))
        .expect("Valid request");

    let response = app.clone().oneshot(req).await.expect("App response");
    // Logout with invalid CSRF returns 403 Forbidden
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "POST /logout with invalid CSRF must return 403 Forbidden"
    );
}
