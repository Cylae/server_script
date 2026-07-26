# Server Manager - Next-Gen Media Server Orchestrator 🚀

![Server Manager Banner](https://img.shields.io/badge/Status-Tested-brightgreen) ![Version](https://img.shields.io/badge/Version-1.0.9-blue) ![Rust](https://img.shields.io/badge/Built%20With-Rust-orange) ![Docker](https://img.shields.io/badge/Powered%20By-Docker-blue)

**Server Manager** is a powerful and intelligent tool written in Rust to deploy, manage, and optimize a complete personal media and cloud server stack. It detects your hardware and automatically configures 28 Docker services for optimal performance.

---

Welcome to the Server Manager documentation. Whether you are a beginner or an expert, this tool is designed to make your life easier.

## ✨ Key Features
*   **28 Integrated Services**: Plex, ArrStack, Nextcloud, Mailserver, etc.
*   **Smart Hardware Detection**: Adapts configuration (RAM, Transcoding, Swap) to your machine (Low/Standard/High Profile).
*   **Secure by Default**: UFW firewall configured, passwords generated, isolated networks.
*   **GPU Support**: Automatic detection and configuration for Nvidia & Intel QuickSync.

## 🚀 Quick Installation

Server Manager is built in Rust for performance and reliability. To get started, you will need to build it from source.

### Prerequisites
*   A server/computer running Linux (Debian 11/12 or Ubuntu 22.04+ recommended).
*   "Root" (administrator) access.

### Build from Source

```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/Cylae/server_script
cd server_script/server_manager
cargo build --release

# The binary is located in target/release/server_manager
sudo cp target/release/server_manager /usr/local/bin/

# Install dependencies, optimize system, and launch Docker Compose
server_manager install
```

Once finished, go to `http://YOUR-SERVER-IP:8099` (or the specific ports listed below) to view the Web Dashboard.

## 🧪 Testing

The project includes a comprehensive test suite covering hardware detection, secrets generation, and Docker Compose validation.

```sh
cd server_script/server_manager
cargo test
cargo clippy --all-targets --all-features
```

## Architecture

Server Manager is organized internally using modules to enforce separation of concerns:
- **`src/core`**: Low-level operations like hardware detection, system configuration, secret management, and handling user configurations.
- **`src/services`**: A modular system mapping definitions of external software integrations to the `Service` trait, controlling docker properties and lifecycle operations.
- **`src/interface`**: Exposes human interfaces, containing the web server interface and standard command line CLI.

## Security

Security is foundational to the design of Server Manager. The following defaults apply:
- The host firewall automatically filters all connections and exposes minimal services.
- Container networks are isolated to prevent unauthorized transverse movement.
- Passwords for deployed applications are completely randomized during installation using securely generated sequences.

## CLI Commands

The tool provides several subcommands:

*   `server_manager install`: Full idempotent installation (dependencies, config, docker-compose up).
*   `server_manager apply`: Apply configuration and deploy services (via docker-compose up) without re-running system installations.
*   `server_manager generate`: Generates `docker-compose.yml` and `secrets.yaml` only, without launching services. Useful for inspection.
*   `server_manager status`: Displays detected hardware statistics and the profile (Low/Standard/High).
*   `server_manager enable <service>`: Enable a service (e.g., `server_manager enable nextcloud`).
*   `server_manager disable <service>`: Disable a service.
*   `server_manager web`: Starts the Web Administration Interface (Default: http://0.0.0.0:8099).
*   `server_manager user add <username> --quota <GB>`: Create a new user (Role: Admin/Observer) and set storage quota.
*   `server_manager user delete <username>`: Delete a user and their data.
*   `server_manager user list`: List all users.
*   `server_manager user passwd <username>`: Reset a user's password.

## 🌐 Web Administration Interface

You can manage your services via a secure web dashboard.
1. Run `server_manager web`.
2. Open `http://YOUR-SERVER-IP:8099`.
3. Login with your credentials. (Username: `admin`. The password is randomly generated during installation and printed in the deployment summary, or can be found in `secrets.yaml`.)
4. View status and Enable/Disable services (Admin only).

## 👥 User Management & Quotas

Server Manager supports full user management:
*   **System Integration**: Adding a user creates a Linux system user (for SFTP/Shell access) and a Web Dashboard user.
*   **Storage Quotas**: You can set a storage limit (in GB) for each user. The system uses filesystem quotas to enforce this.
    *   Example: `server_manager user add john --quota 50`

## ⚙️ Hardware Profiles

Server Manager adjusts configuration via `HardwareManager`:

| Profile | Criteria | Optimizations |
| :--- | :--- | :--- |
| **LOW** | < 4GB RAM or <= 2 Cores | Disk Transcoding, ArrStack GC disabled, Minimal Mailserver (no antivirus/antispam). |
| **STANDARD** | 4-16GB RAM | Balanced configuration. |
| **HIGH** | > 16GB RAM | RAM Transcoding (`/dev/shm`), increased caches. |

*Note: Swap presence is analyzed to avoid OOM on borderline configurations (e.g., 6GB RAM without swap -> Low).*

## 🔒 Secrets Management

Passwords are stored in `secrets.yaml`.
*   Automatically generated on first launch.
*   You can modify this file *before* running `install` or `generate` if you wish to set your own passwords.

## 💾 Data Persistence

Server Manager stores its configuration and data in the following locations:
*   **Configuration**: `/opt/server_manager/config.yaml` (Enabled/Disabled services)
*   **Secrets**: `/opt/server_manager/secrets.yaml` (Passwords and tokens)
*   **Users**: `/opt/server_manager/users.yaml` (Web and System users)
*   **Docker Data**: Docker volumes are managed by Docker (usually `/var/lib/docker/volumes`).

**Backup Recommendation**: Backup the `/opt/server_manager` directory to save your configuration and user accounts.

## 🛠 Services and Ports List

Here is the matrix of deployed services:

**Note**: Services marked with `(Localhost)` are bound to `127.0.0.1` and are **not** accessible directly via the server's public IP. You must use the Reverse Proxy (Nginx Proxy Manager) or an SSH tunnel to access them.

| Category | Service | Host Port / Access | Internal URL | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Infra** | Nginx Proxy Manager | 80, 81, 443 | `http://IP:81` | Reverse Proxy & SSL |
| | Portainer | 9000 (Localhost) | `http://localhost:9000` | Docker Management |
| | MariaDB | - | `mariadb` | SQL Database (Internal) |
| | Redis | - | `redis` | Cache (Internal) |
| | Netdata | 19999 (Localhost) | `http://localhost:19999` | Real-time Monitoring |
| | Uptime Kuma | 3001 (Localhost) | `http://localhost:3001` | Uptime Monitoring |
| | DNSCrypt Proxy | 5300 | `dnscrypt-proxy` | Secure DNS (DoH) |
| | Wireguard | 51820 (UDP) | - | VPN |
| **Media** | Plex | 32400 | `http://IP:32400` | Streaming Server |
| | Jellyfin | 8096 | `http://IP:8096` | Streaming Server (Open Source) |
| | Tautulli | 8181 (Localhost) | `http://localhost:8181` | Plex Stats |
| | Overseerr | 5055 (Localhost) | `http://localhost:5055` | Plex Requests |
| | Jellyseerr | 5056 (Localhost) | `http://localhost:5056` | Jellyfin Requests |
| **ArrStack** | Sonarr | 8989 (Localhost) | `http://localhost:8989` | TV Shows |
| | Radarr | 7878 (Localhost) | `http://localhost:7878` | Movies |
| | Bazarr | 6767 (Localhost) | `http://localhost:6767` | Subtitles |
| | Prowlarr | 9696 (Localhost) | `http://localhost:9696` | Torrent Indexers |
| | Jackett | 9117 (Localhost) | `http://localhost:9117` | Indexer Proxy |
| **Download** | QBittorrent | 8080 (Localhost), 6881 | `http://localhost:8080` | Torrent Client |
| **Apps** | Nextcloud | 4443 (Localhost) | `https://localhost:4443` | Personal Cloud |
| | Vaultwarden | 8001 (Localhost) | `http://localhost:8001` | Password Manager |
| | Filebrowser | 8002 (Localhost) | `http://localhost:8002` | Web File Manager |
| | Yourls | 8003 (Localhost) | `http://localhost:8003` | URL Shortener |
| | GLPI | 8088 (Localhost) | `http://localhost:8088` | IT Asset Management |
| | Gitea | 3000 (Localhost), 2222 | `http://localhost:3000` | Self-hosted Git |
| | Roundcube | 8090 (Localhost) | `http://localhost:8090` | Webmail |
| | Mailserver | 25, 143, 587, 993 | - | Full Mail Server |
| | Syncthing | 8384 (Localhost), 22000 | `http://localhost:8384` | File Synchronization |

---

Built with ❤️ by the Server Manager Team.
