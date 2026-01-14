# Cylae Server Manager 🚀

[![Python](https://img.shields.io/badge/Python-3.9%2B-blue.svg)](https://python.org)
[![Docker](https://img.shields.io/badge/Docker-Enabled-blue.svg)](https://docker.com)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Code Style: Black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)

🇬🇧 **English** | [🇫🇷 Français](#-gestionnaire-de-serveur-cylae)

---

## 🇬🇧 Cylae Server Manager

**The Ultimate Self-Hosted Media Ecosystem Deployer.**

Cylae Server Manager is a production-grade, modular Python framework designed to deploy, manage, and optimize a complete self-hosted media and infrastructure stack. Built with an obsession for clean code, performance, and security.

### 🔥 Features

*   **Intelligent Orchestration:** Automatically adjusts deployment concurrency based on your hardware profile (CPU/RAM/Swap).
*   **Hardware Profiling:** Dynamically tunes service configurations (e.g., Plex transcoding to RAM on high-end systems, disabled heavy mail filters on low-end VPS).
*   **Modular Architecture:** Strictly typed, PEP 8 compliant, and extensible service registry.
*   **Zero-Downtime:** Uses Docker Compose for idempotent deployments.
*   **Interactive CLI:** Beautiful `rich` text user interface for easy management.

### 🛠️ Tech Stack

*   **Core:** Python 3.9+, `pydantic`, `typer`, `rich`
*   **Infrastructure:** Docker, Docker Compose
*   **Services:** Plex, *Arr Suite, Gitea, Nextcloud, MailServer, and more.

### 🚀 Getting Started

#### Prerequisites

*   A Linux server (Debian/Ubuntu recommended)
*   Root privileges

#### Installation

```bash
git clone https://github.com/Cylae/server_script.git
cd server_script
sudo ./install.sh
```

#### Usage

Launch the interactive menu:

```bash
sudo cyl-manager menu
```

Or use the CLI directly:

```bash
sudo cyl-manager install plex
sudo cyl-manager status
sudo cyl-manager install-all
```

---

## 🇫🇷 Gestionnaire de Serveur Cylae

**L'outil ultime de déploiement d'écosystème média auto-hébergé.**

Cylae Server Manager est un framework Python modulaire de qualité production conçu pour déployer, gérer et optimiser une pile complète de médias et d'infrastructure. Construit avec une obsession pour le code propre, la performance et la sécurité.

### 🔥 Fonctionnalités

*   **Orchestration Intelligente :** Ajuste automatiquement la concomitance du déploiement en fonction de votre profil matériel (CPU/RAM/Swap).
*   **Profilage Matériel :** Ajuste dynamiquement les configurations des services (ex: transcodage Plex en RAM sur les systèmes puissants, filtres mail lourds désactivés sur les VPS modestes).
*   **Architecture Modulaire :** Typage strict, conformité PEP 8 et registre de services extensible.
*   **Zéro Interruption :** Utilise Docker Compose pour des déploiements idempotents.
*   **CLI Interactive :** Interface utilisateur magnifique basée sur `rich`.

### 🛠️ Stack Technique

*   **Cœur :** Python 3.9+, `pydantic`, `typer`, `rich`
*   **Infrastructure :** Docker, Docker Compose
*   **Services :** Plex, Suite *Arr, Gitea, Nextcloud, MailServer, et plus.

### 🚀 Démarrage

#### Prérequis

*   Un serveur Linux (Debian/Ubuntu recommandé)
*   Privilèges Root

#### Installation

```bash
git clone https://github.com/Cylae/server_script.git
cd server_script
sudo ./install.sh
```

#### Utilisation

Lancer le menu interactif :

```bash
sudo cyl-manager menu
```

Ou utiliser la CLI directement :

```bash
sudo cyl-manager install plex
sudo cyl-manager status
sudo cyl-manager install-all
```
