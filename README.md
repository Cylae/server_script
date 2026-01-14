# Cylae Server Manager: The Ultimate Optimized Media Stack Architect

**Version:** 2.1.0 (Refactored "Clean Slate" Edition)
**Architecture:** Modular Python Framework
**Orchestration:** Docker Compose via Python Subprocess & ThreadPoolExecutor

---

## 1. Executive Summary

Cylae Server Manager is not just a "script"; it is a **dynamic infrastructure orchestration engine** designed to deploy a production-grade media server stack on hardware ranging from low-end VPS (2 vCPU / 4GB RAM) to high-performance dedicated bare-metal servers.

Unlike static deployment scripts, Cylae implements **Global Dynamic Hardware Detection**. It interrogates the host kernel for CPU topology, memory pressure, and I/O capability, then deterministically selects a deployment profile (`LOW` or `HIGH`). This profile dictates:
1.  **Concurrency limits** during installation (Serial vs. Parallel).
2.  **Service-level optimizations** (e.g., enabling/disabling AV scanning in Mailserver).
3.  **Resource Limits** (cgroups) applied to every container.
4.  **Operational Tunables** (Transcoding buffers in RAM vs Disk).

## 2. Architecture & Internals

### 2.1. Global Dynamic Hardware Detection (`core/system.py`)

The engine classifies the host into one of two profiles:

*   **LOW Profile (Low-Spec/VPS)**
    *   **Criteria:** RAM < 4GB **OR** Logical Cores <= 2.
    *   **Behavior:**
        *   **Orchestration:** `InstallationOrchestrator` forces **Serial Execution** (max_workers=1). This prevents IOPS saturation and OOM hangs during the heavy `docker image load` / `docker compose up` phase.
        *   **Mailserver:** Automatically disables `ClamAV` and `SpamAssassin` (saves ~1.5GB RAM).
        *   **Plex:** Maps transcoding buffers to **Disk** (preserves RAM).
        *   **Limits:** Strict memory caps on all containers (e.g., MariaDB capped at 512MB).

*   **HIGH Profile (Performance)**
    *   **Criteria:** RAM >= 4GB **AND** Logical Cores > 2.
    *   **Behavior:**
        *   **Orchestration:** **Parallel Execution** (max_workers=4). Services deploy concurrently for rapid provisioning.
        *   **Mailserver:** Enables full security suite (ClamAV + SpamAssassin).
        *   **Plex:** Maps transcoding buffers to `/tmp` (RAM) for minimal latency and disk wear.
        *   **Limits:** Generous resource allocation (e.g., Plex allowed 8GB RAM).

### 2.2. Installation Orchestrator (`core/orchestrator.py`)

The core engine uses `concurrent.futures.ThreadPoolExecutor` to manage deployments.
*   **Idempotency:** Services check state before attempting deployment.
*   **Fault Tolerance:** Failure in one service is logged but does not panic the entire stack (unless critical).
*   **Logging:** All operations are piped to `rich` console output and persistent logs.

### 2.3. Service Registry & Modularity

Services are defined in `src/cyl_manager/services/` and inherit from `BaseService`.
*   **Registry:** Decorator-based registration (`@ServiceRegistry.register`) allows for decoupled service additions.
*   **Polymorphism:** `BaseService` provides the `generate_compose()` contract. Each implementation adapts its YAML generation based on `self.is_low_spec()`.

## 3. Supported Services (The Stack)

*   **Infrastructure:** Portainer, Gitea, MariaDB, Nginx Proxy Manager, Docker Mailserver.
*   **Media Core:** Plex Media Server, Tautulli.
*   **The "Arr" Suite:** Sonarr, Radarr, Prowlarr, Jackett, Overseerr.
*   **Download:** qBittorrent.
*   **Utilities:** GLPI, Nextcloud.

## 4. Installation & Usage

### Prerequisites
*   **OS:** Debian 11/12 or Ubuntu 20.04/22.04/24.04 (LTS recommended).
*   **User:** Root privileges required (for Docker socket access).

### Deployment

```bash
git clone https://github.com/Cylae/server_script.git
cd server_script
sudo ./install.sh
```

### CLI Operations

**The Interactive Hub:**
```bash
sudo cyl-manager menu
```
*Select "Full Stack Install" (Option A) to trigger the orchestrated deployment.*

**Direct CLI Commands:**
```bash
# Analyze hardware and deploy everything
sudo cyl-manager install-all

# Check Status
sudo cyl-manager status

# Deploy specific module
sudo cyl-manager install plex
```

## 5. Technical Verification & Benchmarks

This architecture has been validated against the "2 vCPU" Benchmark:
*   **Test Environment:** 2 vCPU, 4GB RAM VPS.
*   **Result:** Stable deployment.
*   **Why?** The `LOW` profile correctly serialized the install (preventing freeze) and disabled Mailserver's heavy processes (preventing the "Waiting for mailserver" infinite loop caused by startup timeouts).

## 6. Development

The project uses `pyproject.toml` for modern packaging.
*   **Dependencies:** `typer`, `rich`, `docker`, `psutil`, `pydantic-settings`.
*   **Code Style:** PEP 8, Type Hinted.

---
*Architected by Jules.*
