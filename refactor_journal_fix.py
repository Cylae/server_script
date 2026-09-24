import re

file_path = 'server_manager/src/core/journal.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace(
    'assert!(def_path.ends_with("journal.jsonl"));\n    }',
    'assert!(def_path.ends_with("journal.jsonl"));\n        Ok(())\n    }'
)

with open(file_path, 'w') as f:
    f.write(content)
