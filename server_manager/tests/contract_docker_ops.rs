use server_manager::core::docker::install_with_ops;
use server_manager::core::ops::MockDockerOps;
use std::sync::atomic::AtomicBool;

#[test]
fn test_docker_install_when_already_installed() {
    let mock = MockDockerOps {
        installed: AtomicBool::new(true),
        calls: Default::default(),
    };
    let result = install_with_ops(&mock);
    assert!(result.is_ok());
    let calls = mock.calls.lock().unwrap();
    assert!(calls.is_empty());
}

#[test]
fn test_docker_install_when_missing() {
    let mock = MockDockerOps {
        installed: AtomicBool::new(false),
        calls: Default::default(),
    };
    let result = install_with_ops(&mock);
    assert!(result.is_ok());
    let calls = mock.calls.lock().unwrap();
    assert!(calls.contains(&"install".to_string()));
}
