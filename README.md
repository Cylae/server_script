# 🚀 Cylae: The Next-Gen Media Server Orchestrator

[![Rust](https://img.shields.io/badge/built_with-Rust-d62f02)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**Cylae** is a high-performance, hardware-aware orchestration tool for home media servers. It replaces legacy shell scripts with a robust Rust-based engine that treats your hardware profile as a first-class citizen.

---

## 🇬🇧 English Guide

### 🌟 For New Users (Start Here)

Welcome! Cylae makes setting up your media server incredibly easy.

1.  **Install:**
    Simply run the binary. It will handle dependencies, Docker installation, and configuration for you.
    ```bash
    ./cylae install
    ```

2.  **Access:**
    Once finished, your services will be available at your server's IP address.
    *   **Dashboard (Portainer):** `http://<your-ip>:9000`
    *   **Plex:** `http://<your-ip>:32400`
    *   **Nginx Proxy Manager:** `http://<your-ip>:81`

3.  **Enjoy:**
    Cylae automatically detects if you have a powerful server or a low-power device and tunes your services (like Plex transcoding and Database memory) accordingly!

### 🔧 For Advanced Users

Cylae is an "Infrastructure as Code" compiler.

*   **Idempotency:** Run `cylae install` as many times as you want. It converges the system state.
*   **Hardware Profiling:**
    *   **Low Profile (<4GB RAM / <2 Cores):** Disables ClamAV/SpamAssassin in Mailserver, tunes GC for *Arr apps, disables disk transcoding.
    *   **High Profile (>16GB RAM):** Enables RAM transcoding (`/dev/shm`), maximizes DB buffer pools.
    *   **GPU:** Auto-detects Nvidia & Intel QuickSync for hardware acceleration.
*   **Commands:**
    *   `cylae install`: Full system convergence.
    *   `cylae generate`: Output the `docker-compose.yml` without running it (dry-run).
    *   `cylae status`: View detected hardware stats.

---

## 🇫🇷 Guide Français

### 🌟 Pour les Nouveaux Utilisateurs (Commencez Ici)

Bienvenue ! Cylae rend l'installation de votre serveur multimédia incroyablement simple.

1.  **Installation :**
    Lancez simplement le binaire. Il s'occupe des dépendances, de Docker et de la configuration.
    ```bash
    ./cylae install
    ```

2.  **Accès :**
    Une fois terminé, vos services seront accessibles via l'adresse IP de votre serveur.
    *   **Tableau de bord (Portainer) :** `http://<votre-ip>:9000`
    *   **Plex :** `http://<votre-ip>:32400`
    *   **Nginx Proxy Manager :** `http://<votre-ip>:81`

3.  **Profitez :**
    Cylae détecte automatiquement la puissance de votre serveur et optimise vos services (comme le transcodage Plex et la mémoire des bases de données) en conséquence !

### 🔧 Pour les Utilisateurs Avancés

Cylae est un compilateur "Infrastructure as Code".

*   **Idempotence :** Exécutez `cylae install` autant de fois que nécessaire. Il converge l'état du système.
*   **Profilage Matériel :**
    *   **Profil Bas (<4Go RAM / <2 Cœurs) :** Désactive ClamAV/SpamAssassin, ajuste le GC pour les applis *Arr, désactive le transcodage disque.
    *   **Profil Haut (>16Go RAM) :** Active le transcodage en RAM (`/dev/shm`), maximise les pools de mémoire DB.
    *   **GPU :** Détection automatique Nvidia & Intel QuickSync.
*   **Commandes :**
    *   `cylae install` : Convergence complète du système.
    *   `cylae generate` : Génère le `docker-compose.yml` sans le lancer.
    *   `cylae status` : Voir les stats matérielles détectées.

---

## 🛠 Supported Services
Cylae orchestrates a massive stack of **24 services**:

| Category | Services |
| :--- | :--- |
| **Media** | Plex, Tautulli, Overseerr |
| **Arrs** | Sonarr, Radarr, Prowlarr, Jackett |
| **Downloads** | QBittorrent |
| **Infra** | MariaDB, Redis, Nginx Proxy, DNSCrypt, Wireguard, Portainer, Netdata, Uptime-Kuma |
| **Apps** | Nextcloud, Vaultwarden, Filebrowser, Yourls, Mailserver, GLPI, Gitea, Roundcube |
