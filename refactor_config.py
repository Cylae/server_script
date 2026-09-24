import re

file_path = 'server_manager/src/core/config.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = re.sub(r'#\[allow\(clippy::unwrap_used, clippy::expect_used, clippy::panic\)\]\n', '', content)
content = re.sub(r'fn (test_\w+)\(\) \{', r'fn \1() -> anyhow::Result<()> {', content)
content = content.replace(r'.unwrap()', r'?')

content = content.replace(
    'let err = Config::load_from(&invalid_file);\n        assert!(err.is_err());\n        assert_eq!(err.unwrap_err().to_string(), "Invalid config YAML");',
    'let err = Config::load_from(&invalid_file);\n        assert!(err.is_err());\n        assert_eq!(err.err().ok_or_else(|| anyhow::anyhow!("Expected error"))?.to_string(), "Invalid config YAML");'
)

content = content.replace(
    'let _ = fs::remove_dir_all(&temp_dir);\n    }',
    'let _ = fs::remove_dir_all(&temp_dir);\n        Ok(())\n    }'
)

content = content.replace(
    'assert!(path.ends_with("config.yaml"));\n    }',
    'assert!(path.ends_with("config.yaml"));\n        Ok(())\n    }'
)

with open(file_path, 'w') as f:
    f.write(content)
