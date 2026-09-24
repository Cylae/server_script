import re

file_path = 'server_manager/src/core/secrets.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = re.sub(r'#\[allow\(clippy::unwrap_used, clippy::expect_used, clippy::panic\)\]\n', '', content)
content = re.sub(r'fn (test_\w+)\(\) \{', r'fn \1() -> anyhow::Result<()> {', content)
content = re.sub(r'\.expect\("([^"]+)"\)', r'?', content)

content = content.replace(
    'assert_eq!(hex.len(), 32);\n    }',
    'assert_eq!(hex.len(), 32);\n        Ok(())\n    }'
)

content = content.replace(
    'assert!(!secrets.mysql_root_password.is_empty());\n    }',
    'assert!(!secrets.mysql_root_password.is_empty());\n        Ok(())\n    }'
)

with open(file_path, 'w') as f:
    f.write(content)
