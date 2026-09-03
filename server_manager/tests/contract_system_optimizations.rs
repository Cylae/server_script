use server_manager::core::hardware::{HardwareInfo, HardwareProfile};
use server_manager::core::system::generate_sysctl_config;

fn mock_hw(ram_gb: u64, cpu_cores: usize, swap_gb: u64) -> HardwareInfo {
    HardwareInfo {
        profile: HardwareInfo::evaluate_profile(ram_gb, cpu_cores, swap_gb),
        ram_gb,
        cpu_cores,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 100,
        swap_gb,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    }
}

#[test]
fn test_sysctl_config_generation_low_ram() {
    let hw = mock_hw(8, 4, 4);
    let cfg = generate_sysctl_config(&hw);

    assert!(cfg.contains("vm.swappiness=10"));
    assert!(cfg.contains("fs.inotify.max_user_watches=524288"));
    assert!(cfg.contains("net.core.default_qdisc=fq"));
    assert!(cfg.contains("net.ipv4.tcp_congestion_control=bbr"));
    assert!(cfg.contains("net.core.somaxconn=4096"));
    assert!(cfg.contains("vm.max_map_count=262144"));
}

#[test]
fn test_sysctl_config_generation_high_ram() {
    let hw = mock_hw(32, 8, 8);
    let cfg = generate_sysctl_config(&hw);

    assert!(cfg.contains("vm.swappiness=1"));
    assert!(cfg.contains("fs.inotify.max_user_watches=524288"));
    assert!(cfg.contains("net.core.default_qdisc=fq"));
}

#[test]
fn test_hardware_profile_boundary_cases() {
    // Exactly 4GB with no swap: downgraded to Low
    assert_eq!(
        HardwareInfo::evaluate_profile(4, 4, 0),
        HardwareProfile::Low
    );

    // Exactly 4GB with 1GB swap: Standard
    assert_eq!(
        HardwareInfo::evaluate_profile(4, 4, 1),
        HardwareProfile::Standard
    );

    // 16GB with 2 cores: Low (core count bottleneck)
    assert_eq!(
        HardwareInfo::evaluate_profile(16, 2, 0),
        HardwareProfile::Low
    );

    // 17GB with 1 core: High (RAM > 16GB takes precedence)
    assert_eq!(
        HardwareInfo::evaluate_profile(17, 1, 0),
        HardwareProfile::High
    );
}
