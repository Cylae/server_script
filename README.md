# 🚀 Cylae Server Manager (v7.0)

![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Bash](https://img.shields.io/badge/language-Bash-4EAA25.svg) ![Docker](https://img.shields.io/badge/container-Docker-2496ED.svg) ![Status](https://img.shields.io/badge/status-Stable-success.svg)

> **The Ultimate "Turnkey" Self-Hosting Solution.**
> *Universal Edition | Auto-Tuning | Modular | Secure by Default*

---

## 🌟 Why this script?

You have a fresh VPS (Debian/Ubuntu) and you want to host your own services (Nextcloud, Gitea, Bitwarden/Vaultwarden, VPN...).
Normally, you would spend hours configuring Nginx, setting up Docker, securing SSH, creating databases, and managing SSL certificates.

**Cylae Server Manager** does it all for you in **minutes**.

It is designed to be **The Best Script EVER**:
*   **Intelligent**: Detects your hardware (RAM) and tunes MySQL/PHP config accordingly.
*   **Modular**: Install/Remove services cleanly without leaving junk behind.
*   **Secure**: Hardens SSH, configures Firewall (UFW), and sets up Nginx with security headers.
*   **Automated**: Updates itself, your system, and your docker containers every night.
*   **Unified**: Provides a beautiful dashboard to access all your services.

---

## 🛠️ Installation

**Prerequisites:**
*   A fresh **Debian 11/12** (Recommended) or **Ubuntu 20.04/22.04** server.
*   Root access.
*   A domain name pointing to your server IP.

### Quick Start

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/cylae-server-manager.git /opt/cylae-manager
    cd /opt/cylae-manager
    ```

2.  **Run the script:**
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

3.  **Follow the wizard:**
    *   Enter your domain name.
    *   Select services to install from the menu.

---

## 📦 Services Catalog

All services are deployed via **Docker** for maximum isolation and stability, served behind **Nginx** with automatic **Let's Encrypt SSL**.

| Service | Description | URL |
| :--- | :--- | :--- |
| **Gitea** | Lightweight Git hosting (Github alternative). | `https://git.yourdomain.com` |
| **Nextcloud** | File hosting & sharing (Google Drive alternative). | `https://cloud.yourdomain.com` |
| **Vaultwarden** | Password manager (Bitwarden compatible). | `https://pass.yourdomain.com` |
| **Mail Server** | Full stack mail server (Postfix, Dovecot, SpamAssassin). | `https://mail.yourdomain.com` |
| **Uptime Kuma** | Monitoring tool to track uptime of services. | `https://status.yourdomain.com` |
| **WireGuard** | Modern, fast VPN with web UI (wg-easy). | `https://vpn.yourdomain.com` |
| **File Browser** | Web-based file manager. | `https://files.yourdomain.com` |
| **YOURLS** | URL Shortener. | `https://x.yourdomain.com` |
| **Portainer** | GUI for managing Docker containers. | `https://portainer.yourdomain.com` |
| **Netdata** | Real-time performance monitoring. | `https://netdata.yourdomain.com` |

> **Note:** Databases are managed via a centralized MariaDB instance (bare-metal) for performance, accessible via **Adminer** on the dashboard.

---

## ⚙️ Advanced Features

### 🧠 Smart Auto-Tuning
The script checks your RAM on every run:
*   **Low Profile (< 4GB)**: Optimizes for stability. Reduces database buffers and PHP workers.
*   **High Profile (>= 4GB)**: Optimizes for speed. Increases cache sizes and connection limits.

### 🛡️ Security
*   **SSH Hardening**: Option to disable password login and change SSH port.
*   **Firewall**: UFW is configured to deny all incoming traffic except SSH, HTTP/S, and specific service ports.
*   **Isolation**: Docker containers run in a dedicated network.
*   **Updates**:
    *   Daily System Updates (`apt-get upgrade`)
    *   Daily Docker Updates (`Watchtower`)
    *   Daily Self-Updates (`git pull`)

### 📂 Directory Structure
*   **Config**: `/etc/cyl_manager.conf`
*   **Logs**: `/var/log/server_manager.log`
*   **Credentials**: `/root/.auth_details` (Contains generated passwords)
*   **Service Data**: `/opt/<service_name>`
*   **Backups**: `/var/backups/cyl_manager`

---

## ❓ Troubleshooting

**Q: I added a service but the URL doesn't work.**
A: Ensure you have created the DNS record (CNAME) for the subdomain. Use option `d` in the menu to see required records. Then run option `r` (Sync All) to refresh Nginx and SSL.

**Q: How do I access the Database?**
A: Go to your main dashboard (`https://admin.yourdomain.com`) and click "DB Admin". Login with `root` and the password found in `/root/.auth_details`.

**Q: The script failed during installation.**
A: Check the logs at `/var/log/server_manager.log` for detailed error messages.

**Q: How do I restore a backup?**
A: Backups are stored in `/var/backups/cyl_manager`.
*   **Database**: `mysql < db_backup.sql`
*   **Files**: Extract the tarball to root: `tar -xzf files_backup.tar.gz -C /`

---

## 🤝 Contributing
Feel free to open issues or pull requests to make this script even better!

*v7.0 - Ultimate Edition*
