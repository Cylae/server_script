# Cylae Server Manager: The Ultimate Optimized Media Stack

> **Engineering the Perfect Home Server Infrastructure.**
> *Ingénierie de l'Infrastructure Serveur Domestique Parfaite.*

[![Python](https://img.shields.io/badge/Python-3.9%2B-blue?style=for-the-badge&logo=python)](https://www.python.org/)
[![Docker](https://img.shields.io/badge/Docker-v24%2B-2496ED?style=for-the-badge&logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

---

## 🇬🇧 Technical Documentation (English)

### 1. Abstract

**Cylae Server Manager** is not a mere collection of shell scripts; it is a sophisticated **Infrastructure-as-Code (IaC)** orchestration engine written in Python. It is engineered to solve the "Day 0" problem of media server deployment: the discrepancy between hardware capabilities and software configuration.

By implementing the **"Clean Slate" Protocol**, this system ensures a pristine, idempotent deployment state, eliminating technical debt before it even begins. It leverages a custom-built **Global Dynamic Hardware Detection (GDHD)** algorithm to analyze host telemetry and apply profile-specific optimizations at runtime.

### 2. Global Dynamic Hardware Detection (GDHD)

The core differentiator of this architecture is its ability to "sense" the host environment. Before a single container is spawned, the `SystemManager` module performs a deep interrogation of the kernel resources.

#### The Heuristics
The system calculates a hardware profile based on the following strict thresholds:

| Resource | "Survival Mode" Threshold (LOW) | Implementation Detail |
| :--- | :--- | :--- |
| **CPU** | `<= 2 vCPUs` | Critical for VPS instances where context switching kills performance. |
| **RAM** | `< 4 GB` | The minimum baseline for a full Java/Python/Mono stack. |
| **Swap** | `< 1 GB` | Essential spillover protection for OOM (Out Of Memory) killers. |

If **ANY** of these conditions are met, the system enforces the **LOW** profile. Otherwise, it defaults to **HIGH**.

### 3. Profile-Specific Optimizations

The orchestration engine applies granular configuration injections based on the detected profile.

#### A. The "Low-Spec" Protocol (VPS / Legacy Hardware)
*Designed for stability on constrained resources (e.g., typical $5/mo VPS).*

1.  **Mailserver Heuristic Optimization:**
    *   **Logic:** The `docker-mailserver` stack is notoriously heavy due to `clamd` (ClamAV) and `amavis`.
    *   **Action:** On LOW profile, the Orchestrator injects `ENABLE_CLAMAV=0` and `ENABLE_SPAMASSASSIN=0` into the environment variables.
    *   **Result:** Prevents the "Infinite Wait Loop" where the healthcheck fails because the service times out swapping memory during boot. Saves ~1.5GB RAM.

2.  **Plex Transcoding IO Redirection:**
    *   **Logic:** Transcoding to RAM (`/dev/shm`) is ideal but fatal on low-RAM systems.
    *   **Action:** The volume mapping dynamically shifts from `/tmp` (RAM) to `$DATA_DIR/plex/transcode` (Disk).
    *   **Result:** Eliminates OOM crashes during playback, trading I/O latency for stability.

3.  **Serialized Concurrency Control:**
    *   **Logic:** Parallel image extraction saturates I/O on shared VPS storage.
    *   **Action:** `InstallationOrchestrator` forces a `max_workers=1` thread pool.
    *   **Result:** Deterministic, sequential installation that never freezes the host.

#### B. The "High-Performance" Protocol (Dedicated / Bare Metal)
*Designed for maximum throughput and responsiveness.*

1.  **RAM-Based Transcoding:** Plex is mapped to `/tmp` for zero-latency seeking and reduced SSD wear leveling.
2.  **Full Security Suite:** Mailserver runs with full ClamAV/SpamAssassin/Fail2Ban heuristic analysis.
3.  **Parallel Deployment:** The Orchestrator spins up 4+ concurrent workers, utilizing multi-core architectures to deploy the full stack in under 2 minutes.

### 4. The Stack Architecture

The application manages a tightly integrated microservices mesh via Docker Compose.

*   **Media Core:** Plex (Media Server), Tautulli (Telemetry).
*   **Automation (*Arr):** Sonarr (TV), Radarr (Movies), Prowlarr (Indexers).
*   **Optimization:** All .NET Core apps (Sonarr/Radarr) run with `COMPlus_EnableDiagnostics=0` to reduce runtime overhead.
*   **Infrastructure:** Docker Mailserver, Nginx Proxy Manager, Portainer, MariaDB, DNSCrypt Proxy.
*   **Networking:** Host networking for Plex, internal bridge `cylae_net` for secure inter-container communication.

### 5. Deployment Instructions

**Prerequisites:**
*   **OS:** Debian 11/12 (Bookworm) or Ubuntu 20.04/22.04 LTS.
*   **Privileges:** Root access (`sudo -i`).

**Bootstrapping:**

```bash
# Clone the repository
git clone https://github.com/YourRepo/cyl-manager.git /opt/cyl-manager

# Enter directory
cd /opt/cyl-manager

# Execute the Bootstrap Protocol
# This installs dependencies, sets up the virtual environment, and launches the CLI.
sudo python3 install.py
```

**Operation:**

Once installed, the `cyl-manager` command is available globally.

```bash
sudo cyl-manager menu
```

*Select "Full Stack Install" to trigger the GDHD analysis and deployment.*

---

## 🇫🇷 Documentation Technique (Français)

### 1. Résumé

**Cylae Server Manager** n'est pas une simple collection de scripts bash ; c'est un moteur d'orchestration **Infrastructure-as-Code (IaC)** sophistiqué écrit en Python. Il est conçu pour résoudre le problème du "Jour 0" : l'écart entre les capacités matérielles et la configuration logicielle.

En implémentant le **Protocole "Clean Slate"**, ce système garantit un état de déploiement vierge et idempotent, éliminant la dette technique avant même qu'elle ne commence. Il exploite un algorithme de **Détection Matérielle Dynamique Globale (GDHD)** pour analyser la télémétrie de l'hôte et appliquer des optimisations spécifiques au profil lors de l'exécution.

### 2. Détection Matérielle Dynamique (GDHD)

L'élément différenciateur clé de cette architecture est sa capacité à "sentir" l'environnement hôte. Avant qu'un seul conteneur ne soit lancé, le module `SystemManager` effectue une interrogation profonde des ressources du noyau.

#### Les Heuristiques
Le système calcule un profil matériel basé sur les seuils stricts suivants :

| Ressource | Seuil "Mode Survie" (LOW) | Détail d'Implémentation |
| :--- | :--- | :--- |
| **CPU** | `<= 2 vCPUs` | Critique pour les VPS où le changement de contexte tue les performances. |
| **RAM** | `< 4 GB` | Le minimum vital pour une stack complète Java/Python/Mono. |
| **Swap** | `< 1 GB` | Protection essentielle contre les tueurs OOM (Out Of Memory). |

Si **UNE SEULE** de ces conditions est remplie, le système force le profil **LOW**. Sinon, il passe en **HIGH**.

### 3. Optimisations Spécifiques au Profil

Le moteur d'orchestration injecte des configurations granulaires basées sur le profil détecté.

#### A. Le Protocole "Low-Spec" (VPS / Matériel Ancien)
*Conçu pour la stabilité sur des ressources contraintes.*

1.  **Optimisation Heuristique Mailserver :**
    *   **Logique :** La stack `docker-mailserver` est notoirement lourde à cause de `clamd` (ClamAV).
    *   **Action :** En profil LOW, l'Orchestrateur injecte `ENABLE_CLAMAV=0` et `ENABLE_SPAMASSASSIN=0`.
    *   **Résultat :** Empêche la "Boucle d'Attente Infinie" lors du démarrage. Économise ~1.5Go de RAM.

2.  **Redirection IO Transcodage Plex :**
    *   **Logique :** Le transcodage en RAM (`/dev/shm`) est idéal mais fatal sur les systèmes à faible RAM.
    *   **Action :** Le mapping de volume bascule dynamiquement de `/tmp` (RAM) vers `$DATA_DIR/plex/transcode` (Disque).
    *   **Résultat :** Élimine les crashs OOM pendant la lecture.

3.  **Contrôle de Concurrence Sérialisé :**
    *   **Logique :** L'extraction d'images en parallèle sature les I/O sur les VPS.
    *   **Action :** `InstallationOrchestrator` force un pool de threads `max_workers=1`.
    *   **Résultat :** Installation déterministe et séquentielle.

#### B. Le Protocole "High-Performance" (Dédié / Bare Metal)
*Conçu pour le débit maximal.*

1.  **Transcodage RAM :** Plex est mappé sur `/tmp` pour une latence nulle.
2.  **Suite de Sécurité Complète :** Mailserver tourne avec l'analyse heuristique complète.
3.  **Déploiement Parallèle :** L'Orchestrateur lance 4+ workers simultanés.

### 4. Instructions de Déploiement

**Prérequis :**
*   **OS :** Debian 11/12 ou Ubuntu 20.04/22.04 LTS.
*   **Privilèges :** Root (`sudo -i`).

**Amorçage (Bootstrapping) :**

```bash
git clone https://github.com/YourRepo/cyl-manager.git /opt/cyl-manager
cd /opt/cyl-manager
sudo python3 install.py
```

**Utilisation :**

```bash
sudo cyl-manager menu
```

---

<p align="center">
  Architected with 🧠 by Cylae.
  <br>
  <em>Code is Law. Efficiency is Mandatory.</em>
</p>
