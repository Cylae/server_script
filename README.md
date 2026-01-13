# Cylae Server Manager: The Ultimate Optimized Media Stack

> **Status:** Production Ready (v2.1.0)
> **Architecture:** Modular Python (Typer/Rich) + Docker Compose
> **Optimization Strategy:** Dynamic Hardware Profiling (DHP)

## 1. Overview

Cylae Server Manager is not just an installation script; it is a **hardware-aware deployment orchestrator**. Designed for stability across the spectrum of compute power—from low-end VPS instances (1 vCPU/2GB RAM) to high-performance bare metal servers—it intelligently adapts the entire stack configuration to the host environment.

Gone are the days of manually tweaking `docker-compose.yml` files or suffering from system lockups because ClamAV decided to eat 2GB of RAM on a 1GB instance. Cylae automates this tuning at the architectural level.

## 2. Dynamic Hardware Profiling (DHP)

The core innovation of this stack is the **DHP Protocol**. Upon initialization, the system analyzes the host hardware metrics to assign a performance profile.

### The Algorithm

The profiling logic is implemented in `cyl_manager.core.system`. It queries the kernel via `psutil` to determine:
*   **Physical Cores:** Logical CPU count.
*   **Total Memory:** Available RAM.
*   **Swap Space:** Virtual memory availability (critical for low-RAM stability).

#### Profile Definitions

| Profile | Criteria | Target Hardware | Deployment Strategy |
| :--- | :--- | :--- | :--- |
| **LOW** | RAM < 4GB **OR** CPU <= 2 Cores | VPS, Raspberry Pi, NUCs | **Serialized & Lean.** Services deploy one-by-one. Heavy features (ClamAV, Transcoding) are disabled or throttled. |
| **HIGH** | RAM >= 4GB **AND** CPU > 2 Cores | Dedicated Servers, Home Labs | **Parallel & Full.** Services deploy concurrently. Full feature sets enabled. |

## 3. Service Optimization Matrix

Each service class implements specific logic to adapt to the detected profile.

### MailServer (Docker Mailserver)
*   **Problem:** ClamAV and SpamAssassin are notoriously resource-heavy, often causing infinite hangs during startup on < 4GB RAM systems.
*   **LOW Profile:**
    *   `ENABLE_CLAMAV=0`
    *   `ENABLE_SPAMASSASSIN=0`
    *   Healthcheck `start_period` extended to 2m.
    *   CPU Limit: 0.5 Cores | RAM Limit: 768MB.
*   **HIGH Profile:** Full Anti-Virus and Anti-Spam protection enabled.

### Plex Media Server
*   **Problem:** Transcoding without limits can starve the host OS of resources.
*   **LOW Profile:**
    *   CPU Limit: 0.75 Cores (Prevents 100% CPU usage lockups).
    *   RAM Limit: 1GB.
*   **HIGH Profile:**
    *   CPU Limit: 2.0 Cores | RAM Limit: 4GB.

### The *Arr Stack (Sonarr, Radarr, etc.)
*   **Optimization:** .NET Core Runtime tuning.
*   **LOW Profile:**
    *   `COMPlus_EnableDiagnostics=0`: Disables the debugging sidecar to save memory cycles.
    *   Aggressive memory limits (384MB per container).

## 4. Architecture & Installation Flow

The system follows a strict orchestration pattern ensuring dependency resolution and system stability.

1.  **System Audit:** Check OS (Debian/Ubuntu), Root privileges, and Disk Space.
2.  **Profile Assignment:** DHP assigns `LOW` or `HIGH`.
3.  **Infrastructure Phase:** Core services (MariaDB, Nginx Proxy) are **always installed first** to ensure database availability.
4.  **Application Phase:**
    *   **LOW:** Sequential installation. `Install A -> Wait for Health -> Install B`.
    *   **HIGH:** Threaded parallel installation (up to 4 workers).

### CLI Usage

```bash
# Install specific service
cyl-manager install plex

# Install everything (Auto-optimized)
cyl-manager menu
# Select "A" for Full Stack Installation
```

## 5. Directory Structure

Data persistence is critical. All configurations are strictly separated from the codebase.

*   `src/cyl_manager`: Source code.
*   `/opt/cylae/data`: Persistent application data (bind mounts).
*   `/opt/cylae/compose`: Generated Docker Compose files (for manual inspection/debugging).
*   `/opt/media`: Unified media storage root.

---
*Built with logic, optimized for reality.*
