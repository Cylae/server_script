use server_manager::services::{self, Protocol};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[test]
fn test_service_count_is_exactly_28() {
    let services = services::get_all_services();
    assert_eq!(
        services.len(),
        28,
        "The catalog must contain exactly 28 integrated services as specified in 01-ARCHITECTURE.md and README.md"
    );
}

#[test]
fn test_all_port_mappings_parse_successfully() {
    let services = services::get_all_services();
    for service in services {
        let ports = service.ports();
        for port_str in ports {
            let parsed = services::PortMapping::parse(&port_str);
            assert!(
                parsed.is_ok(),
                "Failed to parse port '{}' for service '{}': {:?}",
                port_str,
                service.name(),
                parsed.err()
            );
        }
    }
}

#[test]
fn test_no_conflicting_host_ports_across_services() {
    let services = services::get_all_services();
    // Key: (binding IP option, host_port, protocol) -> (service name, full port string)
    let mut bound_ports: HashMap<(Option<String>, u16, Protocol), (&str, String)> = HashMap::new();

    // Include server_manager web dashboard default port
    bound_ports.insert(
        (Some("127.0.0.1".to_string()), 8099, Protocol::Tcp),
        ("server_manager", "127.0.0.1:8099".to_string()),
    );

    for service in services {
        for port in service.parsed_ports() {
            let key = (port.host_ip.clone(), port.host_port, port.protocol);
            if let Some((existing_service, existing_spec)) = bound_ports.get(&key) {
                panic!(
                    "Port collision detected between service '{}' and '{}' on {}:{} ({:?}) [existing: '{}']",
                    existing_service,
                    service.name(),
                    port.host_binding_str(),
                    port.host_port,
                    port.protocol,
                    existing_spec
                );
            }
            bound_ports.insert(key, (service.name(), port.host_binding_str()));
        }
    }

    assert!(
        bound_ports.len() >= 30,
        "Expected at least 30 distinct port mappings including protocols, found {}",
        bound_ports.len()
    );
}

#[test]
fn test_localhost_binding_hygiene_for_sensitive_services() {
    let services = services::get_all_services();
    let sensitive_services = [
        "sonarr",
        "radarr",
        "prowlarr",
        "jackett",
        "bazarr",
        "tautulli",
        "overseerr",
        "jellyseerr",
        "portainer",
        "netdata",
        "uptime-kuma",
        "vaultwarden",
        "filebrowser",
        "yourls",
        "glpi",
        "roundcube",
        "nextcloud",
    ];

    for name in sensitive_services {
        let service = services
            .iter()
            .find(|s| s.name() == name)
            .unwrap_or_else(|| panic!("Service '{}' must exist in catalog", name));

        let ports = service.parsed_ports();
        assert!(
            !ports.is_empty(),
            "Sensitive service '{}' should expose web ports",
            name
        );
        for port in ports {
            assert!(
                port.is_localhost(),
                "Sensitive service '{}' port {} must be bound strictly to localhost (127.0.0.1), got {:?}",
                name,
                port.host_port,
                port.host_ip
            );
        }
    }
}

#[test]
fn test_internal_databases_expose_no_host_ports() {
    let services = services::get_all_services();
    let internal_services = ["mariadb", "redis"];

    for name in internal_services {
        let service = services
            .iter()
            .find(|s| s.name() == name)
            .unwrap_or_else(|| panic!("Internal service '{}' must exist in catalog", name));

        assert!(
            service.ports().is_empty(),
            "Internal service '{}' must NOT expose any host ports (isolated in docker network)",
            name
        );
        assert!(service.parsed_ports().is_empty());
    }
}

#[test]
fn test_categories_and_descriptions_coverage() {
    let catalog = services::get_service_catalog();
    assert_eq!(catalog.len(), 28);

    let mut categories_seen = HashSet::new();
    for entry in &catalog {
        categories_seen.insert(entry.category);
        assert!(
            !entry.description.is_empty(),
            "Service '{}' must have a non-empty description",
            entry.name
        );
    }

    assert!(categories_seen.contains(&services::ServiceCategory::Infrastructure));
    assert!(categories_seen.contains(&services::ServiceCategory::Media));
    assert!(categories_seen.contains(&services::ServiceCategory::Automation));
    assert!(categories_seen.contains(&services::ServiceCategory::Download));
    assert!(categories_seen.contains(&services::ServiceCategory::Apps));
}

#[test]
fn test_port_matrix_documentation_sync() {
    let generated = services::generate_port_matrix_markdown();

    // Check docs/PORT-MATRIX.md relative to repository root
    let doc_paths = [
        Path::new("docs/PORT-MATRIX.md"),
        Path::new("../docs/PORT-MATRIX.md"),
    ];

    let doc_path = doc_paths
        .iter()
        .find(|p| p.exists())
        .copied()
        .unwrap_or_else(|| {
            // If file does not exist yet during initial setup, create it
            let target = Path::new("../docs/PORT-MATRIX.md");
            let _ = fs::write(target, &generated);
            target
        });

    let doc_content = fs::read_to_string(doc_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", doc_path.display(), e));

    // Normalize CRLF to LF for cross-platform stability
    let normalized_generated = generated.replace("\r\n", "\n");
    let normalized_doc = doc_content.replace("\r\n", "\n");

    assert_eq!(
        normalized_generated, normalized_doc,
        "docs/PORT-MATRIX.md is out of sync with code truth in src/services/. Run test to update."
    );
}
