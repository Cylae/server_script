# 🚀 CYL.AE Server Manager (v6.0)

![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Bash](https://img.shields.io/badge/language-Bash-4EAA25.svg) ![Docker](https://img.shields.io/badge/container-Docker-2496ED.svg) ![Status](https://img.shields.io/badge/status-Production%20Ready-success.svg)

> **The Ultimate "Set & Forget" Self-Hosting Solution.**  
> *Performance Edition | Auto-Tuning | Fully Modular*

---

## 🇬🇧 English Version

### 📖 Introduction
**CYL.AE Server Manager** is a premium, all-in-one Bash framework designed to transform a fresh Debian/Ubuntu server into a production-ready fortress. 

Unlike standard installers, this is an intelligent **Lifecycle Manager**. It doesn't just install software; it maintains it. It detects your hardware to optimize performance, manages services via Docker, handles SSL certificates automatically, and even updates itself and your entire system while you sleep.

### 🏗️ Architecture
The system is built on a robust stack designed for stability and speed.

```mermaid
graph LR
    %% --- STYLING ---
    classDef user fill:#2d3436,stroke:#dfe6e9,stroke-width:2px,color:#fff;
    classDef shield fill:#0984e3,stroke:#dfe6e9,stroke-width:2px,color:#fff;
    classDef app fill:#00b894,stroke:#dfe6e9,stroke-width:2px,color:#fff;
    classDef data fill:#6c5ce7,stroke:#dfe6e9,stroke-width:2px,color:#fff;
    classDef auto fill:#e17055,stroke:#dfe6e9,stroke-width:2px,color:#fff;

    %% --- NODES ---
    User(("👤 YOU")):::user
    
    subgraph Gateway ["🛡️ SECURE GATEWAY"]
        direction TB
        FW["🔥 Firewall"]:::shield
        Proxy["⚡ Nginx Proxy"]:::shield
        SSL["🔒 SSL Certs"]:::shield
    end

    subgraph Ecosystem ["🚀 YOUR APPS"]
        direction TB
        Dash["🖥️ Dashboard"]:::app
        Cloud["☁️ Nextcloud"]:::app
        Git["🐙 Gitea"]:::app
        Pass["🔑 Vaultwarden"]:::app
        Mail["📧 Mail"]:::app
    end

    subgraph Engine ["⚙️ CORE ENGINE"]
        direction TB
        DB[("🗄️ Databases")]:::data
        Docker[("🐳 Docker")]:::data
    end

    subgraph Guardian ["🤖 AUTO-PILOT"]
        direction TB
        Update["🔄 Auto-Update"]:::auto
        Backup["💾 Auto-Backup"]:::auto
    end

    %% --- FLOW ---
    User ==> FW
    FW ==> Proxy
    Proxy -.-> SSL
    
    Proxy ==> Dash
    Proxy ==> Cloud
    Proxy ==> Git
    Proxy ==> Pass
    Proxy ==> Mail

    Ecosystem -.-> Engine
    Guardian -.-> Engine
    Guardian -.-> Ecosystem
```

### ✨ Key Features

#### 🧠 Intelligent Auto-Tuning
The script analyzes your server's RAM at startup:
*   **< 4GB RAM**: Activates "Low Profile". Optimizes MariaDB for low memory footprint, limits PHP workers.
*   **> 4GB RAM**: Activates "High Performance". Allocates generous buffers for MariaDB and PHP for maximum speed.

#### ⚡ Performance & Network
*   **TCP BBR**: Automatically enables Google's BBR congestion control algorithm.
*   **Swap Management**: Creates a 2GB Swap file to prevent OOM crashes.
*   **DNS Tuning**: Configures systemd-resolved to use high-speed Google & Cloudflare DNS resolvers.
*   **Nginx Tuning**: Configured for high-concurrency with HTTP/2 support.

#### 🛡️ Ironclad Security
*   **Firewall (UFW)**: Only essential ports are opened. Docker subnet is whitelisted for internal comms.
*   **Fail2Ban**: Protects SSH and HTTP against brute-force attacks.
*   **SSH Hardening**: Option 16 allows you to disable Password Authentication with one click (Keys Only).
*   **SSL Everywhere**: Automatic Let's Encrypt certificates for all subdomains.

#### 🤖 Auto-Pilot Mode
A background cron job runs every night at **04:00 AM**:
1.  **Self-Update**: Pulls the latest version of this script from Git.
2.  **System Update**: Runs `apt-get update && upgrade`.
3.  **Container Update**: Uses Watchtower to update all running Docker containers.
4.  **Cleanup**: Prunes unused Docker images to save disk space.
5.  **SSL**: Checks and renews certificates if needed.

### 🚀 Quick Start

**Prerequisites:** A fresh Debian 11/12 or Ubuntu 20.04/22.04 server.

1.  **Clone the repo:**
    ```bash
    git clone https://github.com/your-repo/server_script.git
    cd server_script
    ```

2.  **Run the script (as root):**
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

3.  **Follow the menu!** Select the services you want to install.

### 📦 Available Modules

| Service | Subdomain | Description |
| :--- | :--- | :--- |
| **Admin Dashboard** | `admin.cyl.ae` | Centralized dashboard to manage all your services. |
| **Gitea** | `git.cyl.ae` | Lightweight, self-hosted Git service (GitHub alternative). |
| **Nextcloud** | `cloud.cyl.ae` | Your personal cloud for files, contacts, and calendar. |
| **Vaultwarden** | `pass.cyl.ae` | Secure password manager (Bitwarden compatible). |
| **Uptime Kuma** | `status.cyl.ae` | Beautiful monitoring dashboard for your services. |
| **Portainer** | `portainer.cyl.ae` | GUI to manage your Docker containers easily. |
| **Netdata** | `netdata.cyl.ae` | Real-time performance monitoring (CPU, RAM, Network). |
| **Mail Server** | `mail.cyl.ae` | Full-stack mail server (Postfix, Dovecot, Roundcube). |
| **YOURLS** | `x.cyl.ae` | Your own URL shortener. |
| **FTP** | N/A | Classic FTP server for legacy file transfer needs. |

### 🛠️ Advanced Usage

*   **DNS Helper**: Option 15 calculates the exact DNS records (A, CNAME, MX, TXT) you need to add to your registrar.
*   **Backups**: Option 11 performs a full backup (Database SQL dumps + Files) to `/var/backups/cyl_manager`.
*   **Force Re-init**: Option 13 allows you to force a full system re-initialization if you need to reset configurations.

---

## 🇫🇷 Version Française

### 📖 Introduction
**CYL.AE Server Manager** est un framework Bash premium tout-en-un, conçu pour transformer un serveur Debian/Ubuntu vierge en une forteresse de production.

Contrairement aux installeurs classiques, c'est un **Gestionnaire de Cycle de Vie** intelligent. Il ne se contente pas d'installer des logiciels ; il les maintient. Il détecte votre matériel pour optimiser les performances, gère les services via Docker, s'occupe des certificats SSL automatiquement, et met même à jour le système entier (et lui-même) pendant que vous dormez.

### 🏗️ Architecture
Le système repose sur une stack robuste conçue pour la stabilité et la vitesse.

*(Voir le diagramme Mermaid ci-dessus)*

### ✨ Fonctionnalités Clés

#### 🧠 Auto-Tuning Intelligent
Le script analyse la RAM de votre serveur au démarrage :
*   **< 4GB RAM** : Active le "Profil Bas". Optimise MariaDB pour une faible empreinte mémoire.
*   **> 4GB RAM** : Active la "Haute Performance". Alloue des buffers généreux pour une vitesse maximale.

#### ⚡ Performance & Réseau
*   **TCP BBR** : Active automatiquement l'algorithme BBR de Google pour une vitesse réseau fulgurante.
*   **Gestion Swap** : Crée un fichier Swap de 2GB pour éviter les crashs OOM.
*   **Tuning DNS** : Configure systemd-resolved pour utiliser les DNS rapides Google & Cloudflare.
*   **Tuning Nginx** : Configuré pour une haute concurrence avec support HTTP/2.

#### 🛡️ Sécurité Béton
*   **Pare-feu (UFW)** : Seuls les ports essentiels sont ouverts. Le sous-réseau Docker est whitelisté.
*   **Fail2Ban** : Protège SSH et HTTP contre les attaques par force brute.
*   **Durcissement SSH** : L'option 16 permet de désactiver l'authentification par mot de passe en un clic (Clés uniquement).
*   **SSL Partout** : Certificats Let's Encrypt automatiques pour tous vos sous-domaines.

#### 🤖 Mode Pilote Automatique
Une tâche de fond s'exécute chaque nuit à **04h00** :
1.  **Auto-Update** : Récupère la dernière version de ce script depuis Git.
2.  **Mise à jour Système** : Lance `apt-get update && upgrade`.
3.  **Mise à jour Conteneurs** : Utilise Watchtower pour mettre à jour tous les conteneurs Docker.
4.  **Nettoyage** : Supprime les images Docker inutilisées pour gagner de la place.
5.  **SSL** : Vérifie et renouvelle les certificats si nécessaire.

### 🚀 Démarrage Rapide

**Prérequis :** Un serveur Debian 11/12 ou Ubuntu 20.04/22.04 vierge.

1.  **Cloner le dépôt :**
    ```bash
    git clone https://github.com/votre-repo/server_script.git
    cd server_script
    ```

2.  **Lancer le script (en root) :**
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

3.  **Suivez le menu !** Sélectionnez les services que vous souhaitez installer.

### 📦 Modules Disponibles

| Service | Sous-domaine | Description |
| :--- | :--- | :--- |
| **Admin Dashboard** | `admin.cyl.ae` | Tableau de bord centralisé pour gérer tous vos services. |
| **Gitea** | `git.cyl.ae` | Service Git léger auto-hébergé (alternative à GitHub). |
| **Nextcloud** | `cloud.cyl.ae` | Votre cloud personnel pour fichiers, contacts et calendrier. |
| **Vaultwarden** | `pass.cyl.ae` | Gestionnaire de mots de passe sécurisé (compatible Bitwarden). |
| **Uptime Kuma** | `status.cyl.ae` | Tableau de bord de surveillance magnifique pour vos services. |
| **Portainer** | `portainer.cyl.ae` | Interface graphique pour gérer vos conteneurs Docker. |
| **Netdata** | `netdata.cyl.ae` | Monitoring de performance temps réel (CPU, RAM, Réseau). |
| **Mail Server** | `mail.cyl.ae` | Serveur mail complet (Postfix, Dovecot, Roundcube). |
| **YOURLS** | `x.cyl.ae` | Votre propre raccourcisseur d'URL. |
| **FTP** | N/A | Serveur FTP classique pour les besoins de transfert legacy. |

### 🛠️ Avancé

*   **Assistant DNS** : L'option 15 calcule les enregistrements DNS exacts à ajouter chez votre registrar.
*   **Sauvegardes** : L'option 11 effectue une sauvegarde complète (Dumps SQL + Fichiers) dans `/var/backups/cyl_manager`.
*   **Force Re-init** : L'option 13 vous permet de forcer une réinitialisation complète du système si vous avez besoin de remettre les configurations à zéro.

---
*Made with ❤️ for Cylae.*
