# Cylae Server Manager (Robust Edition)

**Greetings, user.**

I am Sheldon Cooper, B.S., M.S., M.A., Ph.D., Sc.D., and I have graciously decided to oversee your server infrastructure. Do try to keep up.

This repository contains the `Cylae Server Manager`, a modular, idempotent, and frankly, quite adequate Bash-based framework for deploying self-hosted services via Docker. It is designed for Debian 11/12 and Ubuntu 20.04+, operating under the assumption that you prefer order over chaos.

## 1. The Stack (Or "Why This Is Superior")

The system is architected to run a plethora of applications, harmoniously integrated behind a reverse proxy (Nginx) with automated SSL (Certbot).

### Core Services
*   **Portainer**: Docker management. (For those who need a GUI to understand containers).
*   **Netdata**: Real-time performance monitoring.
*   **WireGuard**: VPN. (Security is not optional).
*   **Uptime Kuma**: Monitoring.
*   **Vaultwarden**: Bitwarden compatible server.
*   **Nextcloud**: Cloud storage.
*   **Gitea**: Git hosting.
*   **Mail Server**: Docker-mailserver.
*   **FileBrowser**: Web-based file manager.
*   **GLPI**: IT Asset Management.

### The Media Suite (New & Improved)
We have expanded the capabilities to include a comprehensive media acquisition and playback suite. All components share a unified directory structure at `/opt/media` to ensure hard links work correctly (minimizing I/O, obviously).

*   **Plex Media Server**: The media player.
*   **Tautulli**: Monitoring for Plex. (Because one must know who is watching what).
*   **Sonarr**: TV Series PVR.
*   **Radarr**: Movie PVR.
*   **Overseerr**: Request management. (To mitigate the incessant requests from your less tech-savvy acquaintances).
*   **Prowlarr**: Indexer manager. (Superior to Jackett, though Jackett is included for legacy reasons).
*   **qBittorrent**: The download client.
*   **Jackett**: Legacy indexer proxy.

## 2. Installation (Pay Attention)

The installation process is automated, but you must execute it with the appropriate privileges.

### One-Liner
```bash
git clone https://github.com/Cylae/server_script.git cylae-manager && cd cylae-manager && chmod +x install.sh && ./install.sh
```

### Protocol
1.  **Clone** the repository.
2.  **Execute** `./install.sh`.
3.  **Follow** the prompts. Do not deviate.
4.  **Select** the services you wish to install.

## 3. Directory Structure (The "Organization")

The system enforces a strict hierarchy. Do not move files arbitrarily.

*   `/opt/<service>`: Persistent data for each service.
*   `/opt/media`: Shared media volume.
    *   `/opt/media/movies`: For Radarr.
    *   `/opt/media/tv`: For Sonarr.
    *   `/opt/media/downloads`: For qBittorrent.
*   `/etc/nginx/sites-available`: Nginx configurations.
*   `/var/backups/cyl_manager`: Where your data goes when you adhere to the backup schedule.

## 4. Usage Guidelines

*   **Idempotency**: The scripts are designed to be run multiple times. If you select "Install" on an installed service, it will update the configuration.
*   **DNS**: The script will tell you exactly what DNS records to set. Do exactly as it says.
*   **Credentials**: Use the 'Show Credentials' option in the menu. Do not write them on a post-it note.

## 5. Troubleshooting

If the system fails, it is statistically likely to be user error. However:
1.  Check the logs: `/var/log/server_manager.log`.
2.  Run `docker ps` to ensure containers are running.
3.  Verify your port forwarding.

*Bazinga.*
