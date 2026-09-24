import re

file_path = 'server_manager/src/core/users.rs'
with open(file_path, 'r') as f:
    content = f.read()

# For .ok_or_else() on Result types, simply use ? (since they already return a Result)
content = re.sub(r'\.ok_or_else\(\|\| anyhow::anyhow!\("[^"]+"\)\)\?', r'?', content)

content = content.replace(
    'assert!(verify_password("SuperSecret123!", &hash));\n    }',
    'assert!(verify_password("SuperSecret123!", &hash));\n        Ok(())\n    }'
)

with open(file_path, 'w') as f:
    f.write(content)
