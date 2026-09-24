import re

file_path = 'server_manager/src/core/journal.rs'
with open(file_path, 'r') as f:
    content = f.read()

# Remove #[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
content = re.sub(r'#\[allow\(clippy::unwrap_used, clippy::expect_used, clippy::panic\)\]\n', '', content)

# Change test function signatures to return anyhow::Result<()>
content = re.sub(r'fn (test_\w+)\(\) \{', r'fn \1() -> anyhow::Result<()> {', content)

# Replace unwrap() with ?
content = content.replace(r'.unwrap()', r'?')

# Append Ok(()) to the end of tests (simple hack using regex for closing brace of the functions we changed)
import re

tests = re.findall(r'fn (test_\w+)\(\) -> anyhow::Result<\(\)> \{.*?(^\s*\})\n', content, re.MULTILINE | re.DOTALL)
# It's better to just use replace to add Ok(()) to the end of all test methods before they close.
# A simple way to do this is to replace `let _ = fs::remove_dir_all(&temp_dir);` followed by `\n    }`
# with `let _ = fs::remove_dir_all(&temp_dir);\n        Ok(())\n    }`

content = content.replace(
    'let _ = fs::remove_dir_all(&temp_dir);\n    }',
    'let _ = fs::remove_dir_all(&temp_dir);\n        Ok(())\n    }'
)

with open(file_path, 'w') as f:
    f.write(content)
