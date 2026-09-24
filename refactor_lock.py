import re

file_path = 'server_manager/src/core/lock.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace(r'let err_msg = lock2_res.err().unwrap().to_string();', r'let err_msg = lock2_res.err().ok_or_else(|| anyhow::anyhow!("Expected error"))?.to_string();')

with open(file_path, 'w') as f:
    f.write(content)
