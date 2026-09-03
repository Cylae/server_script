//! Diagnostic Subcommand (`server_manager doctor`)
//! Authoritative reference: REQ-OPS-005.
//!
//! Provides preventative, non-destructive health and readiness inspection for the host system,
//! container runtime, kernel capabilities, port availability, disk capacity, and firewall.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
    Skipped,
}

impl CheckStatus {
    pub fn badge(&self) -> &'static str {
        match self {
            CheckStatus::Ok => "[  OK  ]",
            CheckStatus::Warn => "[ WARN ]",
            CheckStatus::Fail => "[ FAIL ]",
            CheckStatus::Skipped => "[ SKIP ]",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub timestamp: String,
    pub hostname: String,
    pub overall_status: CheckStatus,
    pub checks: Vec<DoctorCheckResult>,
}

impl DoctorReport {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn print_console(&self) {
        println!("\n======================================================================");
        println!("  SERVER_MANAGER DOCTOR DIAGNOSTIC REPORT (REQ-OPS-005)");
        println!("======================================================================");
        println!("Timestamp : {}", self.timestamp);
        println!("Hostname  : {}", self.hostname);
        println!(
            "Status    : {} {:?}",
            self.overall_status.badge(),
            self.overall_status
        );
        println!("----------------------------------------------------------------------");

        for check in &self.checks {
            println!(
                "{} {:<24} : {}",
                check.status.badge(),
                check.name,
                check.message
            );
            if let Some(details) = &check.details {
                println!("       Details: {}", details);
            }
        }
        println!("======================================================================\n");
    }
}

pub fn run_doctor_checks() -> DoctorReport {
    let checks = vec![
        check_kernel_version(),
        check_cgroups_v2(),
        check_landlock(),
        check_docker_daemon(),
        check_compose_tool(),
        check_firewall(),
        check_port_conflicts(),
        check_disk_space(),
        check_ntp_sync(),
    ];

    let mut overall = CheckStatus::Ok;
    for c in &checks {
        match c.status {
            CheckStatus::Fail => {
                overall = CheckStatus::Fail;
                break;
            }
            CheckStatus::Warn if overall != CheckStatus::Fail => {
                overall = CheckStatus::Warn;
            }
            _ => {}
        }
    }

    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "localhost".to_string());

    let timestamp = format!("{:?}", std::time::SystemTime::now());

    DoctorReport {
        timestamp,
        hostname,
        overall_status: overall,
        checks,
    }
}

pub fn check_kernel_version() -> DoctorCheckResult {
    let path = Path::new("/proc/sys/kernel/osrelease");
    if path.exists() {
        if let Ok(release) = fs::read_to_string(path) {
            let release = release.trim().to_string();
            return DoctorCheckResult {
                name: "Kernel Version".to_string(),
                status: CheckStatus::Ok,
                message: format!("Linux Kernel {}", release),
                details: Some("Host Linux OS detected via /proc/sys/kernel/osrelease".to_string()),
            };
        }
    }

    DoctorCheckResult {
        name: "Kernel Version".to_string(),
        status: CheckStatus::Skipped,
        message: "Non-Linux environment or /proc inaccessible".to_string(),
        details: None,
    }
}

pub fn check_cgroups_v2() -> DoctorCheckResult {
    let controllers_path = Path::new("/sys/fs/cgroup/cgroup.controllers");
    if controllers_path.exists() {
        if let Ok(controllers) = fs::read_to_string(controllers_path) {
            return DoctorCheckResult {
                name: "cgroups v2".to_string(),
                status: CheckStatus::Ok,
                message: "cgroups v2 unified hierarchy active".to_string(),
                details: Some(format!("Available controllers: {}", controllers.trim())),
            };
        }
    }

    let cgroup_dir = Path::new("/sys/fs/cgroup");
    if cgroup_dir.exists() {
        DoctorCheckResult {
            name: "cgroups v2".to_string(),
            status: CheckStatus::Warn,
            message: "cgroups v1 legacy hierarchy detected".to_string(),
            details: Some(
                "Upgrading to unified cgroups v2 is recommended for memory QoS".to_string(),
            ),
        }
    } else {
        DoctorCheckResult {
            name: "cgroups v2".to_string(),
            status: CheckStatus::Skipped,
            message: "cgroup filesystem not mounted or non-Linux host".to_string(),
            details: None,
        }
    }
}

pub fn check_landlock() -> DoctorCheckResult {
    let lsm_path = Path::new("/sys/kernel/security/lsm");
    if lsm_path.exists() {
        if let Ok(lsm) = fs::read_to_string(lsm_path) {
            if lsm.contains("landlock") {
                return DoctorCheckResult {
                    name: "Landlock LSM".to_string(),
                    status: CheckStatus::Ok,
                    message: "Landlock Linux Security Module active".to_string(),
                    details: Some(format!("Active LSMs: {}", lsm.trim())),
                };
            }
        }
    }

    DoctorCheckResult {
        name: "Landlock LSM".to_string(),
        status: CheckStatus::Skipped,
        message: "Landlock LSM not detected or securityfs unmounted".to_string(),
        details: Some(
            "Kernel sandboxing fallback to standard POSIX permissions active".to_string(),
        ),
    }
}

pub fn check_docker_daemon() -> DoctorCheckResult {
    let socket_path = Path::new("/var/run/docker.sock");
    if socket_path.exists() {
        return DoctorCheckResult {
            name: "Docker Daemon".to_string(),
            status: CheckStatus::Ok,
            message: "Docker UNIX socket /var/run/docker.sock accessible".to_string(),
            details: None,
        };
    }

    if let Ok(output) = Command::new("docker").arg("--version").output() {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return DoctorCheckResult {
                name: "Docker Daemon".to_string(),
                status: CheckStatus::Ok,
                message: ver,
                details: Some("Docker CLI binary found in PATH".to_string()),
            };
        }
    }

    DoctorCheckResult {
        name: "Docker Daemon".to_string(),
        status: CheckStatus::Warn,
        message: "Docker daemon socket not found; Docker service may not be running".to_string(),
        details: Some(
            "Ensure Docker service is installed and started (systemctl start docker)".to_string(),
        ),
    }
}

pub fn check_compose_tool() -> DoctorCheckResult {
    if let Ok(output) = Command::new("docker")
        .arg("compose")
        .arg("version")
        .output()
    {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return DoctorCheckResult {
                name: "Docker Compose".to_string(),
                status: CheckStatus::Ok,
                message: format!("Plugin: {}", ver),
                details: None,
            };
        }
    }

    if let Ok(output) = Command::new("docker-compose").arg("--version").output() {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return DoctorCheckResult {
                name: "Docker Compose".to_string(),
                status: CheckStatus::Ok,
                message: format!("Standalone: {}", ver),
                details: None,
            };
        }
    }

    DoctorCheckResult {
        name: "Docker Compose".to_string(),
        status: CheckStatus::Warn,
        message: "Neither 'docker compose' nor 'docker-compose' command found".to_string(),
        details: Some("Install docker-compose-plugin or docker-compose".to_string()),
    }
}

pub fn check_firewall() -> DoctorCheckResult {
    if let Ok(output) = Command::new("ufw").arg("status").output() {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let first_line = status.lines().next().unwrap_or("active");
            return DoctorCheckResult {
                name: "Firewall (UFW)".to_string(),
                status: CheckStatus::Ok,
                message: format!("UFW is {}", first_line),
                details: None,
            };
        }
    }

    if which::which("nft").is_ok() || which::which("iptables").is_ok() {
        return DoctorCheckResult {
            name: "Firewall Backend".to_string(),
            status: CheckStatus::Ok,
            message: "Kernel packet filtering backend (nftables/iptables) available".to_string(),
            details: None,
        };
    }

    DoctorCheckResult {
        name: "Firewall Backend".to_string(),
        status: CheckStatus::Skipped,
        message: "No standard firewall utility (ufw/nftables/iptables) found in PATH".to_string(),
        details: None,
    }
}

pub fn check_port_conflicts() -> DoctorCheckResult {
    let catalog = crate::services::get_service_catalog();
    let mut total_ports = 0;
    for entry in &catalog {
        total_ports += entry.ports.len();
    }

    // In a diagnostic check, we verify that the port matrix has zero internal collisions
    let mut seen = std::collections::HashSet::new();
    for entry in &catalog {
        for port in &entry.ports {
            let key = (port.host_ip.clone(), port.host_port, port.protocol);
            if !seen.insert(key) {
                return DoctorCheckResult {
                    name: "Port Matrix".to_string(),
                    status: CheckStatus::Fail,
                    message: format!("Port collision detected on port {}", port.host_port),
                    details: Some(format!("Service: {}", entry.name)),
                };
            }
        }
    }

    DoctorCheckResult {
        name: "Port Matrix".to_string(),
        status: CheckStatus::Ok,
        message: format!(
            "Zero internal conflicts across {} service port definitions",
            total_ports
        ),
        details: None,
    }
}

pub fn check_disk_space() -> DoctorCheckResult {
    if let Ok(output) = Command::new("df").arg("-Pk").arg(".").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(avail_kb) = parts[3].parse::<u64>() {
                        let avail_gb = avail_kb / (1024 * 1024);
                        if avail_gb >= 10 {
                            return DoctorCheckResult {
                                name: "Disk Capacity".to_string(),
                                status: CheckStatus::Ok,
                                message: format!("{} GB available (>= 10 GB required)", avail_gb),
                                details: None,
                            };
                        } else {
                            return DoctorCheckResult {
                                name: "Disk Capacity".to_string(),
                                status: CheckStatus::Warn,
                                message: format!(
                                    "Low disk space: only {} GB available (< 10 GB)",
                                    avail_gb
                                ),
                                details: Some(
                                    "Free up disk space before deploying media containers"
                                        .to_string(),
                                ),
                            };
                        }
                    }
                }
            }
        }
    }

    DoctorCheckResult {
        name: "Disk Capacity".to_string(),
        status: CheckStatus::Skipped,
        message: "df utility not available or disk space query failed".to_string(),
        details: None,
    }
}

pub fn check_ntp_sync() -> DoctorCheckResult {
    if let Ok(output) = Command::new("timedatectl").arg("status").output() {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            let synced =
                status.contains("synchronized: yes") || status.contains("NTP service: active");
            return DoctorCheckResult {
                name: "NTP Time Sync".to_string(),
                status: if synced {
                    CheckStatus::Ok
                } else {
                    CheckStatus::Warn
                },
                message: if synced {
                    "System clock is synchronized via NTP".to_string()
                } else {
                    "System clock may not be NTP-synchronized".to_string()
                },
                details: None,
            };
        }
    }

    let localtime = Path::new("/etc/localtime");
    if localtime.exists() {
        return DoctorCheckResult {
            name: "NTP Time Sync".to_string(),
            status: CheckStatus::Ok,
            message: "Timezone configured via /etc/localtime".to_string(),
            details: None,
        };
    }

    DoctorCheckResult {
        name: "NTP Time Sync".to_string(),
        status: CheckStatus::Skipped,
        message: "timedatectl or /etc/localtime not accessible".to_string(),
        details: None,
    }
}
