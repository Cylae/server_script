import re

file_path = 'server_manager/src/core/updater.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = re.sub(r'#\[allow\(clippy::unwrap_used, clippy::expect_used, clippy::panic\)\]\n', '', content)
content = re.sub(r'fn (test_\w+)\(\) \{', r'fn \1() -> anyhow::Result<()> {', content)
content = re.sub(r'\.expect\("Checked error condition in code"\)', r'?', content)

content = content.replace(
    'assert!(is_newer_version("v1.2.0", "v1.1.0"));\n    }',
    'assert!(is_newer_version("v1.2.0", "v1.1.0"));\n        Ok(())\n    }'
)

content = content.replace(
    'assert_eq!(info.current_version, CURRENT_VERSION);\n    }',
    'assert_eq!(info.current_version, CURRENT_VERSION);\n        Ok(())\n    }'
)

with open(file_path, 'w') as f:
    f.write(content)
