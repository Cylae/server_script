#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use server_manager::core::updater::{check_for_updates, CURRENT_VERSION};
use server_manager::interface::web::TelemetryData;

#[test]
fn test_updater_check_for_updates_returns_valid_info() {
    let info = check_for_updates().expect("check_for_updates should succeed");
    assert_eq!(info.current_version, CURRENT_VERSION);
    assert!(!info.current_version.is_empty());
}

#[test]
fn test_telemetry_json_serialization() {
    let telemetry = TelemetryData {
        ram_used: 1024,
        ram_total: 8192,
        swap_used: 0,
        swap_total: 2048,
        cpu_usage: 12.5,
        disk_total: 100,
        disk_used: 25,
        version: CURRENT_VERSION.to_string(),
        update_available: false,
        latest_version: CURRENT_VERSION.to_string(),
    };

    let json_str =
        serde_json::to_string(&telemetry).expect("TelemetryData should serialize to JSON");
    assert!(json_str.contains("cpu_usage"));
    assert!(json_str.contains("ram_used"));
    assert!(json_str.contains("version"));

    let deserialized: TelemetryData =
        serde_json::from_str(&json_str).expect("TelemetryData should deserialize from JSON");
    assert_eq!(deserialized.ram_used, 1024);
    assert_eq!(deserialized.version, CURRENT_VERSION);
}
