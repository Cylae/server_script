import re

file_path = 'server_manager/src/core/config.rs'
with open(file_path, 'r') as f:
    content = f.read()

content = content.replace('content.find("plex")?;', 'content.find("plex").ok_or_else(|| anyhow::anyhow!("plex not found"))?;')
content = content.replace('content.find("radarr")?;', 'content.find("radarr").ok_or_else(|| anyhow::anyhow!("radarr not found"))?;')
content = content.replace('content.find("sonarr")?;', 'content.find("sonarr").ok_or_else(|| anyhow::anyhow!("sonarr not found"))?;')

with open(file_path, 'w') as f:
    f.write(content)
