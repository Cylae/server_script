import re

file_path = 'server_manager/tests/integration_tests.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace(
    '        .ok_or_else(|| anyhow::anyhow!("Value missing"))?\n        ?;',
    '        .ok_or_else(|| anyhow::anyhow!("Value missing"))?;'
)

with open(file_path, 'w') as f:
    f.write(content)
