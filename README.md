# Cylae Server Manager (CSM) - Sheldon Edition

![Version](https://img.shields.io/badge/Version-9.3-blue) ![Stability](https://img.shields.io/badge/Stability-Production--Grade-green) ![Tests](https://img.shields.io/badge/Tests-100%2B-brightgreen)

> "Bazinga! It's not just a script; it's a meticulously crafted symphony of logic."

A professional, "turnkey" solution for deploying a self-hosted infrastructure on Debian or Ubuntu. Designed for absolute stability, security, and ease of use, adhering to the strictest scientific standards of organization.

## 🚀 Quick Start (The One-Liner)

Copy and paste this command into your terminal. It handles everything with optimal efficiency:

```bash
sudo apt update && sudo apt install -y git && cd /opt && sudo git clone https://github.com/Cylae/server_script.git cylae-manager && cd cylae-manager && sudo chmod +x install.sh && sudo ./install.sh
```

---

## 📋 Prerequisites (Pre-Flight Check)

The script enforces strict resource checks. Attempting to run this on substandard hardware is futile.

1.  **A Fresh Server:**
    *   **OS:** Debian 11/12 (Recommended) or Ubuntu 20.04/22.04/24.04.
    *   **Architecture:** x86_64 / amd64.
    *   **Hardware:**
        *   Minimum: 1 vCPU, 2GB RAM (Strict check: <5GB disk space will block installation).
        *   Recommended: 2 vCPU, 4GB RAM (High Performance mode).
2.  **Domain Name:** You must own a domain (e.g., `example.com`) pointing to your server's public IP.
3.  **Root Access:** You need `root` or `sudo` privileges.
4.  **Ports Open:** Ensure ports `80` (HTTP) and `443` (HTTPS) are open in your provider's firewall.

---

## 🛠 Features

*   **Modular Design:** Install only what you need.
*   **Docker-Native:** All services run in isolated containers.
*   **Media Stack:** A comprehensive suite for media acquisition and playback (Plex, *arr suite).
*   **Secure by Default:**
    *   Automatic SSL (Let's Encrypt).
    *   Firewall (UFW) & Fail2Ban configured out-of-the-box.
    *   **Auto-Security Updates:** Unattended upgrades enabled for the OS.
    *   Kernel hardening and network stack tuning (BBR enabled).
    *   **Secure Password Policy:** Enforces complexity (Upper, Lower, Digit).
*   **Adaptive Performance:**
    *   **Smart Swap:** Dynamic swap size allocation based on RAM.
    *   **ZRAM:** Automatic memory compression for low-spec VMs.
    *   **Docker Optimization:** Global log rotation to prevent disk saturation.
*   **Reliability:**
    *   **Self-Healing Watchdog:** Auto-restarts critical services (Nginx/DB) if they crash.
    *   **Robust Error Handling:** Strict mode enabled (`set -euo pipefail`).
*   **Centralized Management:**
    *   Single Dashboard.
    *   Unified Database (MariaDB).
    *   Automated Backups & Updates.

### Supported Services

#### Core Infrastructure
| Service | Description | URL |
| :--- | :--- | :--- |
| **Nextcloud** | Cloud storage & collaboration | `cloud.example.com` |
| **Gitea** | Git service (Github alternative) | `git.example.com` |
| **Vaultwarden** | Password manager (Bitwarden) | `pass.example.com` |
| **Portainer** | Docker container management | `portainer.example.com` |
| **Uptime Kuma** | Monitoring & Status Page | `status.example.com` |
| **WireGuard** | Modern VPN | `vpn.example.com` |
| **Mail Server** | Full stack email (Postfix/Dovecot) | `mail.example.com` |
| **FileBrowser** | Web-based file manager | `files.example.com` |
| **GLPI** | IT Asset Management | `support.example.com` |
| **Netdata** | Real-time performance monitoring | `netdata.example.com` |
| **YOURLS** | URL Shortener | `x.example.com` |
| **FTP** | File Transfer Protocol (vsftpd) | `ftp://example.com` |

#### Media Stack (The Fun Part)
| Service | Description | URL |
| :--- | :--- | :--- |
| **Plex** | Media Server | `plex.example.com` |
| **Tautulli** | Plex Monitoring | `tautulli.example.com` |
| **Sonarr** | TV Show PVR | `sonarr.example.com` |
| **Radarr** | Movie PVR | `radarr.example.com` |
| **Prowlarr** | Indexer Manager | `prowlarr.example.com` |
| **Jackett** | Legacy Indexer Proxy | `jackett.example.com` |
| **Overseerr** | Request Management | `request.example.com` |
| **qBittorrent** | Torrent Client | `qbittorrent.example.com` |

---

## 📖 Step-by-Step Installation Guide

If you prefer to run commands manually instead of the one-liner:

### 1. Update and Install Git
```bash
sudo apt update
sudo apt install -y git
```

### 2. Clone the Repository
We recommend installing in `/opt/cylae-manager`.
```bash
cd /opt
sudo git clone https://github.com/Cylae/server_script.git cylae-manager
```

### 3. Run the Installer
```bash
cd cylae-manager
sudo chmod +x install.sh
sudo ./install.sh
```

### 4. Follow the Wizard
The script will ask for:
*   **Domain Name:** Enter your domain (e.g., `example.com`).
*   **Profile:** It automatically detects RAM and suggests a profile (Low/High).

---

## ⚙️ Configuration & Maintenance

### Managing Services
Run the script anytime to access the main menu:
```bash
sudo /usr/local/bin/server_manager.sh
```
Or simply:
```bash
cd /opt/cylae-manager && ./install.sh
```

### Identifiants
Passwords are generated automatically and stored securely.
*   **View Credentials:** Select option `c. Show Credentials` in the menu.
*   **File Location:** `/root/.auth_details` (Root only).

### Backups & Restore
Backups include the database and configuration files.
*   **Location:** `/var/backups/cyl_manager/`
*   **Policy:** Files older than 7 days are automatically deleted.
*   **Compression:** Uses parallel compression (pigz) for speed if available.
*   **Manual Backup:** Select option `b. Backup Data`.
*   **Restore:** Select option `x. Restore from Backup` to restore files and databases.

### Health & Monitoring
*   **Health Check:** Select `k. Health Check` to verify that all services are responding (HTTP 200 OK).
*   **Uptime Kuma:** Install Uptime Kuma for historical uptime tracking.

### Updates
*   **System Update:** Select option `s. System Update` (Updates OS, Docker images, and the script).
*   **Auto-Update:** The system updates itself nightly at 04:00 AM using a robust Git-based mechanism.

---

## 🏗 Architecture

*   **Core:** Bash scripts in `src/lib/`.
*   **Services:** Modular scripts in `src/services/`.
*   **Media:** Unified media structure in `/opt/media` (Movies, TV, Downloads).
*   **State:**
    *   Config: `/etc/cyl_manager.conf`
    *   Auth: `/root/.auth_details`
    *   Data: `/opt/<service_name>`
*   **Proxy:** Nginx acts as a reverse proxy, handling SSL termination and routing traffic to Docker containers.

---

**Auteur :** Équipe Cylae
**Licence :** MIT
