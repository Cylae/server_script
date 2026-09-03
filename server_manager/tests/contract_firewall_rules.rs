use server_manager::core::firewall::configure_with_backend;
use server_manager::core::ops::MockFirewallBackend;

#[test]
fn test_firewall_rule_configuration_with_mock_backend() {
    let mock = MockFirewallBackend::default();
    let result = configure_with_backend(&mock);
    assert!(result.is_ok(), "Firewall configuration should succeed");

    let ports = mock.allowed_ports.lock().unwrap();

    // Verify critical port openings (including those added by configure_defaults)
    assert!(ports.contains(&22));
    assert!(ports.contains(&8099));
    assert!(ports.contains(&80));
    assert!(ports.contains(&443));
    assert!(ports.contains(&32400));
    assert!(ports.contains(&8096));
    assert!(ports.contains(&51820));
}
