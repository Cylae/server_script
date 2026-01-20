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
*   **Universal Optimization:** These profiles cascade down to every single service configuration.

### 2. Service-Specific Optimizations
The framework applies granular tuning based on the GDHD profile:

*   **Mailserver (Docker Mailserver):**
    *   *Low Profile:* Automatically disables memory-heavy processes like **ClamAV**, **SpamAssassin**, and **Fail2Ban** to prevent the infamous "Infinite Startup Hang" caused by OOM kills.
    *   *High Profile:* Enables full security suite for maximum protection.
*   **Plex Media Server:**
    *   *Low Profile:* Transcodes to disk to preserve RAM; limits database plugin processes to 2.
    *   *High Profile:* Mounts `/tmp` (RAM) for transcoding (Zero-Copy Latency); allows up to 6 database plugin processes.
*   **Starr Apps (Sonarr/Radarr/etc.):**
    *   *Universal:* Disables .NET Diagnostics (`COMPlus_EnableDiagnostics=0`) to reduce runtime overhead.
    *   *Low Profile:* Forces **Workstation GC** (`COMPlus_GCServer=0`) instead of Server GC to drastically reduce memory footprint.
*   **Orchestration & Concurrency:**
    *   *Low Profile:* Serializes installations (1 worker) to prevent I/O saturation and CPU lockup.
    *   *High Profile:* Parallelizes deployments (4 workers) for rapid stack bring-up.

### 3. Architecture & Security
*   **Clean Slate Protocol:** Pure Python implementation. No legacy bash scripts.
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
*   `core/`: The brain. Contains `HardwareManager` (GDHD logic), `DockerManager`, and `FirewallManager`.
*   `services/`: The limbs. Each service (e.g., `PlexService`) inherits from `BaseService` and implements `generate_compose()`.
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

---
*Built with precision.*
