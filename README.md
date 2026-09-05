# Server Manager - Professional Media Server Orchestrator 🚀

![Status](https://img.shields.io/badge/Status-Tested-brightgreen) [![CI Gate](https://github.com/Cylae/server_script/actions/workflows/rust.yml/badge.svg)](https://github.com/Cylae/server_script/actions/workflows/rust.yml) ![Rust](https://img.shields.io/badge/Built%20With-Rust-orange) ![Docker](https://img.shields.io/badge/Powered%20By-Docker-blue)

**Server Manager** is a high-performance, enterprise-grade media server management and orchestration platform written in Rust. Inspired by QuickBox Pro, it provides automated deployment, real-time hardware telemetry, 1-click application management, system maintenance tools, and secure multi-user management.

---

## 🏛️ Architecture

The system consists of a compiled Rust application (`server_manager`) operating across three core layers:
* **Core**: Hardware detection, system optimization, UFW firewall automation, fail2ban setup, secrets management, and user quotas.
* **Services**: Abstraction layer generating dynamic Docker Compose stacks with hardware-aware resource allocation (RAM transcoding, GPU passthrough).
* **Interface**: Interactive TUI CLI, REST API endpoints, and a web administration dashboard served via Tokio/Axum.

---

## 🔐 Security

Server Manager follows strict secure-by-default principles:
* **Firewall Binding**: Automatically configures UFW rules; internal/admin microservices are bound to `127.0.0.1` and isolated behind Nginx Proxy Manager.
* **Credential Protection**: Passwords and administrative tokens are cryptographically generated on initial setup and persisted with strict `0600` Unix permissions in `/opt/server_manager/secrets.yaml`.
* **Password Hashing**: Uses Argon2id password hashing with transparent migration support for legacy bcrypt hashes.
* **Role-Based Access Control (RBAC)**: Enforces four fine-grained user roles:
  * `Admin`: Full system control, user management, secret viewing, service management, and updates.
  * `Operator`: Service management, configuration updates, and stack control.
  * `Observer`: Read-only telemetry access and user dashboard view.
  * `Auditor`: Telemetry and system audit log inspection permissions.
* **Session Hardening**: Web interface incorporates session clearing on login (preventing session fixation) and injects HTTP security headers (`Content-Security-Policy`, `HSTS`, `X-Frame-Options`, `X-Content-Type-Options`).
* **Brute-Force Defense**: Integrates with system `fail2ban` services for SSH and Web endpoints.

---

## ✨ Key Features

* **28 Integrated Services**: Plex, Jellyfin, ArrStack (Sonarr, Radarr, Bazarr, Prowlarr, Jackett), Nextcloud, Vaultwarden, Gitea, GLPI, WireGuard, Netdata, and more.
* **Smart Hardware Tuning**: Automatically evaluates CPU cores, RAM, Swap, and GPU availability (Nvidia & Intel QuickSync) to select LOW, STANDARD, or HIGH optimization profiles.
* **QuickBox-Inspired CLI (`server_manager`)**:
  * `install`: Fully automated, idempotent system dependency and Docker stack setup.
  * `apply`: Applies configuration adjustments without re-installing system dependencies.
  * `update`: Pulls latest Docker container images and seamlessly updates active services.
  * `clean`: System cleanup tool (Docker prune, journal log vacuuming, temporary cache removal).
  * `fix`: Repairs file permissions, UFW firewall rules, and container environment state.
  * `interactive`: Menu console for interactive server administration.
  * `status`: Real-time system hardware telemetry and container runtime state.
  * `web`: Launches the web administration interface on port `8099`.
  * `user`: CLI user creation, password reset, quota assignment, and user listing.
* **Modern Web Dashboard**: Responsive dark-themed web administration interface featuring real-time CPU/RAM/Swap/Disk gauges, 1-click app portals, user quota management, and stack controls.

---

## 🚀 Quick Installation Guide

### Prerequisites
* A server running **Debian 11/12/13** or **Ubuntu 22.04/24.04 LTS**.
* Root (administrator) access.

### Installation Steps

```sh
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/Cylae/server_script
cd server_script/server_manager

# Build optimized release binary
cargo build --release

# Install binary globally
sudo cp target/release/server_manager /usr/local/bin/

# Run full setup
server_manager install
```

Once installed, access the Web Dashboard at `http://YOUR-SERVER-IP:8099`.

---

## ⚙️ CLI Reference

| Command | Description |
| :--- | :--- |
| `server_manager install` | Full idempotent installation of system packages, Docker, firewall, and services. |
| `server_manager apply` | Re-evaluates configuration and updates Docker Compose services without re-running system package installs. |
| `server_manager update` | Pulls latest Docker images and updates running containers without downtime. |
| `server_manager clean` | Runs Docker system prune, vacuums journal logs, and removes temp files. |
| `server_manager fix` | Audits and fixes permissions on config files and verifies UFW/Docker daemon state. |
| `server_manager interactive` | Opens the interactive console menu. |
| `server_manager status` | Displays system telemetry (CPU, RAM, Swap, Disk, GPU) and Docker daemon state. |
| `server_manager doctor` | Runs comprehensive non-destructive system diagnostics (kernel, Docker, firewall, ports). |
| `server_manager generate` | Generates the deterministic `docker-compose.yml` stack configuration. |
| `server_manager enable <service>` | Enables a service in the orchestrator configuration. |
| `server_manager disable <service>` | Disables a service in the orchestrator configuration. |
| `server_manager web --bind 127.0.0.1 --port 8099` | Launches the web administration interface. |
| `server_manager user add <name> --role Admin --quota 50` | Adds a new user account with storage quota in GB. |
| `server_manager user delete <name>` | Deletes a user account. |
| `server_manager user list` | Lists all existing user accounts and roles. |
| `server_manager user passwd <name>` | Changes a user's password securely. |

---

## 📦 Application Catalog & Port Matrix

| Category | Service | Access Port | Internal URL | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Infrastructure** | Nginx Proxy Manager | 80, 81, 443 | `http://IP:81` | Reverse Proxy & SSL Management |
| | Portainer | 9000 (Localhost) | `http://localhost:9000` | Docker Stack Management |
| | MariaDB | - | `mariadb` | Internal SQL Database |
| | Redis | - | `redis` | Internal Cache |
| | Netdata | 19999 (Localhost) | `http://localhost:19999` | Real-time System Monitoring |
| | Uptime Kuma | 3001 (Localhost) | `http://localhost:3001` | Service Health & Uptime Dashboard |
| | DNSCrypt Proxy | 5300 | `dnscrypt-proxy` | Secure DNS Over HTTPS |
| | WireGuard | 51820 (UDP) | - | Secure VPN Gateway |
| **Media Servers** | Plex | 32400 | `http://IP:32400` | Media Streaming Platform |
| | Jellyfin | 8096 | `http://IP:8096` | Open Source Media Server |
| | Tautulli | 8181 (Localhost) | `http://localhost:8181` | Plex Statistics & Analytics |
| | Overseerr | 5055 (Localhost) | `http://localhost:5055` | Media Request Management (Plex) |
| | Jellyseerr | 5056 (Localhost) | `http://localhost:5056` | Media Request Management (Jellyfin) |
| **Automation & Arr**| Sonarr | 8989 (Localhost) | `http://localhost:8989` | Smart TV Series Manager |
| | Radarr | 7878 (Localhost) | `http://localhost:7878` | Movie Manager |
| | Bazarr | 6767 (Localhost) | `http://localhost:6767` | Subtitle Management |
| | Prowlarr | 9696 (Localhost) | `http://localhost:9696` | Torrent & Usenet Indexer Sync |
| | Jackett | 9117 (Localhost) | `http://localhost:9117` | Indexer Proxy |
| **Download** | qBittorrent | 8080 (Localhost), 6881 | `http://localhost:8080` | BitTorrent Client |
| **Apps & Utility** | Nextcloud | 4443 (Localhost) | `https://localhost:4443` | Personal Cloud Storage |
| | Vaultwarden | 8001 (Localhost) | `http://localhost:8001` | Password Manager |
| | Filebrowser | 8002 (Localhost) | `http://localhost:8002` | Web File Manager |
| | Yourls | 8003 (Localhost) | `http://localhost:8003` | URL Shortener |
| | GLPI | 8088 (Localhost) | `http://localhost:8088` | Asset Management |
| | Gitea | 3000 (Localhost), 2222 | `http://localhost:3000` | Git Hosting |
| | Roundcube | 8090 (Localhost) | `http://localhost:8090` | Webmail Client |
| | Mailserver | 25, 143, 587, 993 | - | Complete Mail Server |
| | Syncthing | 8384 (Localhost), 22000 | `http://localhost:8384` | Continuous File Synchronization |

---

## 🧪 Testing and Verification

To execute full verification including formatting, clippy static analysis, unit/integration test suite, cargo deny license/dependency checks, and cargo audit vulnerability scans:

```sh
./verify.sh
```

Alternatively, run individual cargo subcommands within the `server_manager` directory:

```sh
cd server_manager
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

---

Built with ❤️ by the Server Manager Engineering Team.
