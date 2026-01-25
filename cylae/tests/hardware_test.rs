use cylae::core::hardware::{HardwareManager, HardwareProfile};

#[test]
fn test_evaluate_profile() {
    // High Spec: 8GB RAM, 4 Cores, 2GB Swap
    assert_eq!(HardwareManager::evaluate_profile(8.0, 4, 2.0), HardwareProfile::High);

    // Low RAM: 3.5GB
    assert_eq!(HardwareManager::evaluate_profile(3.5, 4, 2.0), HardwareProfile::Low);

    // Low CPU: 2 Cores
    assert_eq!(HardwareManager::evaluate_profile(8.0, 2, 2.0), HardwareProfile::Low);

    // Low Swap: 0.5GB
    assert_eq!(HardwareManager::evaluate_profile(8.0, 4, 0.5), HardwareProfile::Low);
}
