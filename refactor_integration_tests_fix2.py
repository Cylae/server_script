import re

file_path = 'server_manager/tests/integration_tests.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace(
    '.get("syncthing")\n        ?;',
    '.get("syncthing")\n        .ok_or_else(|| anyhow::anyhow!("Expected syncthing"))?;'
)

content = content.replace(
    '.get("mailserver")\n        ?;',
    '.get("mailserver")\n        .ok_or_else(|| anyhow::anyhow!("Expected mailserver"))?;'
)


with open(file_path, 'w') as f:
    f.write(content)
