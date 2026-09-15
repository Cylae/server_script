use server_manager::services;

#[test]
fn test_pinned_images() {
    let services = services::get_all_services();
    assert!(!services.is_empty(), "Service catalog must not be empty");

    for svc in services {
        let img = svc.image();
        assert!(
            !img.is_empty(),
            "Service '{}' must declare a non-empty image",
            svc.name()
        );
        assert!(
            img.contains(':'),
            "Service '{}' image '{}' must contain an explicit tag separator ':'",
            svc.name(),
            img
        );
        assert!(
            !img.ends_with(":latest"),
            "Service '{}' image '{}' must NOT use unpinned ':latest' tag (REQ-SEC-011 violation)",
            svc.name(),
            img
        );
        assert!(
            !img.ends_with(':'),
            "Service '{}' image '{}' must specify a valid tag after ':'",
            svc.name(),
            img
        );

        let tag = img.rsplit_once(':').map(|(_, t)| t).unwrap_or("");
        assert!(
            tag != "latest" && !tag.is_empty(),
            "Service '{}' has invalid or unpinned tag '{}'",
            svc.name(),
            tag
        );
    }
}
