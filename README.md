# Cylae Server Manager (Python Edition)

![Version](https://img.shields.io/badge/version-10.0-blue.svg) ![License](https://img.shields.io/badge/license-MIT-green.svg)

**Cylae Server Manager** is a robust, modular, and "turnkey" solution for deploying and managing a self-hosted media and service ecosystem using Docker and Python. Originally written in Bash, this version (v10.0) has been rewritten in Python for better stability, maintainability, and error handling.

## Features

*   **Modular Architecture**: Built on a clean Python framework.
*   **Docker Integration**: Automates Docker Compose deployments.
*   **System Hardening**: Automatic OS checks and disk space monitoring.
*   **Service Management**: Easily install/remove services like Portainer and Netdata.
*   **Idempotency**: Safe to re-run multiple times without breaking configurations.

## Supported Services

*   **Portainer**: Docker management UI.
*   **Netdata**: Real-time infrastructure monitoring.
*   *(More coming soon: Nextcloud, WireGuard, Plex, *Arrs...)*

## Installation

### Prerequisites

*   Debian 11/12 or Ubuntu 20.04+
*   Root privileges

### Quick Start

```bash
git clone https://github.com/Cylae/server_script.git cylae-manager
cd cylae-manager
chmod +x install.sh
./install.sh
```

## Usage

Run the script to verify the system and enter the interactive menu:

```bash
./install.sh
```

Follow the on-screen instructions to install or remove services.

---

# Gestionnaire de Serveur Cylae (Édition Python)

![Version](https://img.shields.io/badge/version-10.0-bleu.svg) ![Licence](https://img.shields.io/badge/licence-MIT-vert.svg)

**Le Gestionnaire de Serveur Cylae** est une solution "clé en main", robuste et modulaire pour déployer et gérer un écosystème de médias et de services auto-hébergé utilisant Docker et Python. Initialement écrit en Bash, cette version (v10.0) a été réécrite en Python pour une meilleure stabilité, maintenabilité et gestion des erreurs.

## Fonctionnalités

*   **Architecture Modulaire** : Construit sur un framework Python propre.
*   **Intégration Docker** : Automatise les déploiements Docker Compose.
*   **Renforcement du Système** : Vérifications automatiques de l'OS et surveillance de l'espace disque.
*   **Gestion des Services** : Installez/supprimez facilement des services comme Portainer et Netdata.
*   **Idempotence** : Peut être relancé plusieurs fois sans casser les configurations existantes.

## Services Supportés

*   **Portainer** : Interface de gestion Docker.
*   **Netdata** : Surveillance d'infrastructure en temps réel.
*   *(D'autres arrivent bientôt : Nextcloud, WireGuard, Plex, *Arrs...)*

## Installation

### Prérequis

*   Debian 11/12 ou Ubuntu 20.04+
*   Privilèges Root

### Démarrage Rapide

```bash
git clone https://github.com/Cylae/server_script.git cylae-manager
cd cylae-manager
chmod +x install.sh
./install.sh
```

## Utilisation

Lancez le script pour vérifier le système et accéder au menu interactif :

```bash
./install.sh
```

Suivez les instructions à l'écran pour installer ou supprimer des services.
