# Cylae Server Manager: The Ultimate Optimized Media Server Stack

[![Stability Verified: CLI & Web Interface Tested](https://img.shields.io/badge/Stability-Verified-green)](https://github.com/Cylae/server_script)
[![Architecture: Clean Slate](https://img.shields.io/badge/Architecture-Clean_Slate-blue)](https://github.com/Cylae/server_script)
[![Optimization: GDHD Enabled](https://img.shields.io/badge/Optimization-GDHD_Active-orange)](https://github.com/Cylae/server_script)

**Cylae Server Manager** is a high-performance, modular DevOps framework designed to deploy, orchestrate, and optimize a self-hosted media ecosystem.

---

## 🇺🇸 English Guide

### 👶 For Newbies
Welcome to your new home server! 🏠✨ This tool will help you install everything you need (Plex, Sonarr, Radarr, etc.) without needing a PhD in computer science.

#### How to Install 🛠️
1.  **Get a Linux Server:** You need a machine running Debian 11+ or Ubuntu 20.04+.
2.  **Open the Terminal:** Connect to your server via SSH.
3.  **Run these Commands:**
    ```bash
    git clone https://github.com/Cylae/server_script.git
    cd server_script
    sudo python3 install.py
    ```
4.  **Follow the Menu:** The script will ask you what you want to install. Just select and press Enter! ✅

#### What is "GDHD"? 🤖
Don't worry about this acronym! It simply means the script is **smart**. It looks at your server's RAM and CPU and automatically adjusts the settings so your server doesn't crash. 🛡️
*   **Small Server ($5 VPS)?** We turn on "Survival Mode" to keep it running smoothly.
*   **Big Server?** We unlock "Turbo Mode" for maximum speed! 🏎️💨

### 🧑‍💻 For Advanced Users & Coders
**Cylae Server Manager** is a Python 3.9+ type-safe orchestration framework built with a "Clean Slate" philosophy. It abandons legacy bash spaghetti for a modular, object-oriented architecture.

#### Core Architecture 🏗️
*   **Global Dynamic Hardware Detection (GDHD):** An algorithmic heuristic engine that profiles the host substrate (CPU cores, RAM, Swap) before deployment.
    *   **Low Profile (<4GB RAM):** Enforces strict resource limits, disables non-essential sidecars (ClamAV, SpamAssassin), uses Workstation GC for .NET apps, and serializes container deployment.
    *   **High Profile:** Enables parallel orchestration, zero-copy RAM transcoding for Plex, and aggressive caching strategies (Redis sidecars).
*   **Security First:**
    *   **Least Privilege:** All containers run with `no-new-privileges:true` by default.
    *   **Network Isolation:** Services communicate via an internal `cylae_net` bridge; only necessary ports are exposed via `ufw`.
*   **Observability:** Built-in TUI (Rich) and Web Dashboard (Flask).

#### Service optimizations ⚡
*   **MariaDB:** Dynamically generates `custom.cnf` to tune `innodb_buffer_pool_size` (128M vs 1G) based on GDHD profile.
*   **Nextcloud:** Automatically deploys a Redis sidecar for object locking and caching.
*   **Plex:** Context-aware transcoding mapping (RAM `/tmp` vs Disk) based on available memory.

#### CLI Usage ⌨️
```bash
# Interactive Menu
cyl-manager menu

# Headless Full Stack Deployment
cyl-manager install_all

# Real-time Status & Heuristics
cyl-manager status
```

---

## 🇫🇷 Guide Français

### 👶 Pour les Débutants
Bienvenue sur votre nouveau serveur multimédia ! 🏠✨ Cet outil va vous aider à installer tout ce dont vous avez besoin (Plex, Sonarr, Radarr, etc.) sans avoir besoin d'un diplôme d'ingénieur.

#### Comment Installer 🛠️
1.  **Avoir un Serveur Linux :** Une machine sous Debian 11+ ou Ubuntu 20.04+.
2.  **Ouvrir le Terminal :** Connectez-vous en SSH.
3.  **Lancer ces Commandes :**
    ```bash
    git clone https://github.com/Cylae/server_script.git
    cd server_script
    sudo python3 install.py
    ```
4.  **Suivre le Menu :** Le script vous demandera ce que vous voulez installer. Sélectionnez et validez ! ✅

#### C'est quoi "GDHD" ? 🤖
Ne vous inquiétez pas pour cet acronyme ! Cela signifie simplement que le script est **intelligent**. Il regarde la RAM et le processeur de votre serveur et ajuste automatiquement les réglages pour éviter les plantages. 🛡️
*   **Petit Serveur (VPS à 5€) ?** On active le "Mode Survie" pour que tout reste fluide.
*   **Gros Serveur ?** On débloque le "Mode Turbo" pour une vitesse maximale ! 🏎️💨

### 🧑‍💻 Pour les Experts & Développeurs
**Cylae Server Manager** est un framework d'orchestration en Python 3.9+ (type-safe) conçu avec une philosophie "Tabula Rasa". Fini le spaghetti bash, place à une architecture modulaire et orientée objet.

#### Architecture Principale 🏗️
*   **Global Dynamic Hardware Detection (GDHD) :** Un moteur heuristique qui profile le substrat hôte (Cœurs CPU, RAM, Swap) avant le déploiement.
    *   **Profil Bas (<4GB RAM) :** Impose des limites strictes, désactive les sidecars non essentiels (ClamAV, SpamAssassin), force le GC Workstation pour les apps .NET, et sérialise les déploiements.
    *   **Profil Haut :** Active l'orchestration parallèle, le transcodage RAM (Zero-Copy) pour Plex, et des stratégies de cache agressives (Redis).
*   **Sécurité Avant Tout :**
    *   **Moindre Privilège :** Tous les conteneurs tournent avec `no-new-privileges:true` par défaut.
    *   **Isolation Réseau :** Les services communiquent via un bridge interne `cylae_net`; seuls les ports nécessaires sont exposés via `ufw`.
*   **Observabilité :** Interface TUI (Rich) intégrée et Dashboard Web (Flask).

#### Optimisations par Service ⚡
*   **MariaDB :** Génère dynamiquement un `custom.cnf` pour ajuster la taille du `innodb_buffer_pool` (128M vs 1G) selon le profil GDHD.
*   **Nextcloud :** Déploie automatiquement un conteneur Redis sidecar pour le verrouillage d'objets et le cache.
*   **Plex :** Mapping de transcodage contextuel (RAM `/tmp` vs Disque) basé sur la mémoire disponible.

#### Utilisation CLI ⌨️
```bash
# Menu Interactif
cyl-manager menu

# Déploiement Complet (Headless)
cyl-manager install_all

# Statut Temps Réel & Heuristiques
cyl-manager status
```
