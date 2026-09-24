import re

file_path = 'server_manager/src/core/users.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = re.sub(r'#\[allow\(clippy::unwrap_used, clippy::expect_used, clippy::panic\)\]\n', '', content)
content = re.sub(r'fn (test_\w+)\(\) \{', r'fn \1() -> anyhow::Result<()> {', content)

content = re.sub(r'\.expect\("([^"]+)"\)', r'.ok_or_else(|| anyhow::anyhow!("\1"))?', content)

content = content.replace(
    'assert!(manager.verify("testuser", "newpass").is_none());\n    }',
    'assert!(manager.verify("testuser", "newpass").is_none());\n        Ok(())\n    }'
)

content = content.replace(
    'assert!(!u2.installed_apps.contains("plex"));\n    }',
    'assert!(!u2.installed_apps.contains("plex"));\n        Ok(())\n    }'
)

content = content.replace(
    'assert!(manager.delete_user("admin").is_ok());\n    }',
    'assert!(manager.delete_user("admin").is_ok());\n        Ok(())\n    }'
)

content = content.replace(
    'assert_eq!(updated_u.quota_gb, Some(50));\n    }',
    'assert_eq!(updated_u.quota_gb, Some(50));\n        Ok(())\n    }'
)

content = content.replace(
    'assert!(manager.verify("legacy_user", "legacy_password").is_some());\n    }',
    'assert!(manager.verify("legacy_user", "legacy_password").is_some());\n        Ok(())\n    }'
)

content = content.replace(
    'assert!(!Role::Auditor.can_trigger_updates());\n    }',
    'assert!(!Role::Auditor.can_trigger_updates());\n        Ok(())\n    }'
)

content = content.replace(
    'assert_eq!(usernames, vec!["alice", "bob", "charlie"]);\n    }',
    'assert_eq!(usernames, vec!["alice", "bob", "charlie"]);\n        Ok(())\n    }'
)

# For Result objects, replacing expect with ok_or_else gives a type mismatch (because the Ok type isn't Option).
# We must use ? on Results directly. Let's find occurrences of Result expecting and replace them.
# The `add_user` function returns a Result.
content = re.sub(r'manager\s*\.add_user\(([^)]+)\)\s*\.ok_or_else\([^)]+\)\?', r'manager.add_user(\1)?', content)

# hash_password returns a Result.
content = re.sub(r'hash_password\(([^)]+)\)\s*\.ok_or_else\([^)]+\)\?', r'hash_password(\1)?', content)

# bcrypt::hash returns a Result.
content = re.sub(r'bcrypt::hash\(([^)]+)\)\s*\.ok_or_else\([^)]+\)\?', r'bcrypt::hash(\1)?', content)

with open(file_path, 'w') as f:
    f.write(content)
