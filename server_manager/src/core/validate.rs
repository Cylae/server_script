use anyhow::{bail, ensure, Result};
use std::net::IpAddr;
use std::path::{Component, Path};

/// Validates a service name: `^[a-z0-9_-]{1,64}$`
pub fn validate_service_name(name: &str) -> Result<&str> {
    ensure!(
        !name.is_empty(),
        "Validation error: service name cannot be empty"
    );
    ensure!(
        name.len() <= 64,
        "Validation error: service name cannot exceed 64 characters (got {})",
        name.len()
    );

    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
            bail!(
                "Validation error: invalid character '{}' in service name '{}'. Only [a-z0-9_-] allowed.",
                c,
                name
            );
        }
    }

    Ok(name)
}

/// Validates a username: `^[a-z_][a-z0-9_-]{0,31}$`
pub fn validate_username(name: &str) -> Result<&str> {
    ensure!(
        !name.is_empty(),
        "Validation error: username cannot be empty"
    );
    ensure!(
        name.len() <= 32,
        "Validation error: username cannot exceed 32 characters (got {})",
        name.len()
    );

    let first = name.chars().next().unwrap_or('\0');
    if !first.is_ascii_lowercase() && first != '_' {
        bail!(
            "Validation error: username must start with [a-z_], got '{}'",
            first
        );
    }

    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
            bail!(
                "Validation error: invalid character '{}' in username '{}'. Only [a-z0-9_-] allowed.",
                c,
                name
            );
        }
    }

    Ok(name)
}

/// Validates an RFC 1123 domain name.
pub fn validate_domain(domain: &str) -> Result<&str> {
    ensure!(
        !domain.is_empty(),
        "Validation error: domain cannot be empty"
    );
    ensure!(
        domain.len() <= 253,
        "Validation error: domain cannot exceed 253 characters"
    );

    for label in domain.split('.') {
        ensure!(
            !label.is_empty(),
            "Validation error: domain label cannot be empty"
        );
        ensure!(
            label.len() <= 63,
            "Validation error: domain label cannot exceed 63 characters: '{}'",
            label
        );

        let first = label.chars().next().unwrap_or('\0');
        let last = label.chars().last().unwrap_or('\0');

        ensure!(
            first.is_ascii_alphanumeric(),
            "Validation error: domain label must start with alphanumeric: '{}'",
            label
        );
        ensure!(
            last.is_ascii_alphanumeric(),
            "Validation error: domain label must end with alphanumeric: '{}'",
            label
        );

        for c in label.chars() {
            if !c.is_ascii_alphanumeric() && c != '-' {
                bail!(
                    "Validation error: invalid character '{}' in domain label '{}'",
                    c,
                    label
                );
            }
        }
    }

    Ok(domain)
}

/// Validates a network port number (1..=65535).
pub fn validate_port(port: u32) -> Result<u16> {
    if (1..=65535).contains(&port) {
        Ok(port as u16)
    } else {
        bail!(
            "Validation error: port must be between 1 and 65535, got {}",
            port
        );
    }
}

/// Validates a port string slice (e.g. "8080").
pub fn validate_port_str(port_str: &str) -> Result<u16> {
    let port: u32 = port_str
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Validation error: invalid port format '{}'", port_str))?;
    validate_port(port)
}

/// Validates an IPv4 or IPv6 address.
pub fn validate_ip(ip_str: &str) -> Result<IpAddr> {
    ip_str
        .trim()
        .parse::<IpAddr>()
        .map_err(|_| anyhow::anyhow!("Validation error: invalid IP address '{}'", ip_str))
}

/// Validates that a path does not contain directory traversal sequences (`..`).
pub fn validate_safe_path<P: AsRef<Path>>(path: P) -> Result<P> {
    let p = path.as_ref();
    for comp in p.components() {
        if comp == Component::ParentDir {
            bail!(
                "Validation error: path traversal forbidden in '{}'",
                p.display()
            );
        }
    }
    Ok(path)
}
