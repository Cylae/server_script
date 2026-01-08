# Cylae Server Manager

Cylae Server Manager is a robust, interactive, and secure tool for managing self-hosted services using Docker on Debian and Ubuntu servers. It simplifies the deployment of services like Gitea, Nextcloud, Mail Server, and more, handling SSL certificates, security hardening, and backups automatically.

## Features

*   **Interactive Menu**: Easy-to-use menu system for installing and managing services.
*   **Secure Credentials**: Automatic generation of strong, random passwords for databases and services.
*   **Customizable Credentials**: Option to specify your own usernames and passwords during installation.
*   **Automatic SSL**: Integrated Let's Encrypt support for automatic SSL/TLS configuration.
*   **Safety Checks**: Built-in checks for port conflicts and resource availability to prevent issues.
*   **System Tuning**: Automatic system tuning (database, PHP, kernel) based on available RAM (Low/High profile).
*   **Backups**: Integrated backup solution for databases and persistent data.
*   **SSH Hardening**: Utilities to secure SSH access.
*   **Auto-Update**: Self-updating mechanism to keep the manager script current.

## Supported Services

*   **Gitea**: Git hosting service.
*   **Nextcloud**: Productivity platform.
*   **Portainer**: Docker container management.
*   **Netdata**: Real-time performance monitoring.
*   **Mail Server**: Full-stack mail server (Postfix, Dovecot, SpamAssassin, ClamAV).
*   **YOURLS**: URL shortener.
*   **FTP Server**: vsftpd for file transfers.
*   **Vaultwarden**: Bitwarden compatible password manager server.
*   **Uptime Kuma**: Monitoring tool.
*   **WireGuard**: Modern VPN server (via wg-easy).
*   **FileBrowser**: Web-based file manager.

## Installation

1.  **Download the script**:
    ```bash
    wget -O install.sh https://github.com/Cylae/server_script/raw/main/install.sh
    chmod +x install.sh
    ```

2.  **Run as root**:
    ```bash
    sudo ./install.sh
    ```

3.  **Follow the on-screen instructions**:
    *   You will be asked to enter your domain name on the first run.
    *   Select services to install from the menu.
    *   You will be prompted to set usernames/passwords for each service, or you can press Enter to auto-generate secure credentials.

## Usage

Run the script anytime to access the management menu:

```bash
sudo server_manager.sh
```

### Options

*   **Install/Remove Services**: Select a number (1-11) to toggle installation.
*   **s. System Update**: Update system packages and Docker images.
*   **b. Backup Data**: Backup all databases and service data to `/var/backups/cyl_manager`.
*   **r. Sync Infrastructure**: Reload Nginx and check for SSL renewals.
*   **t. Tune System**: Re-apply system tuning profiles.
*   **d. Show DNS Records**: Display required DNS records for installed services.
*   **c. Show Credentials**: Display all saved/generated credentials.
*   **h. Harden SSH**: Configure SSH key authentication and change port.

## Directory Structure

*   `/opt/<service>`: Service data and docker-compose files.
*   `/etc/cyl_manager.conf`: Main configuration file.
*   `/root/.auth_details`: Securely stored credentials (read-only by root).
*   `/var/backups/cyl_manager`: Backup location.

## Security Note

This script generates random passwords for database users and service admins if you do not provide them. These are stored in `/root/.auth_details`. **Protect this file.**

## License

MIT License
