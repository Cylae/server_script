import re

file_path = 'server_manager/tests/integration_tests.rs'
with open(file_path, 'r') as f:
    content = f.read()

# Fix option unwraps replacing ? with .ok_or_else()
content = content.replace('build_compose_structure(&hw, &secrets, &config)?', 'build_compose_structure(&hw, &secrets, &config).expect("Must return compose")')
content = content.replace('compose.services.get("plex")?', 'compose.services.get("plex").ok_or_else(|| anyhow::anyhow!("Expected service"))?')
content = content.replace('compose.services.get("yourls")?', 'compose.services.get("yourls").ok_or_else(|| anyhow::anyhow!("Expected service"))?')
content = content.replace('compose.services.get("mariadb")?', 'compose.services.get("mariadb").ok_or_else(|| anyhow::anyhow!("Expected service"))?')
content = content.replace('compose.services.get("sonarr")?', 'compose.services.get("sonarr").ok_or_else(|| anyhow::anyhow!("Expected service"))?')
content = content.replace('compose.services.get("mail")\n        ?', 'compose.services.get("mail")\n        .ok_or_else(|| anyhow::anyhow!("Expected service"))?')
content = content.replace('compose.services.get("syncthing")\n        ?', 'compose.services.get("syncthing")\n        .ok_or_else(|| anyhow::anyhow!("Expected service"))?')

content = content.replace('syncthing.ports.as_ref()?', 'syncthing.ports.as_ref().ok_or_else(|| anyhow::anyhow!("Expected ports"))?')
content = content.replace('plex.networks.as_ref()?', 'plex.networks.as_ref().ok_or_else(|| anyhow::anyhow!("Expected networks"))?')
content = content.replace('sonarr.ports.as_ref()?', 'sonarr.ports.as_ref().ok_or_else(|| anyhow::anyhow!("Expected ports"))?')
content = content.replace('plex.ports.as_ref()?', 'plex.ports.as_ref().ok_or_else(|| anyhow::anyhow!("Expected ports"))?')
content = content.replace('mail.environment.as_ref()?', 'mail.environment.as_ref().ok_or_else(|| anyhow::anyhow!("Expected env"))?')

content = content.replace('mariadb.deploy.as_ref()?', 'mariadb.deploy.as_ref().ok_or_else(|| anyhow::anyhow!("Expected deploy"))?')
content = content.replace('deploy.resources.as_ref()?', 'deploy.resources.as_ref().ok_or_else(|| anyhow::anyhow!("Expected resources"))?')
content = content.replace('resources.limits.as_ref()?', 'resources.limits.as_ref().ok_or_else(|| anyhow::anyhow!("Expected limits"))?')
content = content.replace('limits.memory.as_ref()?', 'limits.memory.as_ref().ok_or_else(|| anyhow::anyhow!("Expected memory"))?')

# Add Ok(()) where missing
content = content.replace('    assert!(compose.services.len() == 28);\n}', '    assert!(compose.services.len() == 28);\n    Ok(())\n}')
content = content.replace('    assert!(plex.environment.as_ref().unwrap().contains(&"NVIDIA_VISIBLE_DEVICES=all".to_string()));\n}', '    assert!(plex.environment.as_ref().unwrap().contains(&"NVIDIA_VISIBLE_DEVICES=all".to_string()));\n    Ok(())\n}')
# Let's just fix all test bodies automatically by appending Ok(()) before the last brace.
content = re.sub(r'    \}\n\n', r'        Ok(())\n    }\n\n', content)
content = re.sub(r'    \}\n$', r'        Ok(())\n    }\n', content)

with open(file_path, 'w') as f:
    f.write(content)
