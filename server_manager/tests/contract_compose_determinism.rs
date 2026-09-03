use server_manager::core::config::Config;
use server_manager::core::hardware::{HardwareInfo, HardwareProfile};
use server_manager::core::secrets::Secrets;
use server_manager::generate_compose_yaml;
use std::fs;
use std::path::Path;
use std::process::Command;

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

fn high_hardware() -> HardwareInfo {
    HardwareInfo {
        profile: HardwareProfile::High,
        ram_gb: 32,
        cpu_cores: 16,
        has_nvidia: true,
        has_intel_quicksync: false,
        disk_gb: 2000,
        swap_gb: 8,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    }
}

fn low_hardware() -> HardwareInfo {
    HardwareInfo {
        profile: HardwareProfile::Low,
        ram_gb: 2,
        cpu_cores: 2,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 100,
        swap_gb: 1,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    }
}

fn test_secrets() -> Secrets {
    Secrets {
        mysql_root_password: Some("deterministic_root_secret".to_string()),
        mysql_user_password: Some("deterministic_user_secret".to_string()),
        nextcloud_db_password: Some("deterministic_nc_db".to_string()),
        glpi_db_password: Some("deterministic_glpi_db".to_string()),
        gitea_db_password: Some("deterministic_gitea_db".to_string()),
        yourls_admin_password: Some("deterministic_yourls_admin".to_string()),
        mailserver_password: Some("deterministic_mail_secret".to_string()),
        nextcloud_admin_password: Some("deterministic_nc_admin".to_string()),
        roundcube_db_password: Some("deterministic_rc_db".to_string()),
        vaultwarden_admin_token: Some("deterministic_vw_token".to_string()),
        server_manager_admin_password: Some("deterministic_sm_pass".to_string()),
    }
}

#[test]
fn test_compose_generation_is_byte_stable_run_twice() {
    let hw = standard_hardware();
    let secrets = test_secrets();
    let config = Config::default();

    // Run #1
    let run1 = generate_compose_yaml(&hw, &secrets, &config)
        .expect("First compose generation must succeed");

    // Run #2
    let run2 = generate_compose_yaml(&hw, &secrets, &config)
        .expect("Second compose generation must succeed");

    // Byte-by-byte equality assertion
    assert_eq!(
        run1.as_bytes(),
        run2.as_bytes(),
        "docker-compose.yml output is not byte-stable between two runs"
    );
    assert_eq!(run1, run2);
}

#[test]
fn test_compose_file_cmp_command() {
    let hw = standard_hardware();
    let secrets = test_secrets();
    let config = Config::default();

    let run1 = generate_compose_yaml(&hw, &secrets, &config)
        .expect("First compose generation must succeed");
    let run2 = generate_compose_yaml(&hw, &secrets, &config)
        .expect("Second compose generation must succeed");

    let tmp_dir = std::env::temp_dir().join(format!("cmp_test_{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).expect("Failed to create temp directory");

    let file1_path = tmp_dir.join("compose_run1.yml");
    let file2_path = tmp_dir.join("compose_run2.yml");

    fs::write(&file1_path, &run1).expect("Failed to write file1");
    fs::write(&file2_path, &run2).expect("Failed to write file2");

    // Execute OS 'cmp' command as required by AGENTS.md Rule 7
    let cmp_output = Command::new("cmp")
        .arg(&file1_path)
        .arg(&file2_path)
        .output();

    // Cleanup temp files
    let _ = fs::remove_file(&file1_path);
    let _ = fs::remove_file(&file2_path);
    let _ = fs::remove_dir(&tmp_dir);

    match cmp_output {
        Ok(output) => {
            assert!(
                output.status.success(),
                "cmp exited with non-zero status: stderr='{}', stdout='{}'",
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
        }
        Err(e) => {
            // On Windows non-WSL environments where cmp is absent, fallback to byte check
            eprintln!("'cmp' command not found ({}), byte comparison was verified in test_compose_generation_is_byte_stable_run_twice", e);
        }
    }
}

#[test]
fn test_services_and_keys_are_alphabetically_sorted() {
    let hw = standard_hardware();
    let secrets = test_secrets();
    let config = Config::default();

    let compose = server_manager::build_compose_structure(&hw, &secrets, &config)
        .expect("build_compose_structure must succeed");

    // 1. Verify services order in BTreeMap keys
    let service_keys: Vec<&String> = compose.services.keys().collect();
    for window in service_keys.windows(2) {
        assert!(
            window[0] <= window[1],
            "Services not sorted: '{}' comes before '{}'",
            window[0],
            window[1]
        );
    }

    // 2. Verify environment variables are sorted within services
    for (service_name, service) in &compose.services {
        if let Some(envs) = &service.environment {
            for window in envs.windows(2) {
                assert!(
                    window[0] <= window[1],
                    "Environment variables for service '{}' not sorted: '{}' comes before '{}'",
                    service_name,
                    window[0],
                    window[1]
                );
            }
        }

        // 3. Verify labels are sorted
        if let Some(labels) = &service.labels {
            for window in labels.windows(2) {
                assert!(
                    window[0] <= window[1],
                    "Labels for service '{}' not sorted: '{}' comes before '{}'",
                    service_name,
                    window[0],
                    window[1]
                );
            }
        }
    }
}

#[test]
fn test_compose_golden_files_match() {
    let profiles = [
        ("standard", standard_hardware()),
        ("high", high_hardware()),
        ("low", low_hardware()),
    ];

    let secrets = test_secrets();
    let config = Config::default();

    for (name, hw) in profiles {
        let generated = generate_compose_yaml(&hw, &secrets, &config)
            .unwrap_or_else(|e| panic!("Failed to generate compose for profile '{}': {}", name, e));

        let golden_path =
            Path::new("tests/golden").join(format!("docker-compose.{}.golden.yml", name));

        // Create parent if missing
        if let Some(parent) = golden_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if !golden_path.exists() {
            fs::write(&golden_path, &generated).unwrap_or_else(|e| {
                panic!(
                    "Failed to write initial golden file {}: {}",
                    golden_path.display(),
                    e
                )
            });
        }

        let golden_content = fs::read_to_string(&golden_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read golden file {}: {}",
                golden_path.display(),
                e
            )
        });

        let norm_gen = generated.replace("\r\n", "\n");
        let norm_golden = golden_content.replace("\r\n", "\n");

        assert_eq!(
            norm_gen,
            norm_golden,
            "Compose output for profile '{}' diverged from golden file at {}",
            name,
            golden_path.display()
        );
    }
}
