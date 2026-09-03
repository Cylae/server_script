use server_manager::core::ops::{
    DockerOps, FirewallBackend, MockDockerOps, MockFirewallBackend, MockSystemOps, SystemOps,
};
use std::path::Path;

#[tokio::test]
async fn test_mock_system_ops() {
    let ops = MockSystemOps::default();
    assert!(!ops.is_root());

    assert!(ops.install_dependencies().is_ok());
    assert!(ops.create_system_user("alice", "secret123").is_ok());
    assert!(ops.set_system_quota("alice", 50).is_ok());
    assert!(ops.delete_system_user("alice").is_ok());

    let calls = ops.calls.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0], "install_dependencies");
    assert_eq!(calls[1], "create_system_user:alice");
    assert_eq!(calls[2], "set_quota:alice:50");
    assert_eq!(calls[3], "delete_system_user:alice");
}

#[tokio::test]
async fn test_mock_docker_ops() {
    let ops = MockDockerOps::default();
    assert!(!ops.is_installed());

    assert!(ops.install().is_ok());
    assert!(ops.is_installed());

    let compose_file = Path::new("docker-compose.yml");
    assert!(ops.compose_up(compose_file).is_ok());
    assert!(ops.compose_pull(compose_file).is_ok());
    assert!(ops.compose_down(compose_file).is_ok());
    assert!(ops.prune_system().is_ok());

    let calls = ops.calls.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(calls.len(), 5);
    assert_eq!(calls[0], "install");
    assert_eq!(calls[1], "compose_up:docker-compose.yml");
    assert_eq!(calls[2], "compose_pull:docker-compose.yml");
    assert_eq!(calls[3], "compose_down:docker-compose.yml");
    assert_eq!(calls[4], "prune_system");
}

#[tokio::test]
async fn test_mock_firewall_backend() {
    let fw = MockFirewallBackend::default();
    assert!(!fw.is_active().unwrap_or(true));

    assert!(fw.allow_port(80, "tcp").is_ok());
    assert!(fw.allow_port(443, "tcp").is_ok());

    {
        let ports = fw.allowed_ports.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(*ports, vec![80, 443]);
    }

    assert!(fw.deny_port(80, "tcp").is_ok());

    {
        let ports = fw.allowed_ports.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(*ports, vec![443]);
    }

    assert!(fw.configure_defaults().is_ok());

    {
        let ports = fw.allowed_ports.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(*ports, vec![443, 22, 80, 443, 8099]);
    }
}
