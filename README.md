# Cylae Server Manager (CSM)

![Version](https://img.shields.io/badge/Version-8.1-blue) ![Stability](https://img.shields.io/badge/Stability-Production--Grade-green)

A professional, "turnkey" solution for deploying a self-hosted infrastructure on Debian or Ubuntu. Designed for absolute stability, security, and ease of use.

## 🚀 Quick Start (The One-Liner)

Copy and paste this command into your terminal. It handles everything:

```bash
sudo apt update && sudo apt install -y git && cd /opt && sudo git clone https://github.com/Cylae/server_script.git cylae-manager && cd cylae-manager && sudo chmod +x install.sh && sudo ./install.sh
```

---

## 📋 Prerequisites (Pre-Flight Check)

Before you begin, ensure you have:

1.  **A Fresh Server:**
    *   **OS:** Debian 11/12 or Ubuntu 20.04/22.04/24.04.
    *   **Architecture:** x86_64 / amd64.
    *   **Hardware:**
        *   Minimum: 1 vCPU, 2GB RAM (Low Profile mode)
        *   Recommended: 2 vCPU, 4GB RAM (High Performance mode)
2.  **Domain Name:** You must own a domain (e.g., `example.com`) pointing to your server's public IP.
3.  **Root Access:** You need `root` or `sudo` privileges.
4.  **Ports Open:** Ensure ports `80` (HTTP) and `443` (HTTPS) are open in your provider's firewall.

---

## 🛠 Features

*   **Modular Design:** Install only what you need.
*   **Docker-Native:** All services run in isolated containers.
*   **Secure by Default:**
    *   Automatic SSL (Let's Encrypt).
    *   Firewall (UFW) & Fail2Ban configured out-of-the-box.
    *   Kernel hardening and network stack tuning (BBR enabled).
*   **Centralized Management:**
    *   Single Dashboard.
    *   Unified Database (MariaDB).
    *   Automated Backups & Updates.

### Supported Services
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

### Credentials
Passwords are generated automatically and stored securely.
*   **View Credentials:** Select option `c. Show Credentials` in the menu.
*   **File Location:** `/root/.auth_details` (Root only).

### Backups
Backups include the database and configuration files.
*   **Location:** `/var/backups/cyl_manager/`
*   **Manual Backup:** Select option `b. Backup Data`.

### Updates
*   **System Update:** Select option `s. System Update` (Updates OS, Docker images, and the script).
*   **Auto-Update:** The system updates itself nightly at 04:00 AM.

---

## ❓ Deep Troubleshooting

### "The script fails immediately with exit code 1"
*   **Cause:** Usually permission errors or missing dependencies.
*   **Fix:** Ensure you run with `sudo`. The latest version automatically handles this. Check logs at `/var/log/server_manager.log`.

### "Port 80/443 already in use"
*   **Cause:** Another web server (e.g., default Apache) is running.
*   **Fix:** The script attempts to fix this, but you can manually run: `sudo apt remove apache2 -y`.

### "SSL Certificate Generation Failed"
*   **Cause:** DNS is not propagating or Firewall is blocking.
*   **Fix:**
    1.  Verify your domain points to the server IP: `ping example.com`
    2.  Ensure port 80 is open.
    3.  Run option `r. Refresh Infrastructure` to retry.

### "Docker service not starting"
*   **Cause:** Port conflict or configuration error.
*   **Fix:** Check container logs: `docker logs <container_name>`.

---

## 🏗 Architecture

*   **Core:** Bash scripts in `src/lib/`.
*   **Services:** Modular scripts in `src/services/`.
*   **State:**
    *   Config: `/etc/cyl_manager.conf`
    *   Auth: `/root/.auth_details`
    *   Data: `/opt/<service_name>`
*   **Proxy:** Nginx acts as a reverse proxy, handling SSL termination and routing traffic to Docker containers.

---

**Author:** Cylae Team
**License:** MIT
