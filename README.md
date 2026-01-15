# Cylae Server Manager 🚀

![Cylae Banner](https://img.shields.io/badge/Status-Stable-brightgreen?style=for-the-badge) ![Python](https://img.shields.io/badge/Python-3.12%2B-blue?style=for-the-badge&logo=python) ![Docker](https://img.shields.io/badge/Docker-Enabled-blue?style=for-the-badge&logo=docker) ![License](https://img.shields.io/badge/License-MIT-purple?style=for-the-badge)

> **The Ultimate Optimized Media Server Stack.**
> *Architected for Performance. Scaled for Stability. Secured by Design.*

---

## 🇬🇧 Technical Documentation & Architecture

### 1. Executive Summary
Cylae Server Manager is not just an installer; it is a **Python-based orchestration framework** designed to deploy a state-of-the-art self-hosted ecosystem. Unlike simple bash scripts, Cylae leverages strict typing (`Mypy`), modular architecture (`ServiceRegistry`), and object-oriented design to ensure idempotency and reliability.

It features **Global Dynamic Hardware Detection**, an intelligent profiling engine that analyzes host resources (CPU Instructions per Cycle capacity implied via Core count, Available RAM, Swap pressure) to dynamically tailor container configurations *before* they are instantiated.

### 2. Core Architecture
The system is built on a modular `src/` layout:

*   **`InstallationOrchestrator`**: The central brain. It utilizes `concurrent.futures.ThreadPoolExecutor` to manage deployment.
    *   **High-Spec Strategy**: Launches 4 parallel workers for rapid deployment.
    *   **Low-Spec Strategy**: Enforces strict serialization (1 worker) to prevent I/O saturation and CPU thrashing.
*   **`SystemManager`**: A static utility class acting as the Hardware Abstraction Layer (HAL). It interfaces with `psutil` to probe system metrics.
*   **`ServiceRegistry`**: A metaclass-driven registry that auto-discovers capabilities.
*   **`DockerManager`**: A thread-safe Singleton (utilizing `threading.Lock`) that manages the Docker Socket connection, ensuring race-free network creation and container health checks.

### 3. Global Dynamic Hardware Detection
The framework categorizes hosts into two distinct profiles based on strict resource thresholds defined in `cyl_manager.core.system`.

| Metric | Condition for **LOW** Profile (VPS/RPi) | **HIGH** Profile (Dedicated/Metal) |
| :--- | :--- | :--- |
| **RAM** | `< 4 GB` | `>= 4 GB` |
| **CPU Cores** | `<= 2 vCPUs` | `> 2 vCPUs` |
| **Swap** | `< 1 GB` | `>= 1 GB` |

#### Adaptive Service Optimization
Upon detection, the `BaseService` injects specific environment variables and `deploy.resources.limits` into the generated Docker Compose configurations.

*   **MailServer (`docker-mailserver`)**:
    *   **Low-Spec**: Automatically disables `ClamAV` (Anti-Virus) and `SpamAssassin`.
        *   *Why?* These processes are memory vampires. Disabling them prevents the infamous "Startup Hang" and "Waiting for mailserver" loops caused by OOM kills or timeouts on 2 vCPU instances.
    *   **Resource Limits**: Caps CPU at 1.0 (50% of 2 cores) and RAM at 1GB to ensure system responsiveness.
*   **Plex Media Server**:
    *   **Transcoding**:
        *   *High-Spec*: Maps `/tmp` (RAM) to `/transcode` for ultra-fast, disk-less transcoding.
        *   *Low-Spec*: Maps local disk, preserving RAM for the OS.
*   **Starr Apps (Sonarr/Radarr)**:
    *   **Optimization**: Injects `COMPlus_EnableDiagnostics=0` to disable .NET Core debugging overhead.
    *   **Limits**: Dynamically scales heap size limits.

### 4. Security & Networking
*   **Firewall Automation**: The `FirewallManager` interacts directly with `ufw`. It parses the `get_ports()` method of each service and opens *only* the necessary ports just-in-time during installation.
*   **User Mapping**: `PUID` and `PGID` are auto-detected from the `SUDO_UID` environment variable, ensuring that files on the host are owned by your actual user, not `root`.
*   **Zero-Trust Networking**: Services communicate over a dedicated bridge network (`cyl_network`), isolated from other potential containers.

### 5. Installation & Usage

**System Requirements:** Debian 11/12 or Ubuntu 20.04/22.04 LTS.

```bash
# Clone and Install (Root required for Docker/UFW checks)
sudo ./install.py
```

**CLI Commands:**
```bash
cyl-manager menu         # Interactive TUI (Text User Interface)
cyl-manager status       # Real-time container health & URL map
cyl-manager install plex # Targeted deployment
```

---

## 🇫🇷 Documentation Technique & Architecture

### 1. Résumé Exécutif
Cylae Server Manager est un **framework d'orchestration Python** conçu pour déployer un écosystème auto-hébergé de pointe. Contrairement aux scripts bash simples, Cylae exploite un typage strict, une architecture modulaire et une conception orientée objet pour garantir fiabilité et idempotence.

Il intègre une **Détection Matérielle Dynamique Globale**, un moteur de profilage intelligent qui analyse les ressources de l'hôte pour adapter dynamiquement les configurations des conteneurs *avant* leur instanciation.

### 2. Architecture Centrale
Le système repose sur une structure modulaire :

*   **`InstallationOrchestrator`** : Le cerveau central. Utilise `ThreadPoolExecutor` pour gérer le déploiement.
    *   *Stratégie Haute-Perf* : 4 workers parallèles.
    *   *Stratégie Basse-Conso* : Sérialisation stricte (1 worker) pour éviter la saturation E/S.
*   **`DockerManager`** : Un Singleton "Thread-Safe" gérant la connexion au Docker Socket.

### 3. Détection Matérielle Dynamique
Le framework classe les hôtes en deux profils distincts :

| Métrique | Profil **LOW** (VPS/RPi) | Profil **HIGH** (Dédié) |
| :--- | :--- | :--- |
| **RAM** | `< 4 Go` | `>= 4 Go` |
| **CPU Cores** | `<= 2 vCPUs` | `> 2 vCPUs` |

#### Optimisation Adaptative des Services
*   **MailServer** :
    *   *Low-Spec* : Désactive automatiquement `ClamAV` et `SpamAssassin` pour prévenir les boucles de démarrage infinies sur les petites instances (2 vCPUs).
*   **Plex Media Server** :
    *   *High-Spec* : Transcodage en RAM (`/tmp`).
    *   *Low-Spec* : Transcodage sur disque.

### 4. Installation

```bash
sudo ./install.py
```

---

<p align="center">
  Engineering Excellence. No Compromises.
</p>
