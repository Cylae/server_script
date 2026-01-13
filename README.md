# Cylae Server Manager: Architect Edition (v11.0)

> **"The Ultimate Optimized Media Server Stack"**

This is not just a script; it is a **cybernetic organism** designed to assimilate your hardware and deploy a perfect, optimized, self-hosted infrastructure.

## 🚀 The "Clean Slate" Architecture

We have rebuilt the core. The legacy code was purged. In its place, we implemented a **Dynamic Hardware Detection Matrix**.

### 🧠 Neuro-Adaptive Installation (Hardware Profiling)
The system no longer blindly installs services. It *thinks*.
Before a single container is spawned, the `HardwareManager` scans your neural pathways (CPU Cores, RAM, Swap, Disk Space).

*   **LOW_SPEC Profile** (<= 2 Cores or < 4GB RAM):
    *   **Protocol:** "Survival Mode".
    *   **Concurrency:** **Serialized**. Services are installed one by one to prevent the host from seizing up.
    *   **MailServer:** ClamAV and SpamAssassin are **lobotomized** (disabled) to save ~2GB of RAM. The initialization loop is patient, waiting for the slow boot.
    *   **Media Stack:** Plex, Sonarr, Radarr are given strict memory rations (e.g., Plex is capped at 1GB, *Arr apps at 256MB/512MB).
    *   **Cloud & Misc:** Nextcloud, Gitea, and all other services have tighter memory limits (128M-512M) to fit within 2GB RAM.
    *   **Safety:** Installation blocks if disk space is critical (<5GB).

*   **HIGH_PERFORMANCE Profile**:
    *   **Protocol:** "God Mode".
    *   **Concurrency:** **Parallelized**. We spin up multiple installation threads (Max Workers: 3) to deploy the stack in record time.
    *   **Resource Limits:** The floodgates are opened. Plex gets 4GB+, and services run with full feature sets enabled.

## 🛠 Features & Capabilities

### The Media Stack (Unified)
A fully integrated, automated media consumption engine.
*   **Plex**: The core. Optimized transcoding buffers.
*   **The *Arr Suite**: Sonarr (TV), Radarr (Movies), Prowlarr (Indexers), Jackett (Legacy Indexers), Readarr (Books - *coming soon*).
*   **Overseerr**: The request management frontend.
*   **Tautulli**: Monitoring and analytics for Plex.
*   **qBittorrent**: The downloader, bound to the stack.

### The Core Services
*   **MailServer**: Full stack (Postfix/Dovecot) with DKIM/DMARC. *Smart-disabled antivirus on low-end hardware.*
*   **Nextcloud**: Your cloud.
*   **Vaultwarden**: Your passwords.
*   **WireGuard**: Your tunnel.
*   **Uptime Kuma**: The heartbeat monitor.
*   **Gitea**: Self-hosted git service.
*   **Portainer**: Docker management.
*   **GLPI**: IT Asset Management.

## 💻 Usage

### 0. Prerequisites
*   OS: Debian 11/12 or Ubuntu 20.04/22.04 LTS.
*   Root access.
*   A valid domain name pointing to this server.

### 1. Installation
Clone the repository and run the bootstrap script. It will handle dependencies (Python, Docker, Venv).

```bash
git clone https://github.com/Cylae/server_script.git cylae-manager
cd cylae-manager
chmod +x install.sh
./install.sh
```

### 2. The Interface
Launch the manager globally:
```bash
cyl-manager
```

Navigate to **Option 13 (Manage Media Stack)** and select **99. Install ALL Media Services**.
*   Sit back. The script will detect your hardware and choose the optimal deployment strategy automatically.

## 🔧 Technical Details (Under the Hood)

*   **Language**: Python 3.10+
*   **Containerization**: Docker Compose V2 (Dynamic Generation).
*   **Proxy**: Nginx (Auto-configured with Let's Encrypt SSL).
*   **Database**: MariaDB (Optimized).

### Memory Management Strategy
We use `psutil` to query the kernel.
*   **Code Reference**: `cyl_manager.core.system.determine_profile()`
*   **Service Injection**: `BaseService` injects `self.profile` into every service class.
*   **Docker Limits**: We utilize `deploy.resources.limits.memory` in generated Compose files to enforce the profile's will.

## ⚠️ "The 2 vCPU Benchmark"
This system is certified to deploy on a **2 vCPU / 4GB RAM** VPS without crashing.
*   *MailServer hanging fix*: Verified. ClamAV is terminated on sight in low-spec environments.
*   *Installation freeze fix*: Verified. Serial execution prevents IOwait spikes.

---
*Built for efficiency. Designed for power.*
