import re

file_path = 'server_manager/tests/integration_tests.rs'
with open(file_path, 'r') as f:
    content = f.read()

# Make all tests return anyhow::Result
content = re.sub(r'fn (test_\w+)\(\) \{', r'fn \1() -> anyhow::Result<()> {', content)

# Change expect to proper Result handling
content = content.replace('.expect("Value should exist")', '.ok_or_else(|| anyhow::anyhow!("Value missing"))?')
content = content.replace('build_compose_structure(&hw, &secrets, &config).ok_or_else(|| anyhow::anyhow!("Value missing"))?', 'build_compose_structure(&hw, &secrets, &config)?')
content = content.replace('.expect("Should parse update command")', '?')

# Add Ok(()) to the end of tests by finding the closing bracket
lines = content.split('\n')
in_test = False
new_lines = []
for i, line in enumerate(lines):
    if line.startswith('fn test_') and '() -> anyhow::Result<()> {' in line:
        in_test = True
    elif in_test and line == '}':
        new_lines.append('    Ok(())')
        in_test = False

    # We must also handle expect/unwrap replacements
    # There are unwrap() usages in tests
    line = line.replace('.unwrap()', '.ok_or_else(|| anyhow::anyhow!("unwrap failed"))?')

    new_lines.append(line)

content = '\n'.join(new_lines)

with open(file_path, 'w') as f:
    f.write(content)
