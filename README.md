# Cylae Server Manager 🚀

> **The Ultimate Optimized Media Server Stack.**
> *L'Ultime Stack Media Server Optimisée.*

[![Python](https://img.shields.io/badge/Python-3.9%2B-blue?style=for-the-badge&logo=python)](https://www.python.org/)
[![Docker](https://img.shields.io/badge/Docker-v24%2B-2496ED?style=for-the-badge&logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

---

## 🇬🇧 English Documentation

### ⚡ Overview

**Cylae Server Manager** is a next-generation, intelligent infrastructure-as-code solution designed to deploy a battle-hardened media server stack on **Debian** and **Ubuntu** systems.

Unlike dumb bash scripts, this is a fully modular **Python application** that performs **Deep System Analysis** before deployment. It dynamically adapts every service configuration to match your hardware reality—whether you're running on a potato VPS or a Threadripper beast.

### 🧠 Intelligent Architecture (The "Nerdy" Stuff)

This isn't just `docker-compose up`. We implemented a **Global Dynamic Hardware Detection** engine that classifies your host into profiles (`LOW` vs `HIGH`).

#### 1. The "Low-Spec" Protocol (< 4GB RAM or <= 2 Cores)
If your system is detected as resource-constrained (e.g., a cheap $5 VPS), the system engages **survival mode**:
*   **Mailserver:** Automatically kills memory hogs like **ClamAV** and **Amavis**, preventing the dreaded "Infinite Start Loop" caused by OOM kills. We also tune `fail2ban` and reduce process limits.
*   **Plex/Jellyfin:** Forces transcoding buffers to **Disk** instead of **RAM** to save precious memory. Reduces database cache size.
*   **Concurrency:** The Orchestrator switches to **Serial Mode** (Concurrency = 1). Services are installed one-by-one to prevent system lockups during image extraction.

#### 2. The "High-Performance" Protocol
If you have the juice, we use it:
*   **Parallel Deployment:** Spins up 4+ installers simultaneously for lightning-fast setup.
*   **In-Memory Transcoding:** Plex is configured to transcode directly in `/tmp` (RAM) for zero-latency seeking and reduced SSD wear.
*   **Full Security Suite:** Enables all mail security features (ClamAV, SpamAssassin, Postgrey) for maximum protection.

### 📦 The Stack

| Service | Category | Function |
| :--- | :--- | :--- |
| **Plex** | Media | The King of Media Servers. |
| **Tautulli** | Monitoring | Analytics and monitoring for Plex. |
| **Sonarr** | Automation | TV Show PVR. |
| **Radarr** | Automation | Movie PVR. |
| **Jackett/Prowlarr** | Indexer | Indexer manager for Torrents/Usenet. |
| **Overseerr** | Request | Beautiful media request management. |
| **qBittorrent** | Download | Lightweight, robust torrent client. |
| **Docker Mailserver** | Infrastructure | Full-stack email server (Postfix, Dovecot). |
| **Nginx Proxy Manager** | Infrastructure | Reverse proxy with auto-SSL (Let's Encrypt). |
| **Portainer** | Management | GUI for Docker container management. |
| **MariaDB** | Database | Centralized database backend. |

### 🛠️ Installation

**Requirements:**
*   OS: Debian 11/12 or Ubuntu 20.04+ (LTS recommended)
*   User: Root (or sudo)
*   Git & Python 3 installed

**One-Liner (The Easy Way):**

```bash
git clone https://github.com/YourRepo/cyl-manager.git /opt/cyl-manager
cd /opt/cyl-manager
sudo python3 install.py
```

After installation, the `cyl-manager` command is available globally.

### 🚀 Usage

Launch the interactive CLI:

```bash
sudo cyl-manager menu
```

*   **A - Full Stack Install:** The magic button. Deploys everything based on your profile.
*   **Service Management:** Install/Uninstall specific services.
*   **Configuration:** Change domain, email, etc.
*   **Service Credentials:** View generated passwords and URLs.

---

## 🇫🇷 Documentation Française

### ⚡ Vue d'Ensemble

**Cylae Server Manager** est une solution "Infrastructure-as-Code" de nouvelle génération, conçue pour déployer une stack média serveur robuste sur **Debian** et **Ubuntu**.

Contrairement aux scripts bash basiques, il s'agit d'une **application Python modulaire** qui effectue une **Analyse Système Profonde** avant le déploiement. Elle adapte dynamiquement la configuration de chaque service à votre matériel réel.

### 🧠 Architecture Intelligente (Le Côté Tech)

Nous avons implémenté un moteur de **Détection Matérielle Dynamique** qui classifie votre hôte (`LOW` vs `HIGH`).

#### 1. Protocole "Low-Spec" (< 4GB RAM ou <= 2 Coeurs)
Si votre système est limité (ex: VPS à 5€), le système active le **mode survie** :
*   **Mailserver :** Désactive automatiquement **ClamAV** et **Amavis** pour éviter les boucles de démarrage infinies causées par le manque de RAM.
*   **Plex :** Force le transcodage sur le **Disque** plutôt que la **RAM**.
*   **Concurrence :** L'Orchestrateur passe en **Mode Série** (Concurrence = 1). Les services sont installés un par un pour ne pas figer le système.

#### 2. Protocole "High-Performance"
Si vous avez la puissance, nous l'utilisons :
*   **Déploiement Parallèle :** Lance 4+ installateurs simultanément.
*   **Transcodage en RAM :** Plex utilise `/tmp` pour une latence nulle.
*   **Sécurité Maximale :** Active toute la suite de sécurité mail (ClamAV, SpamAssassin).

### 🚀 Installation & Usage

**Installation Rapide :**

```bash
git clone https://github.com/YourRepo/cyl-manager.git /opt/cyl-manager
cd /opt/cyl-manager
sudo python3 install.py
```

**Lancer le Menu :**

```bash
sudo cyl-manager menu
```

---

<p align="center">
  Made with ❤️ and Python type hints by Cylae.
  <br>
  <em>Zero Tolerance for Technical Debt.</em>
</p>
