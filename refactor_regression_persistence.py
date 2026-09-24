import re

file_path = 'server_manager/tests/regression_persistence.rs'
with open(file_path, 'r') as f:
    content = f.read()

# I will replace .unwrap() to return Result inside tests correctly.
# But let's leave the thread closure and drop as expect or match since they cannot easily return Result

content = content.replace('fs::create_dir_all(&path).unwrap();', 'fs::create_dir_all(&path).expect("failed to create dir");')
content = content.replace('.unwrap();', '?;')

content = re.sub(r'#\[allow\(clippy::unwrap_used\)\]\n', '', content)
content = re.sub(r'fn (regression_\w+)\(\) \{', r'fn \1() -> anyhow::Result<()> {', content)

# Assertions
content = content.replace('fs::read(&path).unwrap()', 'fs::read(&path)?')
content = content.replace('fs::read_to_string(victim).unwrap()', 'fs::read_to_string(victim)?')
content = content.replace('fs::read_to_string(&secret_path).unwrap()', 'fs::read_to_string(&secret_path)?')
content = content.replace('Config::load_from(&first).unwrap()', 'Config::load_from(&first)?')
content = content.replace('Config::load_from(&second).unwrap()', 'Config::load_from(&second)?')
content = content.replace('fs::metadata(&path).unwrap()', 'fs::metadata(&path)?')
content = content.replace('fs::metadata(&path)?.modified().unwrap()', 'fs::metadata(&path)?.modified()?')
content = content.replace('Config::load_from(&path).unwrap()', 'Config::load_from(&path)?')

# fs::read_dir
content = content.replace('fs::read_dir(&dir.0).unwrap()', 'fs::read_dir(&dir.0)?')
content = content.replace('.any(|entry| entry\n        .unwrap()', '.any(|entry| entry\n        .expect("read_dir failure")')

# thread.join
content = content.replace('thread.join().unwrap();', 'thread.join().expect("thread panicked");')

# The loop in thread spawn
content = content.replace('cfg.disabled_services.insert(name.into())\n            })\n            .unwrap();', 'cfg.disabled_services.insert(name.into())\n            })\n            .expect("update failed");')

# Append Ok(()) to tests
content = content.replace('    assert_eq!(fs::read(&path)?, b"second");\n    assert!(!fs::read_dir(&dir.0)?.any(|entry| entry\n        .expect("read_dir failure")\n        .file_name()\n        .to_string_lossy()\n        .starts_with(".tmp.")));\n}', '    assert_eq!(fs::read(&path)?, b"second");\n    assert!(!fs::read_dir(&dir.0)?.any(|entry| entry\n        .expect("read_dir failure")\n        .file_name()\n        .to_string_lossy()\n        .starts_with(".tmp.")));\n    Ok(())\n}')

content = content.replace('    fs::remove_dir_all(dir.0).expect("failed to delete dir");\n}', '    fs::remove_dir_all(dir.0).expect("failed to delete dir");\n    Ok(())\n}')

content = content.replace('    assert_eq!(fs::read_to_string(victim)?, "unchanged");\n}', '    assert_eq!(fs::read_to_string(victim)?, "unchanged");\n    Ok(())\n}')

content = content.replace('        fs::metadata(&path)?.permissions().mode() & 0o777,\n        0o644\n    );\n}', '        fs::metadata(&path)?.permissions().mode() & 0o777,\n        0o644\n    );\n    Ok(())\n}')

content = content.replace('    assert_eq!(fs::read(&path)?, corrupt);\n', '    assert_eq!(fs::read(&path)?, corrupt);\n')
content = content.replace('    assert_eq!(fs::read_to_string(&secret_path)?, secret);\n}', '    assert_eq!(fs::read_to_string(&secret_path)?, secret);\n    Ok(())\n}')

content = content.replace('    assert_eq!(Config::load_from(&path)?.disabled_services.len(), 8);\n}', '    assert_eq!(Config::load_from(&path)?.disabled_services.len(), 8);\n    Ok(())\n}')

content = content.replace('    assert!(Config::load_from(&second)?.is_enabled("plex"));\n}', '    assert!(Config::load_from(&second)?.is_enabled("plex"));\n    Ok(())\n}')


with open(file_path, 'w') as f:
    f.write(content)
