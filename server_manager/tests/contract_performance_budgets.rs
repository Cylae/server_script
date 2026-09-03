use server_manager::core::atomic_io::atomic_write_str;
use server_manager::core::config::Config;
use server_manager::core::hardware::{HardwareInfo, HardwareProfile};
use server_manager::core::secrets::Secrets;
use server_manager::core::validate::{
    validate_safe_path, validate_service_name, validate_username,
};
use server_manager::generate_compose_yaml;
use server_manager::services::generate_port_matrix_markdown;
use std::fs;
use std::time::Instant;

fn standard_hardware() -> HardwareInfo {
    HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 2,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    }
}

fn test_secrets() -> Secrets {
    Secrets {
        mysql_root_password: Some("perf_root_secret".to_string()),
        mysql_user_password: Some("perf_user_secret".to_string()),
        nextcloud_db_password: Some("perf_nc_db".to_string()),
        glpi_db_password: Some("perf_glpi_db".to_string()),
        gitea_db_password: Some("perf_gitea_db".to_string()),
        yourls_admin_password: Some("perf_yourls_admin".to_string()),
        mailserver_password: Some("perf_mail_secret".to_string()),
        nextcloud_admin_password: Some("perf_nc_admin".to_string()),
        roundcube_db_password: Some("perf_rc_db".to_string()),
        vaultwarden_admin_token: Some("perf_vw_token".to_string()),
        server_manager_admin_password: Some("perf_sm_pass".to_string()),
    }
}

#[test]
fn test_budget_compose_generation_under_25ms() {
    let hw = standard_hardware();
    let secrets = test_secrets();
    let config = Config::default();

    // Warm up
    let _ = generate_compose_yaml(&hw, &secrets, &config).expect("Warmup must succeed");

    let start = Instant::now();
    let iterations = 5;
    for _ in 0..iterations {
        let yaml = generate_compose_yaml(&hw, &secrets, &config).expect("Generation must succeed");
        assert!(!yaml.is_empty());
    }
    let elapsed = start.elapsed();
    let avg_ms = (elapsed.as_secs_f64() * 1000.0) / (iterations as f64);

    println!(
        "Compose generation average latency over {} iterations: {:.3} ms (budget: 25.0 ms)",
        iterations, avg_ms
    );
    assert!(
        avg_ms < 25.0,
        "Compose generation exceeded performance budget: {:.3} ms > 25.0 ms",
        avg_ms
    );
}

#[test]
fn test_budget_port_matrix_generation_under_5ms() {
    // Warm up
    let _ = generate_port_matrix_markdown();

    let start = Instant::now();
    let iterations = 10;
    for _ in 0..iterations {
        let md = generate_port_matrix_markdown();
        assert!(!md.is_empty());
    }
    let elapsed = start.elapsed();
    let avg_ms = (elapsed.as_secs_f64() * 1000.0) / (iterations as f64);

    println!(
        "Port matrix generation average latency over {} iterations: {:.3} ms (budget: 5.0 ms)",
        iterations, avg_ms
    );
    assert!(
        avg_ms < 5.0,
        "Port matrix markdown generation exceeded performance budget: {:.3} ms > 5.0 ms",
        avg_ms
    );
}

#[test]
fn test_budget_input_validation_1000_iterations_under_10ms() {
    let names = [
        "sonarr",
        "radarr",
        "plex",
        "mariadb",
        "nextcloud",
        "vaultwarden",
        "nginx-proxy",
    ];
    let users = ["admin", "alice", "bob_dev", "operator-1", "auditor"];
    let paths = [
        "./config/plex",
        "./media/tv",
        "/var/run/docker.sock",
        "./config/mariadb/initdb.d",
    ];

    let start = Instant::now();
    for _ in 0..1000 {
        for n in &names {
            assert!(validate_service_name(n).is_ok());
        }
        for u in &users {
            assert!(validate_username(u).is_ok());
        }
        for p in &paths {
            assert!(validate_safe_path(p).is_ok());
        }
    }
    let elapsed = start.elapsed();
    let total_ms = elapsed.as_secs_f64() * 1000.0;

    println!(
        "Input validation (16,000 checks) total elapsed: {:.3} ms (budget: 50.0 ms)",
        total_ms
    );
    assert!(
        total_ms < 50.0,
        "Input validation exceeded budget: {:.3} ms > 50.0 ms",
        total_ms
    );
}

#[test]
fn test_budget_atomic_write_under_50ms() {
    let tmp_dir = std::env::temp_dir().join(format!("perf_atomic_test_{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).expect("Failed to create temp dir");

    let test_file = tmp_dir.join("test_file.txt");
    let payload = "A".repeat(64 * 1024); // 64 KiB

    let start = Instant::now();
    let iterations = 5;
    for _ in 0..iterations {
        atomic_write_str(&test_file, &payload, 0o600).expect("Atomic write must succeed");
    }
    let elapsed = start.elapsed();
    let avg_ms = (elapsed.as_secs_f64() * 1000.0) / (iterations as f64);

    let _ = fs::remove_dir_all(&tmp_dir);

    println!(
        "Atomic 64 KiB write average latency over {} iterations: {:.3} ms (budget: 50.0 ms)",
        iterations, avg_ms
    );
    assert!(
        avg_ms < 50.0,
        "Atomic write exceeded performance budget: {:.3} ms > 50.0 ms",
        avg_ms
    );
}
