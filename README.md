# Cylae: The Ultimate Optimized Media Server Stack

> **Status:** `Stable` | **Version:** `3.0.0 (Clean Slate)` | **Code Quality:** `Strict`

## 1. The Protocol (Architecture)

Welcome to **Cylae**. This is not just a script; it's a hyper-optimized, modular, and intelligent deployment engine for self-hosted media and infrastructure services.

Built on the "Clean Slate" protocol, Cylae was architected to address the bloat and inefficiency common in other installation scripts. It adheres to a strict "Zero Tolerance" policy for technical debt.

### Core Principles
*   **Idempotency:** Run it once or a thousand times; the result is always a consistent, desired state.
*   **Modularity:** Services are isolated plugins in `src/cyl_manager/services/`. Adding a new service is as simple as subclassing `BaseService`.
*   **Hardware Intelligence:** The system does not blindly install. It *analyzes* the host first.

---

## 2. Dynamic Hardware Detection

Cylae implements a global **Dynamic Hardware Detection** engine located in `src/cyl_manager/core/system.py`. Before a single container is spawned, the system profiles your hardware into one of two tiers:

### 🔴 Low-Spec / VPS Profile
*   **Trigger:** RAM < 4GB OR CPU <= 2 Cores OR Swap < 1GB.
*   **Strategy:** "Survival Mode." The goal is stability over raw speed. Services are tuned to minimize memory footprint and prevent OOM (Out of Memory) kills.
*   **Concurrency:** **Serial Installation (1 worker).** Services are installed one by one to prevent CPU saturation during startup.

### 🟢 High-Performance Profile
*   **Trigger:** Resources exceeding the Low-Spec thresholds.
*   **Strategy:** "Maximum Velocity." The goal is performance and throughput.
*   **Concurrency:** **Parallel Installation (4 workers).** Multiple services deploy simultaneously.

---

## 3. Service Optimizations

The hardware profile triggers specific, universal optimizations across the stack:

### 📧 MailServer (Docker Mailserver)
*   **Low-Spec:**
    *   **ClamAV Disabled:** The memory-hungry antivirus engine is turned off.
    *   **SpamAssassin Disabled:** Heavy spam filtering is disabled to save CPU cycles.
    *   *Result:* Startup time reduced from ~3 minutes to ~10 seconds. Fixes the "Waiting for mailserver" hang.
*   **High-Spec:** Full protection suite enabled (ClamAV + SpamAssassin).

### 🎬 Plex Media Server
*   **Low-Spec:**
    *   **Transcoding:** Force disk-based transcoding.
    *   **Plugin Procs:** Limited to **2** concurrent processes.
*   **High-Spec:**
    *   **Transcoding:** Maps `/transcode` to `/tmp` (RAM) for ultra-fast seeking and zero SSD wear.
    *   **Plugin Procs:** Allowed up to **6** concurrent processes.

### 🏴‍☠️ Starr Apps (Sonarr, Radarr, etc.)
*   **Low-Spec:**
    *   **Garbage Collection:** Forces `.NET` **Workstation GC** (`DOTNET_GCServer=0`). This aggressively reclaims memory at the cost of slight CPU overhead, preventing the idle memory creep typical of .NET apps.
*   **High-Spec:** Uses default Server GC for maximum throughput.

---

## 4. Installation & Usage

### Prerequisites
*   **OS:** Debian 11/12 or Ubuntu 20.04/22.04 (LTS recommended).
*   **User:** Root (sudo).
*   **Ports:** Ensure ports 80, 443, and others are open in your cloud firewall.

### Quick Start
```bash
# Clone the repository
git clone https://github.com/your-repo/cylae.git
cd cylae

# Run the installer
sudo ./install.sh
```

### The Menu
Once installed, the `cyl-manager` CLI is available globally.
```bash
cyl-manager menu
```
Use the interactive menu to:
1.  **Install Services:** Select from Plex, Tautulli, Sonarr, Radarr, Overseerr, and more.
2.  **View Status:** Check health and URLs of running services.
3.  **Manage Settings:** Update domain, email, and passwords.

---

## 5. Technical Deep Dive

### Directory Structure
```
/opt/cylae/
├── .env                # Global configuration (Domain, Passwords)
├── data/               # Persistent data for all containers
│   ├── plex/
│   ├── sonarr/
│   └── ...
└── compose/            # Generated Docker Compose files
```

### Debugging
Logs are your friend.
```bash
# View Cylae Manager logs
cat /var/log/cyl-manager.log

# View specific service logs
docker logs -f plex
```

---
*Built with ❤️ and strict type-hinting by Jules.*
