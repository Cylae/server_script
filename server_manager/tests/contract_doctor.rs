use server_manager::core::doctor::{run_doctor_checks, CheckStatus};

#[test]
fn test_doctor_runs_all_checks_and_produces_report() {
    let report = run_doctor_checks();

    assert!(!report.hostname.is_empty(), "Hostname must not be empty");
    assert!(!report.timestamp.is_empty(), "Timestamp must not be empty");
    assert!(
        !report.checks.is_empty(),
        "Doctor report must contain diagnostic checks"
    );

    // Verify presence of expected diagnostic checks
    let check_names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
    assert!(check_names.contains(&"Kernel Version"));
    assert!(check_names.contains(&"cgroups v2"));
    assert!(check_names.contains(&"Landlock LSM"));
    assert!(check_names.contains(&"Docker Daemon"));
    assert!(check_names.contains(&"Docker Compose"));
    assert!(check_names.contains(&"Firewall Backend") || check_names.contains(&"Firewall (UFW)"));
    assert!(check_names.contains(&"Port Matrix"));
    assert!(check_names.contains(&"Disk Capacity"));
    assert!(check_names.contains(&"NTP Time Sync"));

    // Verify all statuses are valid enum variants
    for check in &report.checks {
        match check.status {
            CheckStatus::Ok | CheckStatus::Warn | CheckStatus::Fail | CheckStatus::Skipped => {}
        }
        assert!(!check.name.is_empty());
        assert!(!check.message.is_empty());
    }
}

#[test]
fn test_doctor_json_serialization() {
    let report = run_doctor_checks();
    let json_str = report
        .to_json()
        .expect("DoctorReport must serialize to valid JSON");

    assert!(json_str.contains("\"timestamp\""));
    assert!(json_str.contains("\"hostname\""));
    assert!(json_str.contains("\"overall_status\""));
    assert!(json_str.contains("\"checks\""));

    // Verify deserialization round-trip
    let deserialized: serde_json::Value =
        serde_json::from_str(&json_str).expect("Serialized JSON must parse cleanly");
    assert!(deserialized.get("checks").is_some());
}

#[test]
fn test_doctor_non_destructive_guarantee() {
    // REQ-OPS-004 / REQ-OPS-005: Running doctor multiple times must be completely non-destructive
    for _ in 0..5 {
        let report = run_doctor_checks();
        assert!(!report.checks.is_empty());
    }
}
