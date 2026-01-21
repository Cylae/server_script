# Cylae Server Manager: The Ultimate Optimized Media Server Stack

[![Stability Verified: CLI & Web Interface Tested](https://img.shields.io/badge/Stability-Verified-green)](https://github.com/Cylae/server_script)
[![Architecture: Clean Slate](https://img.shields.io/badge/Architecture-Clean_Slate-blue)](https://github.com/Cylae/server_script)
[![Optimization: GDHD Enabled](https://img.shields.io/badge/Optimization-GDHD_Active-orange)](https://github.com/Cylae/server_script)

**Cylae Server Manager** is a high-performance, modular DevOps framework designed to deploy, orchestrate, and optimize a self-hosted media ecosystem (Plex, Starr Apps, Mailserver, etc.). Built with a "Clean Slate" philosophy, it abandons legacy bloat for a pristine, type-safe Python 3 architecture.

At its core lies the **Global Dynamic Hardware Detection (GDHD)** engine, an algorithmic heuristic system that analyzes the host environment in real-time to tailor every service configuration for maximum stability and performance—whether running on a $5/mo 1GB VPS or a 64-core dedicated server.

## 🚀 Key Features

### 1. Global Dynamic Hardware Detection (GDHD)
The system does not blindly install containers. It **analyzes** the substrate first.

*   **Heuristic Analysis:** Scans CPU Cores, Total RAM, and Swap Space.
*   **Profile Enforcement:**
    *   **LOW (Survival Mode):** Triggered if RAM < 4GB, CPU <= 2 Cores, or Swap < 1GB.
    *   **HIGH (Performance Mode):** Triggered on robust hardware.
*   **Universal Optimization:** These profiles cascade down to every single service configuration and the operating system kernel.

#### Deep Dive: Hardware Profiles
| Metric | "LOW" Threshold | "HIGH" Threshold |
| :--- | :--- | :--- |
| **RAM** | < 4 GB | >= 4 GB |
| **vCPU** | <= 2 Cores | > 2 Cores |
| **Swap** | < 1 GB | >= 1 GB |

*If **ANY** of the "LOW" criteria are met, the system engages **Survival Mode** to prevent OOM kills and CPU saturation.*

### 2. Service-Specific Optimizations
The framework applies granular tuning based on the GDHD profile:

#### 📬 Mailserver (Docker Mailserver)
*   **Problem:** ClamAV and SpamAssassin require ~1.5GB RAM alone. On a 1GB VPS, this causes an infinite startup hang (OOM loop).
*   **Solution (GDHD Low):** Automatically injects `ENABLE_CLAMAV=0` and `ENABLE_SPAMASSASSIN=0`. Disables `Fail2Ban` to save python overhead.
*   **Solution (GDHD High):** Enables full security suite.

#### 🎥 Plex Media Server
*   **Problem:** Transcoding to disk wears out SD cards/SSDs. Transcoding to RAM is faster but risky on low RAM.
*   **Solution (GDHD Low):** Transcodes to disk (`/config/transcode`). Limits `PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS=2`.
*   **Solution (GDHD High):** Mounts `/tmp` to `/transcode` for **Zero-Copy RAM Transcoding**. Allows 6 plugin procs.

#### 🌟 Starr Apps (Sonarr, Radarr, etc.)
*   **Problem:** .NET Core garbage collection (GC) defaults to "Server GC", which is aggressive on memory allocation.
*   **Solution (Universal):** Sets `COMPlus_EnableDiagnostics=0` to reduce runtime overhead.
*   **Solution (GDHD Low):** Forces **Workstation GC** (`COMPlus_GCServer=0`). This drastically reduces the memory footprint (often by 50%+) at the cost of slightly lower peak throughput.

#### 🐧 Kernel-Level Tuning (Sysctl)
*   **Universal:** Hardens network stack (disables redirects). Enables TCP Fast Open (`net.ipv4.tcp_fastopen=3`).
*   **GDHD High:**
    *   Enables **BBR Congestion Control** (`net.ipv4.tcp_congestion_control=bbr`) for max throughput.
    *   Increases file descriptors (`fs.file-max=2097152`) and connection limits (`somaxconn=65535`).
    *   Sets `vm.swappiness=10` to prefer RAM.
*   **GDHD Low:**
    *   Sets `vm.swappiness=20` to balance RAM usage without thrashing.
    *   Increases `vm.vfs_cache_pressure=50` to retain filesystem cache longer for performance.

### 3. Architecture & Security
*   **Clean Slate Protocol:** Pure Python implementation using `pydantic`, `typer`, and `docker` SDK. No legacy bash scripts.
*   **Network Isolation:** All services communicate over an internal `cylae_net` Docker bridge.
*   **Firewall Automation:** `ufw` rules are injected dynamically only for exposed ports.
*   **Least Privilege:** Services like Portainer run with `no-new-privileges:true`.

## 🛠️ Installation

### Prerequisites
*   OS: Debian 11+ / Ubuntu 20.04+ (LTS recommended)
*   Python: 3.9+
*   Root privileges

### Quick Start
```bash
# 1. Clone the repository
git clone https://github.com/Cylae/server_script.git
cd server_script

# 2. Run the Bootstrap Installer
# This script compiles dependencies, sets up the virtualenv, and installs the CLI.
sudo python3 install.py
```

## 💻 Usage

### The CLI (`cyl-manager`)
The `cyl-manager` command is your primary interface.

*   **Interactive Dashboard:**
    ```bash
    cyl-manager menu
    ```
    *Visualizes installed services, URLs, and allows one-click installs.*

*   **Full Stack Deployment:**
    ```bash
    cyl-manager install_all
    ```
    *Deploys the entire suite. GDHD will determine if this happens in parallel or serially.*

*   **System Status:**
    ```bash
    cyl-manager status
    ```
    *Shows real-time container health and hardware profile.*

### The Web Dashboard
A lightweight Flask application for monitoring from your browser.

```bash
sudo python3 server.py
# Access at http://<your-ip>:5000
```

## 🏗️ Technical Architecture

The project is structured as a Python package (`src/cyl_manager`).

### Directory Layout
*   `core/`: The brain.
    *   `hardware.py`: GDHD algorithm implementation.
    *   `optimization.py`: Kernel sysctl tuner.
    *   `orchestrator.py`: Manages parallel/serial execution.
*   `services/`: The limbs.
    *   Each service (e.g., `PlexService`) inherits from `BaseService`.
    *   Implements `generate_compose()` to return dynamic configuration.
*   `ui/`: The face. Rich-based TUI components.
*   `web/`: The remote eye. Flask web dashboard.

### Adding a New Service
To add a service, create a class in `services/` and register it:

```python
@ServiceRegistry.register
class MyNewService(BaseService):
    name = "myservice"
    def generate_compose(self):
        return { ... } # Return Docker Compose dict
```

## 🧪 Testing & Verification

We adhere to a strict **Testing & Validation Loop**.

*   **Unit Tests:** Coverage for all core logic.
    ```bash
    pytest
    ```
*   **Architecture Compliance:** `tests/test_architecture_compliance.py` verifies that GDHD profiles are correctly enforced.
*   **Optimization Verification:** `tests/test_ultimate_optimizations.py` validates that specific env vars (like GC tuning) are injected.
*   **Kernel Verification:** `tests/test_kernel_optimization.py` ensures sysctl rules are generated correctly.

---
*Built with precision.*
