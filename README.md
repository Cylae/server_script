# Server Manager - Next-Gen Media Server Orchestrator 🚀

![Server Manager Banner](https://img.shields.io/badge/Status-Tested-brightgreen) ![Version](https://img.shields.io/badge/Version-1.0.9-blue) ![Rust](https://img.shields.io/badge/Built%20With-Rust-orange) ![Docker](https://img.shields.io/badge/Powered%20By-Docker-blue)

**Server Manager** is a powerful and intelligent tool written in Rust to deploy, manage, and optimize a complete personal media and cloud server stack. It detects your hardware and automatically configures 28 Docker services for optimal performance.

Welcome to the Server Manager documentation. Whether you are a beginner or an expert, this tool is designed to make your life easier by adopting modern "Infrastructure as Code" principles.

---

## ✨ Key Features
*   **28 Integrated Services**: Plex, ArrStack, Nextcloud, Mailserver, Vaultwarden, and more.
*   **Smart Hardware Detection**: Adapts configuration (RAM limits, Transcoding, Swap, Garbage Collection) to your machine (Low/Standard/High Profile).
*   **Secure by Default**: Automatic UFW firewall configuration, randomly generated passwords, and isolated Docker networks.
*   **GPU Support**: Automatic detection and configuration for Nvidia NVENC & Intel QuickSync.
*   **Rust-Powered Performance**: Zero-cost abstractions allow for complex hardware profiling and orchestration without startup lag.

## 🏗️ Architecture
Server Manager acts as an **Infrastructure Compiler** rather than a simple script:
1.  **Input:** Reads the host hardware state (RAM, CPU, GPU, Disk) and user configuration.
2.  **Process:** Analyzes the hardware profile (e.g., dynamically adjusting container caches and disabling memory-heavy tasks on low-spec machines).
3.  **Output:** Generates deterministic `docker-compose.yml` and configuration files safely and idempotently.
4.  **Execution:** Leverages Docker for robust, scalable container orchestration.

This Rust-first architecture ensures memory safety and avoids the fragility of external scripts.

## 🚀 Quick Installation

Server Manager is built in Rust for peak performance and reliability. You must compile it from source.

### Prerequisites
*   A server/computer running Linux (Debian 11/12 or Ubuntu 22.04+ recommended).
*   "Root" (administrator) access.
*   Docker and Docker Compose installed.

### Build from Source

Once Rust is installed and you've cloned the repository:

```text
# Build the release binary
cargo build --release

# Move to a directory in your PATH
sudo cp target/release/server_manager /usr/local/bin/

# Install dependencies, optimize system, and launch Docker Compose services
sudo server_manager install
```

Once finished, navigate to `http://YOUR-SERVER-IP:8099` (or the specific ports listed below) to view the Web Dashboard.

## 🔒 Security
Server Manager takes security seriously and applies several proactive measures:
*   **Secrets Generation:** On first launch, highly secure, cryptographic passwords and tokens are automatically generated for all databases and services (MariaDB, Nextcloud, Vaultwarden, etc.) and saved in `/opt/server_manager/secrets.yaml`. The initial Web UI `admin` password is also generated automatically.
*   **UFW Firewall Integration:** Server Manager configures Uncomplicated Firewall (UFW) to enforce a default-deny policy for incoming traffic, allowing only SSH (port 22) explicitly, leaving internal container routing and reverse proxy management up to isolated Docker networks.
*   **Role-Based Access:** The Web UI and API implement strict authentication and Role-Based Access Control (Admin/Observer).

## 🧪 Testing and CI

The project includes a comprehensive test suite to ensure robust operations. The CI pipeline runs tests and code quality checks.

To run tests locally:
```text
cd server_manager
cargo test
```

## 🛠️ CLI Commands

The `server_manager` CLI provides comprehensive orchestration commands:

*   `server_manager install`: Full idempotent installation (dependencies, firewall config, docker-compose up).
*   `server_manager apply`: Apply configuration and deploy services via Docker Compose without re-running full system setups.
*   `server_manager generate`: Dry-run generation of `docker-compose.yml` and `secrets.yaml` without launching services.
*   `server_manager status`: Display detected hardware statistics and the applied performance profile.
*   `server_manager enable <service>` / `server_manager disable <service>`: Toggle service states.
*   `server_manager web`: Starts the Web Administration Interface (Default: `http://0.0.0.0:8099`).
*   `server_manager user add <username> --quota <GB>`: Create a new system and web user with an optional quota.
*   `server_manager user delete <username>`: Remove a user and associated data.
*   `server_manager user list`: List all registered users.
*   `server_manager user passwd <username>`: Reset a user's password.

## 🌐 Web Administration Interface

Manage your active services through a secure dashboard:
1. Run `sudo server_manager web`.
2. Open `http://YOUR-SERVER-IP:8099`.
3. Login using `admin`. The password is provided during installation or can be read from `/opt/server_manager/secrets.yaml`.
4. Monitor hardware stats, manage users, and Enable/Disable services directly from the UI.

## 👥 User Management & Quotas

*   **System Integration**: Adding a user creates both a Linux system user (for SFTP/Shell access) and a Web Dashboard account.
*   **Storage Quotas**: You can optionally define storage limits using filesystem quotas.

## ⚙️ Hardware Profiles

Hardware adjustments are handled dynamically by `HardwareManager`:

| Profile | Criteria | Optimizations |
| :--- | :--- | :--- |
| **LOW** | < 4GB RAM or <= 2 Cores | Disables Disk Transcoding, ArrStack GC, and heavy Mailserver tasks (e.g., ClamAV). |
| **STANDARD** | 4-16GB RAM | Balanced configuration with moderate caching. |
| **HIGH** | > 16GB RAM | Enables RAM Transcoding (`/dev/shm`) and increased application buffer caches. |

*Note: The system actively monitors Swap space to prevent Out-Of-Memory errors on borderline configurations.*

## 💾 Data Persistence

All configurations and user states are preserved:
*   **Configuration**: `/opt/server_manager/config.yaml` (Service toggles)
*   **Secrets**: `/opt/server_manager/secrets.yaml` (Passwords, database credentials)
*   **Users**: `/opt/server_manager/users.yaml` (Accounts and roles)
*   **Docker Data**: Services store data in default Docker volumes (`/var/lib/docker/volumes`).

*Backup `/opt/server_manager` and your Docker volumes to securely save your setup.*

## 🧩 Services and Ports

*Services marked with `(Localhost)` bind to `127.0.0.1` and are inaccessible directly via public IP for security. Access them via Nginx Proxy Manager or an SSH tunnel.*

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
| **Download** | QBittorrent | 8080 (Local), 6881 | `http://localhost:8080` | Torrent Client |
| **Apps** | Nextcloud | 4443 (Localhost) | `https://localhost:4443` | Personal Cloud |
| | Vaultwarden | 8001 (Localhost) | `http://localhost:8001` | Password Manager |
| | Filebrowser | 8002 (Localhost) | `http://localhost:8002` | Web File Manager |
| | Yourls | 8003 (Localhost) | `http://localhost:8003` | URL Shortener |
| | GLPI | 8088 (Localhost) | `http://localhost:8088` | IT Asset Management |
| | Gitea | 3000 (Local), 2222 | `http://localhost:3000` | Self-hosted Git |
| | Roundcube | 8090 (Localhost) | `http://localhost:8090` | Webmail |
| | Mailserver | 25, 143, 587, 993 | - | Full Mail Server |
| | Syncthing | 8384 (Local), 22000 | `http://localhost:8384` | File Synchronization |

---

*Built with ❤️ by the Server Manager Team.*
