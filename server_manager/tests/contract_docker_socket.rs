use server_manager::core::hardware::HardwareInfo;
use server_manager::services;

#[test]
fn test_docker_socket_mounting() {
    let hw = HardwareInfo::detect();
    let services = services::get_all_services();

    let permitted_services = ["portainer", "netdata"];

    for svc in services {
        let vols = svc.volumes(&hw);
        for vol in &vols {
            if vol.contains("docker.sock") {
                assert!(
                    permitted_services.contains(&svc.name()),
                    "Service '{}' mounts docker.sock without explicit authorization (REQ-SEC-010 violation)",
                    svc.name()
                );
                if svc.name() == "netdata" {
                    assert!(
                        vol.ends_with(":ro"),
                        "Service 'netdata' must mount docker.sock read-only (:ro), got '{}' (REQ-SEC-010 violation)",
                        vol
                    );
                }
            }
        }
    }
}
