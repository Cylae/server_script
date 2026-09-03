# Port Matrix — Cylae/server_script

## Authoritative Specification
This document defines the normative port allocation matrix for all services managed by `server_manager`.
This matrix is derived directly from the typed service catalog in `src/services/` and verified
programmatically by contract tests (`server_manager/tests/contract_port_matrix.rs`).

## Security Tiers
- **Localhost Only (`127.0.0.1`)**: Internal microservice web UIs and management APIs bound strictly to loopback to prevent direct external exposure without reverse proxy authentication.
- **Public (`0.0.0.0` / All Interfaces)**: Ingress services requiring external exposure (Reverse Proxy, VPN, DNS, Mail, Media streaming, Git SSH, Torrent peer discovery).
- **Internal Only (`None`)**: Databases and cache instances attached strictly to the Docker bridge network (`server_manager_net`) with no exposed host ports.

## Port Matrix Table

| Category | Service | Host Port | Container Port | Protocol | Host Binding | Security Tier | Description |
|:---|:---|:---|:---|:---|:---|:---|:---|
| Infrastructure | server_manager | 8099 | 8099 | TCP | 127.0.0.1:8099 | Localhost Only | Server Manager Core Web Administration Interface |
| Media | plex | 32400 | 32400 | TCP | 0.0.0.0:32400 | Public | Media Streaming Platform |
| Media | tautulli | 8181 | 8181 | TCP | 127.0.0.1:8181 | Localhost Only | Plex Statistics & Analytics |
| Media | overseerr | 5055 | 5055 | TCP | 127.0.0.1:5055 | Localhost Only | Media Request Management (Plex) |
| Media | jellyfin | 8096 | 8096 | TCP | 0.0.0.0:8096 | Public | Open Source Media Server |
| Media | jellyseerr | 5056 | 5055 | TCP | 127.0.0.1:5056 | Localhost Only | Media Request Management (Jellyfin) |
| Automation & Arr | sonarr | 8989 | 8989 | TCP | 127.0.0.1:8989 | Localhost Only | Smart TV Series Manager |
| Automation & Arr | radarr | 7878 | 7878 | TCP | 127.0.0.1:7878 | Localhost Only | Movie Manager |
| Automation & Arr | prowlarr | 9696 | 9696 | TCP | 127.0.0.1:9696 | Localhost Only | Torrent & Usenet Indexer Sync |
| Automation & Arr | jackett | 9117 | 9117 | TCP | 127.0.0.1:9117 | Localhost Only | Indexer Proxy |
| Automation & Arr | bazarr | 6767 | 6767 | TCP | 127.0.0.1:6767 | Localhost Only | Subtitle Management |
| Download | qbittorrent | 8080 | 8080 | TCP | 127.0.0.1:8080 | Localhost Only | BitTorrent Client |
| Download | qbittorrent | 6881 | 6881 | TCP | 0.0.0.0:6881 | Public | BitTorrent Client |
| Download | qbittorrent | 6881 | 6881 | UDP | 0.0.0.0:6881 | Public | BitTorrent Client |
| Infrastructure | mariadb | - | - | - | - | Internal Only | Internal Relational SQL Database |
| Infrastructure | redis | - | - | - | - | Internal Only | Internal In-Memory Cache |
| Infrastructure | nginx-proxy | 80 | 80 | TCP | 0.0.0.0:80 | Public | Reverse Proxy & SSL Management |
| Infrastructure | nginx-proxy | 81 | 81 | TCP | 0.0.0.0:81 | Public | Reverse Proxy & SSL Management |
| Infrastructure | nginx-proxy | 443 | 443 | TCP | 0.0.0.0:443 | Public | Reverse Proxy & SSL Management |
| Infrastructure | dnscrypt-proxy | 5300 | 5053 | TCP | 0.0.0.0:5300 | Public | Secure DNS Over HTTPS/TLS Proxy |
| Infrastructure | dnscrypt-proxy | 5300 | 5053 | UDP | 0.0.0.0:5300 | Public | Secure DNS Over HTTPS/TLS Proxy |
| Infrastructure | wireguard | 51820 | 51820 | UDP | 0.0.0.0:51820 | Public | Secure Fast VPN Gateway |
| Infrastructure | portainer | 9000 | 9000 | TCP | 127.0.0.1:9000 | Localhost Only | Docker Container Management Web UI |
| Infrastructure | netdata | 19999 | 19999 | TCP | 127.0.0.1:19999 | Localhost Only | Real-time System Metrics & Monitoring |
| Infrastructure | uptime-kuma | 3001 | 3001 | TCP | 127.0.0.1:3001 | Localhost Only | Service Health & Uptime Monitoring |
| Apps & Utility | vaultwarden | 8001 | 80 | TCP | 127.0.0.1:8001 | Localhost Only | Bitwarden-Compatible Lightweight Password Vault |
| Apps & Utility | filebrowser | 8002 | 80 | TCP | 127.0.0.1:8002 | Localhost Only | Web-Based File Manager |
| Apps & Utility | yourls | 8003 | 80 | TCP | 127.0.0.1:8003 | Localhost Only | Self-Hosted URL Shortener |
| Apps & Utility | glpi | 8088 | 80 | TCP | 127.0.0.1:8088 | Localhost Only | IT Asset Management & Service Desk |
| Apps & Utility | gitea | 3000 | 3000 | TCP | 127.0.0.1:3000 | Localhost Only | Self-Hosted Git Service |
| Apps & Utility | gitea | 2222 | 22 | TCP | 0.0.0.0:2222 | Public | Self-Hosted Git Service |
| Apps & Utility | roundcube | 8090 | 80 | TCP | 127.0.0.1:8090 | Localhost Only | Browser-Based Multilingual Webmail Client |
| Apps & Utility | nextcloud | 4443 | 443 | TCP | 127.0.0.1:4443 | Localhost Only | Self-Hosted Productivity & File Sync Platform |
| Apps & Utility | mailserver | 25 | 25 | TCP | 0.0.0.0:25 | Public | Full-Featured Production Mail Server |
| Apps & Utility | mailserver | 143 | 143 | TCP | 0.0.0.0:143 | Public | Full-Featured Production Mail Server |
| Apps & Utility | mailserver | 587 | 587 | TCP | 0.0.0.0:587 | Public | Full-Featured Production Mail Server |
| Apps & Utility | mailserver | 993 | 993 | TCP | 0.0.0.0:993 | Public | Full-Featured Production Mail Server |
| Apps & Utility | syncthing | 8384 | 8384 | TCP | 127.0.0.1:8384 | Localhost Only | Continuous Decentralized File Synchronization |
| Apps & Utility | syncthing | 22000 | 22000 | TCP | 0.0.0.0:22000 | Public | Continuous Decentralized File Synchronization |
| Apps & Utility | syncthing | 22000 | 22000 | UDP | 0.0.0.0:22000 | Public | Continuous Decentralized File Synchronization |
| Apps & Utility | syncthing | 21027 | 21027 | UDP | 0.0.0.0:21027 | Public | Continuous Decentralized File Synchronization |
