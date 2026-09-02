# CONTRACT.md — Established Interface & Behaviour Contract

This document freezes the established user-facing contract of `server_manager`. All future gates must preserve this contract unless explicit backwards-compatible extensions or documented breaking changes are recorded in `DECISIONS.md`.

---

## 1. CLI Commands & Options

| Subcommand | Arguments / Flags | Established Behavior |
| :--- | :--- | :--- |
| `install` | None | Full idempotent setup: system packages (fail2ban, docker), firewall rules, users, secrets, and docker-compose stacks. |
| `apply` | None | Re-evaluates configuration and updates active docker-compose stack without re-executing system package installations. |
| `update` | None | Pulls latest container images and updates active services in place. |
| `clean` | None | Performs Docker system prune, vacuums journal logs, removes temp cache files. |
| `fix` | None | Repairs file permissions on `/opt/server_manager`, verifies UFW/Docker daemon status. |
| `interactive` | None | Launches interactive administration menu. |
| `status` | None | Outputs hardware telemetry (CPU, RAM, Swap, Disk, GPU) and Docker runtime state. |
| `web` | `--port <PORT>` (default: 8099) | Launches web administration dashboard. |
| `user add` | `<username> --role <Role> --quota <GB>` | Creates new user account with role (`Admin` or `Observer`) and storage quota. |
| `user delete` | `<username>` | Deletes specified user account. |
| `user list` | None | Lists all user accounts, roles, and storage quotas. |
| `user passwd` | `<username>` | Prompts securely and updates password for specified user. |

---

## 2. Configuration & State Persistence Files

| File Path | Format | Permissions | Description |
| :--- | :--- | :--- | :--- |
| `/opt/server_manager/config.yaml` | YAML | `0644` | System configuration (domain, port bindings, profile, enabled services). |
| `/opt/server_manager/secrets.yaml` | YAML | `0600` | Administrative credentials, database passwords, API tokens. |
| `/opt/server_manager/users.yaml` | YAML | `0600` | User account database (usernames, password hashes, roles, quotas, installed apps). |
| `/opt/server_manager/docker-compose.yml` | YAML | `0644` | Generated Docker Compose stack definition. |

---

## 3. Hardware Profiles & Thresholds

| Profile | RAM Threshold | CPU Cores | Transcoding / Tuning Behavior |
| :--- | :--- | :--- | :--- |
| **LOW** | < 4 GB | <= 2 cores | Reduced memory allocations, Argon2id parameter scaling (m=19456), software transcoding fallback. |
| **STANDARD**| 4 GB - 16 GB | 4 cores | Standard allocations, hardware transcoding when GPU detected. |
| **HIGH** | > 16 GB | > 4 cores | High-performance allocations, in-memory RAM transcoding paths. |

---

## 4. Application Catalog & Port Matrix (28 Services)

| Category | Service Name | Port(s) | Default Binding |
| :--- | :--- | :--- | :--- |
| **Infrastructure** | Nginx Proxy Manager | 80, 81, 443 | Public / Localhost for Admin |
| | Portainer | 9000 | Localhost |
| | MariaDB | 3306 (Internal) | Docker Network |
| | Redis | 6379 (Internal) | Docker Network |
| | Netdata | 19999 | Localhost |
| | Uptime Kuma | 3001 | Localhost |
| | DNSCrypt Proxy | 5300 | Localhost / Docker Network |
| | WireGuard | 51820 (UDP) | Public UDP |
| **Media Servers** | Plex | 32400 | Public |
| | Jellyfin | 8096 | Public |
| | Tautulli | 8181 | Localhost |
| | Overseerr | 5055 | Localhost |
| | Jellyseerr | 5056 | Localhost |
| **Automation & Arr**| Sonarr | 8989 | Localhost |
| | Radarr | 7878 | Localhost |
| | Bazarr | 6767 | Localhost |
| | Prowlarr | 9696 | Localhost |
| | Jackett | 9117 | Localhost |
| **Download** | qBittorrent | 8080, 6881 | Localhost |
| **Apps & Utility** | Nextcloud | 4443 | Localhost |
| | Vaultwarden | 8001 | Localhost |
| | Filebrowser | 8002 | Localhost |
| | Yourls | 8003 | Localhost |
| | GLPI | 8088 | Localhost |
| | Gitea | 3000, 2222 | Localhost |
| | Roundcube | 8090 | Localhost |
| | Mailserver | 25, 143, 587, 993 | Public |
| | Syncthing | 8384, 22000 | Localhost |
