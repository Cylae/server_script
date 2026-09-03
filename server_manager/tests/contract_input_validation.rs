use server_manager::core::validate::{
    validate_domain, validate_ip, validate_port, validate_port_str, validate_safe_path,
    validate_service_name, validate_username,
};
use std::path::Path;

#[test]
fn test_validate_service_name() {
    // Valid names
    assert!(validate_service_name("vaultwarden").is_ok());
    assert!(validate_service_name("plex-media").is_ok());
    assert!(validate_service_name("nextcloud_db").is_ok());
    assert!(validate_service_name("traefik2").is_ok());

    // Invalid names / Command injection attempts
    assert!(validate_service_name("").is_err());
    assert!(validate_service_name("service;rm -rf /").is_err());
    assert!(validate_service_name("service | cat /etc/shadow").is_err());
    assert!(validate_service_name("service`whoami`").is_err());
    assert!(validate_service_name("service$(id)").is_err());
    assert!(validate_service_name("service name with spaces").is_err());
    assert!(validate_service_name("ServiceWithUppercase").is_err());
    assert!(validate_service_name("../../etc/passwd").is_err());

    // Exceeds 64 chars
    let long_name = "a".repeat(65);
    assert!(validate_service_name(&long_name).is_err());
}

#[test]
fn test_validate_username() {
    // Valid usernames
    assert!(validate_username("admin").is_ok());
    assert!(validate_username("john_doe").is_ok());
    assert!(validate_username("_service").is_ok());
    assert!(validate_username("user1").is_ok());

    // Invalid usernames / Injection attempts
    assert!(validate_username("").is_err());
    assert!(validate_username("1user").is_err()); // Cannot start with digit
    assert!(validate_username("-user").is_err()); // Cannot start with hyphen
    assert!(validate_username("user;id").is_err());
    assert!(validate_username("user`id`").is_err());
    assert!(validate_username("user$(id)").is_err());
    assert!(validate_username("user/admin").is_err());
    assert!(validate_username("../root").is_err());

    // Exceeds 32 chars
    let long_user = "a".repeat(33);
    assert!(validate_username(&long_user).is_err());
}

#[test]
fn test_validate_domain() {
    // Valid domains
    assert!(validate_domain("example.com").is_ok());
    assert!(validate_domain("cloud.sub.example.org").is_ok());
    assert!(validate_domain("my-server-01.local").is_ok());
    assert!(validate_domain("localhost").is_ok());

    // Invalid domains
    assert!(validate_domain("").is_err());
    assert!(validate_domain("-leading-hyphen.com").is_err());
    assert!(validate_domain("trailing-hyphen-.com").is_err());
    assert!(validate_domain("double..dot.com").is_err());
    assert!(validate_domain("domain;curl evil.com").is_err());
    assert!(validate_domain("domain with spaces.com").is_err());
}

#[test]
fn test_validate_port() {
    // Valid ports
    assert_eq!(validate_port(1).unwrap_or_default(), 1);
    assert_eq!(validate_port(80).unwrap_or_default(), 80);
    assert_eq!(validate_port(443).unwrap_or_default(), 443);
    assert_eq!(validate_port(8080).unwrap_or_default(), 8080);
    assert_eq!(validate_port(65535).unwrap_or_default(), 65535);

    // Invalid ports
    assert!(validate_port(0).is_err());
    assert!(validate_port(65536).is_err());
    assert!(validate_port(100000).is_err());

    // String parsing
    assert_eq!(validate_port_str("8099").unwrap_or_default(), 8099);
    assert!(validate_port_str("abc").is_err());
    assert!(validate_port_str("80;rm -rf").is_err());
}

#[test]
fn test_validate_ip() {
    // Valid IPs
    assert!(validate_ip("127.0.0.1").is_ok());
    assert!(validate_ip("192.168.1.100").is_ok());
    assert!(validate_ip("::1").is_ok());
    assert!(validate_ip("fe80::1").is_ok());

    // Invalid IPs
    assert!(validate_ip("").is_err());
    assert!(validate_ip("256.1.1.1").is_err());
    assert!(validate_ip("127.0.0.1;cat /etc/passwd").is_err());
    assert!(validate_ip("evil.domain.com").is_err());
}

#[test]
fn test_validate_safe_path() {
    // Valid paths
    assert!(validate_safe_path(Path::new("users.yaml")).is_ok());
    assert!(validate_safe_path(Path::new("/opt/server_manager/config.yaml")).is_ok());
    assert!(validate_safe_path(Path::new("subdir/nested/file.txt")).is_ok());

    // Traversal attempts
    assert!(validate_safe_path(Path::new("../secret")).is_err());
    assert!(validate_safe_path(Path::new("/opt/server_manager/../../etc/shadow")).is_err());
    assert!(validate_safe_path(Path::new("a/b/../../../etc/passwd")).is_err());
}
