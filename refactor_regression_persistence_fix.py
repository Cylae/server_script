import re

file_path = 'server_manager/tests/regression_persistence.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace(
    'fs::metadata(&path)?.permissions().mode() & 0o777,\n        0o600\n    );\n}',
    'fs::metadata(&path)?.permissions().mode() & 0o777,\n        0o600\n    );\n    Ok(())\n}'
)

content = content.replace(
    '        0o644\n    );\n    Ok(())\n}',
    '        0o644\n    );\n}'
)

content = content.replace('cfg.disabled_services.insert(name.into())\n                })\n                ?;', 'cfg.disabled_services.insert(name.into())\n                })\n                .expect("Failed to update config");')

content = content.replace('thread.join()?;', 'thread.join().unwrap_or_else(|_| panic!("Thread panicked"));')

with open(file_path, 'w') as f:
    f.write(content)
