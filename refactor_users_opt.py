import re

file_path = 'server_manager/src/core/users.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace('assert!(!verify_password("WrongPassword!", &hash));\n    }', 'assert!(!verify_password("WrongPassword!", &hash));\n        Ok(())\n    }')

with open(file_path, 'w') as f:
    f.write(content)
