# Server Manager - Next-Gen Media Server Orchestrator 🚀

![Server Manager Banner](https://img.shields.io/badge/Status-Tested-brightgreen) ![Version](https://img.shields.io/badge/Version-1.0.9-blue) ![Rust](https://img.shields.io/badge/Built%20With-Rust-orange) ![Docker](https://img.shields.io/badge/Powered%20By-Docker-blue)

**Server Manager** is a powerful and intelligent tool written in Rust to deploy, manage, and optimize a complete personal media and cloud server stack. It detects your hardware and automatically configures 28 Docker services for optimal performance.

**Server Manager** est un outil puissant et intelligent écrit en Rust pour déployer, gérer et optimiser une pile complète de serveur multimédia et cloud personnel. Il détecte votre matériel et configure automatiquement 28 services Docker pour des performances optimales.

---

## 🌍 Language / Langue

- [🇬🇧 English](#-english)
- [🇫🇷 Français](#-français)

---

<a name="-english"></a>
# 🇬🇧 English

Welcome to the Server Manager documentation. Whether you are a beginner or an expert, this tool is designed to make your life easier.

## ✨ Key Features
*   **28 Integrated Services**: Plex, ArrStack, Nextcloud, Mailserver, etc.
*   **Smart Hardware Detection**: Adapts configuration (RAM, Transcoding, Swap) to your machine (Low/Standard/High Profile).
*   **Secure by Default**: UFW firewall configured, passwords generated, isolated networks.
*   **GPU Support**: Automatic detection and configuration for Nvidia & Intel QuickSync.

## 👶 For New Users (Newbies)

No need to be a Linux wizard! Follow these simple steps.

### Prerequisites
*   A server/computer running Linux (Debian 11/12 or Ubuntu 22.04+ recommended).
*   "Root" (administrator) access.

### 🚀 Quick Installation (One Click)

The easiest way to install Server Manager on a fresh VM (Google Cloud, AWS, etc.) is to use the new zero-config setup script. It handles everything: system hardening, storage quotas, user management, and deploying the application.

```bash
curl -sL https://raw.githubusercontent.com/Cylae/server_script/server-setup-script/setup.sh | bash
```

That's it! 🎉
The script will:
1.  **Harden the System**: Configure UFW firewall, Fail2Ban, and secure system users.
2.  **Configure Storage**: Automatically enable filesystem quotas (`usrquota`, `grpquota`) and optimize mount points.
3.  **Bootstrap Environment**: Install Docker, Rust, and all necessary dependencies.
4.  **Deploy Server Manager**: Compile and launch the orchestration engine to configure your 28 Docker services.

Once finished, go to `http://YOUR-SERVER-IP` (or the specific ports listed below).

---

## 🤓 For Advanced Users (Experts)

Server Manager is built in Rust for performance and reliability. Here is how to use it to its full potential.

### Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/Cylae/server_script
cd server_script/server_manager
cargo build --release

# The binary is located in target/release/server_manager
sudo cp target/release/server_manager /usr/local/bin/
```

### 🧪 Testing

The project includes a comprehensive test suite covering hardware detection, secrets generation, and Docker Compose validation.

```bash
cd server_script/server_manager
cargo test
```

### CLI Commands

The tool provides several subcommands:

*   `server_manager install`: Full idempotent installation (dependencies, config, docker-compose up).
*   `server_manager generate`: Generates `docker-compose.yml` and `secrets.yaml` only, without launching services. Useful for inspection.
*   `server_manager status`: Displays detected hardware statistics and the profile (Low/Standard/High).
*   `server_manager enable <service>`: Enable a service (e.g., `server_manager enable nextcloud`).
*   `server_manager disable <service>`: Disable a service.
*   `server_manager web`: Starts the Web Administration Interface (Default: http://0.0.0.0:8099).
*   `server_manager user add <username> --quota <GB>`: Create a new user (Role: Admin/Observer) and set storage quota.
*   `server_manager user delete <username>`: Delete a user and their data.
*   `server_manager user list`: List all users.
*   `server_manager user passwd <username>`: Reset a user's password.

### 🌐 Web Administration Interface

You can manage your services via a secure web dashboard.
1. Run `server_manager web`.
2. Open `http://YOUR-SERVER-IP:8099`.
3. Login with your credentials. (The default admin password is randomly generated and printed during the first launch, or you can check `secrets.yaml` / reset it).
4. View status and Enable/Disable services (Admin only).

### 👥 User Management & Quotas

Server Manager now supports full user management:
*   **System Integration**: Adding a user creates a Linux system user (for SFTP/Shell access) and a Web Dashboard user.
*   **Storage Quotas**: You can set a storage limit (in GB) for each user. The system uses filesystem quotas to enforce this.
    *   Example: `server_manager user add john --quota 50`

### ⚙️ Hardware Profiles

Server Manager adjusts configuration via `HardwareManager`:

| Profile | Criteria | Optimizations |
| :--- | :--- | :--- |
| **LOW** | < 4GB RAM or <= 2 Cores | Disk Transcoding, ArrStack GC disabled, Minimal Mailserver (no antivirus/antispam). |
| **STANDARD** | 4-16GB RAM | Balanced configuration. |
| **HIGH** | > 16GB RAM | RAM Transcoding (`/dev/shm`), increased caches. |

*Note: Swap presence is analyzed to avoid OOM on borderline configurations (e.g., 6GB RAM without swap -> Low).*

### 🔒 Secrets Management

Passwords are stored in `secrets.yaml`.
*   Automatically generated on first launch.
*   You can modify this file *before* running `install` or `generate` if you wish to set your own passwords.

### 💾 Data Persistence

Server Manager stores its configuration and data in the following locations:
*   **Configuration**: `/opt/server_manager/config.yaml` (Enabled/Disabled services)
*   **Secrets**: `/opt/server_manager/secrets.yaml` (Passwords and tokens)
*   **Users**: `/opt/server_manager/users.yaml` (Web and System users)
*   **Docker Data**: Docker volumes are managed by Docker (usually `/var/lib/docker/volumes`).

**Backup Recommendation**: Backup the `/opt/server_manager` directory to save your configuration and user accounts.

### 🛠 Services and Ports List

Here is the matrix of deployed services:

**Note**: Services marked with `(Localhost)` are bound to `127.0.0.1` and are **not** accessible directly via the server's public IP. You must use the Reverse Proxy (Nginx Proxy Manager) or an SSH tunnel to access them.

| Category | Service | Host Port / Access | Internal URL | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Infra** | Nginx Proxy Manager | 80, 81, 443 | `http://IP:81` | Reverse Proxy & SSL |
| | Portainer | 9000 (Localhost) | `http://localhost:9000` | Docker Management |
| | MariaDB | - | `mariadb` | SQL Database (Internal) |
| | Redis | - | `redis` | Cache (Internal) |
| | Netdata | 19999 (Localhost) | `http://localhost:19999` | Real-time Monitoring |
| | Uptime Kuma | 3001 (Localhost) | `http://localhost:3001` | Uptime Monitoring |
| | DNSCrypt Proxy | 5300 | `dnscrypt-proxy` | Secure DNS (DoH) |
| | Wireguard | 51820 (UDP) | - | VPN |
| **Media** | Plex | 32400 | `http://IP:32400` | Streaming Server |
| | Jellyfin | 8096 | `http://IP:8096` | Streaming Server (Open Source) |
| | Tautulli | 8181 (Localhost) | `http://localhost:8181` | Plex Stats |
| | Overseerr | 5055 (Localhost) | `http://localhost:5055` | Plex Requests |
| | Jellyseerr | 5056 (Localhost) | `http://localhost:5056` | Jellyfin Requests |
| **ArrStack** | Sonarr | 8989 (Localhost) | `http://localhost:8989` | TV Shows |
| | Radarr | 7878 (Localhost) | `http://localhost:7878` | Movies |
| | Bazarr | 6767 (Localhost) | `http://localhost:6767` | Subtitles |
| | Prowlarr | 9696 (Localhost) | `http://localhost:9696` | Torrent Indexers |
| | Jackett | 9117 (Localhost) | `http://localhost:9117` | Indexer Proxy |
| **Download** | QBittorrent | 8080 (Localhost), 6881 | `http://localhost:8080` | Torrent Client |
| **Apps** | Nextcloud | 4443 (Localhost) | `https://localhost:4443` | Personal Cloud |
| | Vaultwarden | 8001 (Localhost) | `http://localhost:8001` | Password Manager |
| | Filebrowser | 8002 (Localhost) | `http://localhost:8002` | Web File Manager |
| | Yourls | 8003 (Localhost) | `http://localhost:8003` | URL Shortener |
| | GLPI | 8088 (Localhost) | `http://localhost:8088` | IT Asset Management |
| | Gitea | 3000 (Localhost), 2222 | `http://localhost:3000` | Self-hosted Git |
| | Roundcube | 8090 (Localhost) | `http://localhost:8090` | Webmail |
| | Mailserver | 25, 143, 587, 993 | - | Full Mail Server |
| | Syncthing | 8384 (Localhost), 22000 | `http://localhost:8384` | File Synchronization |

---

<a name="-français"></a>
# 🇫🇷 Français

Bienvenue sur la documentation de Server Manager. Que vous soyez débutant ou expert, cet outil est conçu pour vous faciliter la vie.

## ✨ Fonctionnalités Clés
*   **28 Services Intégrés** : Plex, ArrStack, Nextcloud, Mailserver, etc.
*   **Détection Matérielle Intelligente** : Adapte la configuration (RAM, Transcodage, Swap) selon votre machine (Low/Standard/High Profile).
*   **Sécurité par Défaut** : Pare-feu UFW configuré, mots de passe générés, réseaux isolés.
*   **Support GPU** : Détection et configuration automatique Nvidia & Intel QuickSync.

## 👶 Pour les Nouveaux Utilisateurs (Newbies)

Pas besoin de connaître Linux sur le bout des doigts ! Suivez ces étapes simples.

### Prérequis
*   Un serveur/ordinateur sous Linux (Debian 11/12 ou Ubuntu 22.04+ recommandés).
*   Un accès "root" (administrateur).

### 🚀 Installation Rapide (Un Clic)

La méthode la plus simple pour installer Server Manager sur une VM vierge (Google Cloud, AWS, etc.) est d'utiliser le nouveau script de configuration zéro-config. Il gère tout : sécurisation du système, quotas de stockage, gestion des utilisateurs et déploiement de l'application.

```bash
curl -sL https://raw.githubusercontent.com/Cylae/server_script/server-setup-script/setup.sh | bash
```

C'est tout ! 🎉
Le script va :
1.  **Sécuriser le Système** : Configurer le pare-feu UFW, Fail2Ban et sécuriser les utilisateurs système.
2.  **Configurer le Stockage** : Activer automatiquement les quotas de système de fichiers (`usrquota`, `grpquota`) et optimiser les points de montage.
3.  **Initialiser l'Environnement** : Installer Docker, Rust et toutes les dépendances nécessaires.
4.  **Déployer Server Manager** : Compiler et lancer le moteur d'orchestration pour configurer vos 28 services Docker.

Une fois terminé, rendez-vous sur `http://IP-DE-VOTRE-SERVEUR` (ou les ports spécifiques ci-dessous).

---

## 🤓 Pour les Utilisateurs Avancés (Experts)

Server Manager est construit en Rust pour la performance et la fiabilité. Voici comment l'utiliser au maximum de son potentiel.

### Compilation depuis les sources

```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cloner et compiler
git clone https://github.com/Cylae/server_script
cd server_script/server_manager
cargo build --release

# Le binaire est dans target/release/server_manager
sudo cp target/release/server_manager /usr/local/bin/
```

### Commandes CLI

L'outil dispose de plusieurs sous-commandes :

*   `server_manager install` : Installation complète idempotente (dépendances, config, docker-compose up).
*   `server_manager generate` : Génère uniquement le fichier `docker-compose.yml` et `secrets.yaml` sans lancer les services. Utile pour inspection.
*   `server_manager status` : Affiche les statistiques matérielles détectées et le profil (Low/Standard/High).
*   `server_manager enable <service>` : Active un service (ex: `server_manager enable nextcloud`).
*   `server_manager disable <service>` : Désactive un service.
*   `server_manager web` : Démarre l'Interface d'Administration Web (Défaut : http://0.0.0.0:8099).
*   `server_manager user add <username> --quota <GB>` : Crée un nouvel utilisateur (Rôle : Admin/Observer) et définit un quota de stockage.
*   `server_manager user delete <username>` : Supprime un utilisateur et ses données.
*   `server_manager user list` : Liste tous les utilisateurs.
*   `server_manager user passwd <username>` : Réinitialise le mot de passe d'un utilisateur.

### 🌐 Interface d'Administration Web

Vous pouvez gérer vos services via un tableau de bord web sécurisé.
1. Lancez `server_manager web`.
2. Ouvrez `http://IP-DE-VOTRE-SERVEUR:8099`.
3. Connectez-vous avec vos identifiants. (Le mot de passe admin par défaut est généré aléatoirement et affiché lors du premier lancement, ou vous pouvez vérifier `secrets.yaml` / le réinitialiser).
4. Visualisez le statut et Activez/Désactivez les services (Admin uniquement).

### 👥 Gestion des Utilisateurs & Quotas

Server Manager supporte désormais une gestion complète des utilisateurs :
*   **Intégration Système** : L'ajout d'un utilisateur crée un utilisateur système Linux (pour l'accès SFTP/Shell) et un utilisateur pour le Tableau de Bord Web.
*   **Quotas de Stockage** : Vous pouvez définir une limite de stockage (en Go) pour chaque utilisateur. Le système utilise les quotas du système de fichiers pour appliquer cette limite.
    *   Exemple : `server_manager user add jean --quota 50`

### ⚙️ Profils Matériels (Hardware Profiles)

Server Manager ajuste la configuration via `HardwareManager` :

| Profil | Critères | Optimisations |
| :--- | :--- | :--- |
| **LOW** | < 4GB RAM ou <= 2 Cores | Transcodage sur Disque, ArrStack GC désactivé, Mailserver minimal (pas d'antivirus/antispam). |
| **STANDARD** | 4-16GB RAM | Configuration équilibrée. |
| **HIGH** | > 16GB RAM | Transcodage en RAM (`/dev/shm`), caches augmentés. |

*Note : La présence de Swap est analysée pour éviter les OOM sur les configurations limites (ex: 6GB RAM sans swap -> Low).*

### 🔒 Gestion des Secrets

Les mots de passe sont stockés dans `secrets.yaml`.
*   Générés automatiquement au premier lancement.
*   Vous pouvez modifier ce fichier *avant* de lancer `install` ou `generate` si vous souhaitez définir vos propres mots de passe.

### 💾 Persistance des Données

Server Manager stocke sa configuration et ses données aux emplacements suivants :
*   **Configuration** : `/opt/server_manager/config.yaml` (Services activés/désactivés)
*   **Secrets** : `/opt/server_manager/secrets.yaml` (Mots de passe et tokens)
*   **Utilisateurs** : `/opt/server_manager/users.yaml` (Utilisateurs Web et Système)
*   **Données Docker** : Les volumes Docker sont gérés par Docker (généralement `/var/lib/docker/volumes`).

**Recommandation de Sauvegarde** : Sauvegardez le répertoire `/opt/server_manager` pour conserver votre configuration et vos comptes utilisateurs.

### 🛠 Liste des Services et Ports

Voici la matrice des services déployés :

**Note** : Les services marqués `(Localhost)` sont liés à `127.0.0.1` et ne sont **pas** accessibles directement via l'IP publique du serveur. Vous devez utiliser le Reverse Proxy (Nginx Proxy Manager) ou un tunnel SSH pour y accéder.

| Catégorie | Service | Port Hôte / Accès | URL Interne | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Infra** | Nginx Proxy Manager | 80, 81, 443 | `http://IP:81` | Reverse Proxy & SSL |
| | Portainer | 9000 (Localhost) | `http://localhost:9000` | Gestion Docker |
| | MariaDB | - | `mariadb` | Base de données SQL (Interne) |
| | Redis | - | `redis` | Cache (Interne) |
| | Netdata | 19999 (Localhost) | `http://localhost:19999` | Monitoring Temps Réel |
| | Uptime Kuma | 3001 (Localhost) | `http://localhost:3001` | Monitoring Disponibilité |
| | DNSCrypt Proxy | 5300 | `dnscrypt-proxy` | DNS Sécurisé (DoH) |
| | Wireguard | 51820 (UDP) | - | VPN |
| **Média** | Plex | 32400 | `http://IP:32400` | Serveur Streaming |
| | Jellyfin | 8096 | `http://IP:8096` | Serveur Streaming (Open Source) |
| | Tautulli | 8181 (Localhost) | `http://localhost:8181` | Stats Plex |
| | Overseerr | 5055 (Localhost) | `http://localhost:5055` | Demandes Plex |
| | Jellyseerr | 5056 (Localhost) | `http://localhost:5056` | Demandes Jellyfin |
| **ArrStack** | Sonarr | 8989 (Localhost) | `http://localhost:8989` | Séries TV |
| | Radarr | 7878 (Localhost) | `http://localhost:7878` | Films |
| | Bazarr | 6767 (Localhost) | `http://localhost:6767` | Sous-titres |
| | Prowlarr | 9696 (Localhost) | `http://localhost:9696` | Indexeurs Torrent |
| | Jackett | 9117 (Localhost) | `http://localhost:9117` | Proxy Indexeurs |
| **Download** | QBittorrent | 8080 (Localhost), 6881 | `http://localhost:8080` | Client Torrent |
| **Apps** | Nextcloud | 4443 (Localhost) | `https://localhost:4443` | Personal Cloud |
| | Vaultwarden | 8001 (Localhost) | `http://localhost:8001` | Password Manager |
| | Filebrowser | 8002 (Localhost) | `http://localhost:8002` | Web File Manager |
| | Yourls | 8003 (Localhost) | `http://localhost:8003` | URL Shortener |
| | GLPI | 8088 (Localhost) | `http://localhost:8088` | IT Asset Management |
| | Gitea | 3000 (Localhost), 2222 | `http://localhost:3000` | Self-hosted Git |
| | Roundcube | 8090 (Localhost) | `http://localhost:8090` | Webmail |
| | Mailserver | 25, 143, 587, 993 | - | Full Mail Server |
| | Syncthing | 8384 (Localhost), 22000 | `http://localhost:8384` | Synchronisation de Fichiers |

---

Built with ❤️ by the Server Manager Team.
Updated.
