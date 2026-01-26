# Server Manager - Next-Gen Media Server Orchestrator 🚀

![Server Manager Banner](https://img.shields.io/badge/Status-Tested-brightgreen) ![Rust](https://img.shields.io/badge/Built%20With-Rust-orange) ![Docker](https://img.shields.io/badge/Powered%20By-Docker-blue)

**Server Manager** is a powerful and intelligent tool written in Rust to deploy, manage, and optimize a complete personal media and cloud server stack. It detects your hardware and automatically configures 24 Docker services for optimal performance.

**Server Manager** est un outil puissant et intelligent écrit en Rust pour déployer, gérer et optimiser une pile complète de serveur multimédia et cloud personnel. Il détecte votre matériel et configure automatiquement 24 services Docker pour des performances optimales.

---

## 🌍 Language / Langue

- [🇬🇧 English](#-english)
- [🇫🇷 Français](#-français)

---

<a name="-english"></a>
# 🇬🇧 English

Welcome to the Server Manager documentation. Whether you are a beginner or an expert, this tool is designed to make your life easier.

## ✨ Key Features
*   **24 Integrated Services**: Plex, ArrStack, Nextcloud, Mailserver, etc.
*   **Smart Hardware Detection**: Adapts configuration (RAM, Transcoding, Swap) to your machine (Low/Standard/High Profile).
*   **Secure by Default**: UFW firewall configured, passwords generated, isolated networks.
*   **GPU Support**: Automatic detection and configuration for Nvidia & Intel QuickSync.

## 👶 For New Users (Newbies)

No need to be a Linux wizard! Follow these simple steps.

### Prerequisites
*   A server/computer running Linux (Debian 11/12 or Ubuntu 22.04+ recommended).
*   "Root" (administrator) access.

### 🚀 Quick Installation

1.  **Download the binary** (or build it if you don't have the pre-compiled binary).
2.  **Start the installation** with a single command:

```bash
sudo ./server_manager install
```

That's it! 🎉
Server Manager will automatically:
1.  Install system dependencies (curl, git, build-essential, etc.).
2.  Check and install Docker.
3.  Scan your hardware (RAM, CPU, Disk).
4.  Generate secure passwords (`secrets.yaml`).
5.  Configure the firewall.
6.  Launch all services.

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

### 🛠 Services and Ports List

Here is the matrix of deployed services:

| Category | Service | Port (Host) | Internal URL | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Infra** | Nginx Proxy Manager | 80, 81, 443 | `http://IP:81` | Reverse Proxy & SSL |
| | Portainer | 9000 | `http://IP:9000` | Docker Management |
| | MariaDB | 3306 | `mariadb` | SQL Database |
| | Redis | 6379 | `redis` | Cache |
| | Netdata | 19999 | `http://IP:19999` | Real-time Monitoring |
| | Uptime Kuma | 3001 | `http://IP:3001` | Uptime Monitoring |
| | DNSCrypt Proxy | 5300 | `dnscrypt-proxy` | Secure DNS (DoH) |
| | Wireguard | 51820 (UDP) | - | VPN |
| **Media** | Plex | 32400 | `http://IP:32400` | Streaming Server |
| | Tautulli | 8181 | `http://IP:8181` | Plex Stats |
| | Overseerr | 5055 | `http://IP:5055` | Media Requests |
| **ArrStack** | Sonarr | 8989 | `http://IP:8989` | TV Shows |
| | Radarr | 7878 | `http://IP:7878` | Movies |
| | Prowlarr | 9696 | `http://IP:9696` | Torrent Indexers |
| | Jackett | 9117 | `http://IP:9117` | Indexer Proxy |
| **Download** | QBittorrent | 8080 | `http://IP:8080` | Torrent Client |
| **Apps** | Nextcloud | 4443 | `https://IP:4443` | Personal Cloud |
| | Vaultwarden | 8001 | `http://IP:8001` | Password Manager |
| | Filebrowser | 8002 | `http://IP:8002` | Web File Manager |
| | Yourls | 8003 | `http://IP:8003` | URL Shortener |
| | GLPI | 8088 | `http://IP:8088` | IT Asset Management |
| | Gitea | 3000, 2222 | `http://IP:3000` | Self-hosted Git |
| | Roundcube | 8090 | `http://IP:8090` | Webmail |
| | Mailserver | 25, 143, 587, 993 | - | Full Mail Server |

---

<a name="-français"></a>
# 🇫🇷 Français

Bienvenue sur la documentation de Server Manager. Que vous soyez débutant ou expert, cet outil est conçu pour vous faciliter la vie.

## ✨ Fonctionnalités Clés
*   **24 Services Intégrés** : Plex, ArrStack, Nextcloud, Mailserver, etc.
*   **Détection Matérielle Intelligente** : Adapte la configuration (RAM, Transcodage, Swap) selon votre machine (Low/Standard/High Profile).
*   **Sécurité par Défaut** : Pare-feu UFW configuré, mots de passe générés, réseaux isolés.
*   **Support GPU** : Détection et configuration automatique Nvidia & Intel QuickSync.

## 👶 Pour les Nouveaux Utilisateurs (Newbies)

Pas besoin de connaître Linux sur le bout des doigts ! Suivez ces étapes simples.

### Prérequis
*   Un serveur/ordinateur sous Linux (Debian 11/12 ou Ubuntu 22.04+ recommandés).
*   Un accès "root" (administrateur).

### 🚀 Installation Rapide

1.  **Téléchargez le binaire** (ou compilez-le si vous n'avez pas le binaire pré-compilé).
2.  **Lancez l'installation** avec une seule commande :

```bash
sudo ./server_manager install
```

C'est tout ! 🎉
Server Manager va automatiquement :
1.  Installer les dépendances système (curl, git, build-essential, etc.).
2.  Vérifier et installer Docker.
3.  Scanner votre matériel (RAM, CPU, Disque).
4.  Générer des mots de passe sécurisés (`secrets.yaml`).
5.  Configurer le pare-feu.
6.  Lancer tous les services.

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

### 🛠 Liste des Services et Ports

Voici la matrice des services déployés :

| Catégorie | Service | Port (Hôte) | URL Interne | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Infra** | Nginx Proxy Manager | 80, 81, 443 | `http://IP:81` | Reverse Proxy & SSL |
| | Portainer | 9000 | `http://IP:9000` | Gestion Docker |
| | MariaDB | 3306 | `mariadb` | Base de données SQL |
| | Redis | 6379 | `redis` | Cache |
| | Netdata | 19999 | `http://IP:19999` | Monitoring Temps Réel |
| | Uptime Kuma | 3001 | `http://IP:3001` | Monitoring Disponibilité |
| | DNSCrypt Proxy | 5300 | `dnscrypt-proxy` | DNS Sécurisé (DoH) |
| | Wireguard | 51820 (UDP) | - | VPN |
| **Média** | Plex | 32400 | `http://IP:32400` | Serveur Streaming |
| | Tautulli | 8181 | `http://IP:8181` | Stats Plex |
| | Overseerr | 5055 | `http://IP:5055` | Demandes de Média |
| **ArrStack** | Sonarr | 8989 | `http://IP:8989` | Séries TV |
| | Radarr | 7878 | `http://IP:7878` | Films |
| | Prowlarr | 9696 | `http://IP:9696` | Indexeurs Torrent |
| | Jackett | 9117 | `http://IP:9117` | Proxy Indexeurs |
| **Download** | QBittorrent | 8080 | `http://IP:8080` | Client Torrent |
| **Apps** | Nextcloud | 4443 | `https://IP:4443` | Personal Cloud |
| | Vaultwarden | 8001 | `http://IP:8001` | Password Manager |
| | Filebrowser | 8002 | `http://IP:8002` | Web File Manager |
| | Yourls | 8003 | `http://IP:8003` | URL Shortener |
| | GLPI | 8088 | `http://IP:8088` | IT Asset Management |
| | Gitea | 3000, 2222 | `http://IP:3000` | Self-hosted Git |
| | Roundcube | 8090 | `http://IP:8090` | Webmail |
| | Mailserver | 25, 143, 587, 993 | - | Full Mail Server |

---

Built with ❤️ by the Server Manager Team.
