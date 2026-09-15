use std::fs;
use std::path::Path;

fn check_forbidden_patterns(dir: &Path) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).expect("Failed to read dir") {
        let entry = entry.expect("DirEntry failure");
        let path = entry.path();
        if path.is_dir() {
            check_forbidden_patterns(&path);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path).expect("Read file failed");
            for (line_idx, line) in content.lines().enumerate() {
                // Ignore comments
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }
                assert!(
                    !line.contains("std::process::Command::new")
                        && !line.contains("tokio::process::Command::new")
                        && !line.contains("Command::new("),
                    "Architectural Trust-Boundary Violation: Process execution detected at {}:{}: '{}'. \
                     All privileged OS mutations must be mediated through src/core abstractions.",
                    path.display(),
                    line_idx + 1,
                    line
                );
            }
        }
    }
}

#[test]
fn test_no_process_execution_in_interface_or_services() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let interface_dir = manifest_dir.join("src").join("interface");
    let services_dir = manifest_dir.join("src").join("services");

    check_forbidden_patterns(&interface_dir);
    check_forbidden_patterns(&services_dir);
}
