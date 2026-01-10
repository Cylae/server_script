# Cylae Server Manager

A production-grade, modular, and secure server management suite for Debian/Ubuntu systems. Easily deploy and manage self-hosted services using Docker.

## 🚀 Features

*   **Modular Architecture:** Core logic separated from service definitions.
*   **Docker-Native:** All services run in isolated Docker containers.
*   **Secure by Default:**
    *   Hardened SSH and Firewall (UFW).
    *   Fail2Ban pre-configured.
    *   Automatic SSL (Let's Encrypt) via Certbot.
    *   Nginx with HSTS and HTTP/2.
*   **Performance Tuned:** Smart system tuning based on available RAM (Kernel, TCP, MariaDB, PHP).
*   **Centralized Management:** Unified dashboard and CLI tool.

## 📋 Prerequisites

*   **OS:** Debian 11/12 or Ubuntu 20.04/22.04/24.04 (LTS recommended).
*   **User:** Root privileges (must run as `root`).
*   **Ports:** 80, 443, 22 (and service specific ports) must be open.
*   **Domain:** A valid domain name pointing to your server IP.

## 🛠️ Installation

1.  **Switch to Root**
    ```bash
    sudo -i
    ```

2.  **Install Dependencies**
    ```bash
    apt update && apt install -y git ca-certificates
    ```

3.  **Clone the Repository**
    ```bash
    cd /opt/
    git clone https://github.com/Cylae/server_script.git cylae-manager
    cd cylae-manager
    ```

4.  **Run the Installer**
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

## 🎮 Usage

Once installed, the manager provides an interactive menu. You can access it anytime by running the script again or using the installed alias (if applicable, or just run `./install.sh`).

**Menu Options:**
*   **1-12:** Manage specific services (Install/Remove).
*   **s:** System Update (OS + Docker Images).
*   **b:** Backup Data (Database dumps + File archives).
*   **r:** Refresh Infrastructure (Regenerate Nginx configs & SSL).
*   **t:** Tune System (Re-apply performance profiles).
*   **c:** Show Credentials (View saved passwords).
*   **h:** Hardening & SSH (Change SSH port, verify security).

## 📦 Supported Services

*   **Gitea** (Git Service)
*   **Nextcloud** (Cloud Storage)
*   **Portainer** (Docker Management)
*   **Netdata** (Monitoring)
*   **Mail Server** (Postfix/Dovecot stack)
*   **YOURLS** (URL Shortener)
*   **FTP** (vsftpd)
*   **Vaultwarden** (Password Manager)
*   **Uptime Kuma** (Uptime Monitoring)
*   **WireGuard** (VPN)
*   **FileBrowser** (Web File Manager)
*   **GLPI** (IT Asset Management)

## ❓ Troubleshooting

### The script fails immediately
*   Check the log file: `/var/log/server_manager.log`
*   Ensure you are running as **root**.
*   Ensure you have internet connectivity.

### "Command not found" errors
*   Ensure you ran the `apt install` step for git and ca-certificates.
*   The script attempts to install other dependencies (jq, curl, etc.) automatically.

### Service not accessible
*   Verify DNS records using option `d` in the menu.
*   Run `r` (Refresh Infrastructure) to ensure Nginx and SSL are correctly configured.
*   Check firewall status: `ufw status`.

### Database Connection Errors
*   Use option `c` (Show Credentials) to verify the generated passwords match what is configured in the service.
*   Ensure the `server-net` docker network exists.
