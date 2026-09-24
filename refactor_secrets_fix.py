import re

file_path = 'server_manager/src/core/secrets.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace(
    'assert_eq!(hex.len(), 32); // 16 bytes = 32 hex chars\n    }',
    'assert_eq!(hex.len(), 32); // 16 bytes = 32 hex chars\n        Ok(())\n    }'
)

content = content.replace(
    'assert!(secrets.server_manager_admin_password.is_none());\n    }',
    'assert!(secrets.server_manager_admin_password.is_none());\n        Ok(())\n    }'
)

with open(file_path, 'w') as f:
    f.write(content)
