<div id="english">

# Cylae Server Manager (Secure Edition)

**Cylae Server Manager** is a production-grade, modular, and secure deployment suite for self-hosted services. It simplifies the installation, management, and maintenance of popular applications using Docker, Nginx, and automated scripting.

## Features

- **Modular Architecture:** Core logic separated from service definitions for easy maintenance.
- **Security First:**
  - Automated Firewall (UFW) configuration.
  - Fail2Ban integration for SSH.
  - Hardened SSH configuration (Root login disabled, Key-based auth preferred).
  - Secure credential management (generated or user-provided).
- **Automated Infrastructure:**
  - Docker & Docker Compose (V2) installation.
  - Nginx Reverse Proxy with automated Let's Encrypt SSL.
  - Centralized MariaDB database with performance tuning.
  - Redis for caching (Nextcloud).
- **Smart Optimization:**
  - Auto-detects system resources (Low vs. High profile).
  - Tunes Kernel (BBR, TCP stack), Database, and PHP settings.
- **Backups:** Automated backup of data, databases, and configuration.
- **Auto-Update:** Self-updating mechanism for the manager script.

## Service Catalog

| Service | Description | URL |
| :--- | :--- | :--- |
| **FileBrowser** | Web-based file manager | `files.$DOMAIN` |
| **FTP** | vsftpd server | `ftp://$DOMAIN` |
| **Gitea** | Git hosting service | `git.$DOMAIN` |
| **GLPI** | IT Asset Management & Ticketing | `support.$DOMAIN` |
| **Mail Server** | Full stack mail server | `mail.$DOMAIN` |
| **Netdata** | Real-time performance monitoring | `netdata.$DOMAIN` |
| **Nextcloud** | File storage and collaboration | `cloud.$DOMAIN` |
| **Portainer** | Docker management UI | `portainer.$DOMAIN` |
| **Uptime Kuma** | Monitoring tool | `status.$DOMAIN` |
| **Vaultwarden** | Bitwarden compatible password manager | `pass.$DOMAIN` |
| **WireGuard** | VPN Server (via WG-Easy) | `vpn.$DOMAIN` |
| **YOURLS** | URL Shortener | `x.$DOMAIN` |

## Installation

### Prerequisites
- **OS:** Debian 11/12 (Recommended) or Ubuntu 20.04/22.04.
- **User:** Root access is required.
- **Domain:** A valid domain name pointing to your server IP.

### Quick Start
```bash
git clone https://github.com/your-repo/cylae-manager.git
cd cylae-manager
./install.sh
```

Follow the on-screen instructions to configure your domain and email.

## Usage

Run the manager anytime using:
```bash
./install.sh
# OR if installed globally
server_manager.sh
```

### Menu Options
- **Manage Services:** Install/Remove individual services.
- **System Update:** Updates OS packages and Docker images.
- **Backup Data:** Creates a backup in `/var/backups/cyl_manager`.
- **Refresh Infrastructure:** Regenerates Nginx configs and SSL certificates.
- **Tune System:** Re-applies system optimizations.
- **DNS Records Info:** Displays required DNS records.
- **Show Credentials:** Displays saved passwords.
- **Hardening:** Manage SSH port and security settings.

## Troubleshooting

- **Logs:** Check `/var/log/server_manager.log` for detailed execution logs.
- **Credentials:** If you forgot a password, use the "Show Credentials" option in the menu or check `/root/.auth_details`.
- **Port Conflicts:** The script checks for port conflicts. If a service fails to start, ensure the port is free using `ss -tuln`.

</div>

<div id="français">

# Gestionnaire de Serveur Cylae (Édition Sécurisée)

**Cylae Server Manager** est une suite de déploiement modulaire, sécurisée et prête pour la production pour les services auto-hébergés. Elle simplifie l'installation, la gestion et la maintenance d'applications populaires utilisant Docker, Nginx et des scripts automatisés.

## Fonctionnalités

- **Architecture Modulaire :** Logique centrale séparée des définitions de services pour une maintenance facile.
- **Sécurité Avant Tout :**
  - Configuration automatisée du pare-feu (UFW).
  - Intégration Fail2Ban pour SSH.
  - Configuration SSH durcie (Connexion Root désactivée, Auth par clé préférée).
  - Gestion sécurisée des identifiants (générés ou fournis par l'utilisateur).
- **Infrastructure Automatisée :**
  - Installation de Docker & Docker Compose (V2).
  - Proxy Inverse Nginx avec SSL Let's Encrypt automatisé.
  - Base de données MariaDB centralisée avec optimisation des performances.
  - Redis pour le cache (Nextcloud).
- **Optimisation Intelligente :**
  - Détection automatique des ressources système (Profil Bas vs Haut).
  - Optimisation du Noyau (BBR, pile TCP), Base de données et PHP.
- **Sauvegardes :** Sauvegarde automatisée des données, bases de données et configurations.
- **Mise à jour Auto :** Mécanisme de mise à jour automatique pour le script de gestion.

## Catalogue de Services

| Service | Description | URL |
| :--- | :--- | :--- |
| **FileBrowser** | Gestionnaire de fichiers Web | `files.$DOMAIN` |
| **FTP** | Serveur vsftpd | `ftp://$DOMAIN` |
| **Gitea** | Service d'hébergement Git | `git.$DOMAIN` |
| **GLPI** | Gestion de Parc Informatique & Tickets | `support.$DOMAIN` |
| **Mail Server** | Serveur mail complet | `mail.$DOMAIN` |
| **Netdata** | Surveillance des performances en temps réel | `netdata.$DOMAIN` |
| **Nextcloud** | Stockage de fichiers et collaboration | `cloud.$DOMAIN` |
| **Portainer** | Interface de gestion Docker | `portainer.$DOMAIN` |
| **Uptime Kuma** | Outil de surveillance | `status.$DOMAIN` |
| **Vaultwarden** | Gestionnaire de mots de passe compatible Bitwarden | `pass.$DOMAIN` |
| **WireGuard** | Serveur VPN (via WG-Easy) | `vpn.$DOMAIN` |
| **YOURLS** | Raccourcisseur d'URL | `x.$DOMAIN` |

## Installation

### Prérequis
- **OS :** Debian 11/12 (Recommandé) ou Ubuntu 20.04/22.04.
- **Utilisateur :** L'accès Root est requis.
- **Domaine :** Un nom de domaine valide pointant vers l'IP de votre serveur.

### Démarrage Rapide
```bash
git clone https://github.com/votre-repo/cylae-manager.git
cd cylae-manager
./install.sh
```

Suivez les instructions à l'écran pour configurer votre domaine et email.

## Utilisation

Lancez le gestionnaire à tout moment avec :
```bash
./install.sh
# OU si installé globalement
server_manager.sh
```

### Options du Menu
- **Gérer les Services :** Installer/Supprimer des services individuels.
- **Mise à jour Système :** Met à jour les paquets OS et les images Docker.
- **Sauvegarde Données :** Crée une sauvegarde dans `/var/backups/cyl_manager`.
- **Rafraîchir Infrastructure :** Régénère les configs Nginx et certificats SSL.
- **Optimiser Système :** Ré-applique les optimisations système.
- **Info Enregistrements DNS :** Affiche les enregistrements DNS requis.
- **Afficher Identifiants :** Affiche les mots de passe enregistrés.
- **Durcissement :** Gérer le port SSH et les paramètres de sécurité.

## Dépannage

- **Logs :** Vérifiez `/var/log/server_manager.log` pour des logs d'exécution détaillés.
- **Identifiants :** Si vous avez oublié un mot de passe, utilisez l'option "Afficher Identifiants" dans le menu ou vérifiez `/root/.auth_details`.
- **Conflits de Port :** Le script vérifie les conflits de port. Si un service ne démarre pas, assurez-vous que le port est libre avec `ss -tuln`.

</div>
