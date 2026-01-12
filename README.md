# Cylae Server Manager

Cylae Server Manager is a powerful, Python-based CLI tool designed to simplify the deployment and management of a self-hosted server ecosystem. It leverages Docker to provide a modular, secure, and easy-to-use platform for hosting various services, including a comprehensive media stack.

## Features

-   **Service Management:** Install, remove, and manage popular self-hosted services with a single click.
-   **Media Stack:** Integrated management for Plex, Sonarr, Radarr, and more.
-   **Security:** Built-in hardening with UFW and Fail2Ban.
-   **Infrastructure:** Automatic Nginx reverse proxy configuration and Let's Encrypt SSL certificates.
-   **Backup & Restore:** (Coming soon)
-   **User Friendly:** Interactive CLI menu.

## Services

-   **Core:** Portainer, Netdata, FileBrowser, Uptime Kuma, WireGuard, Mail Server.
-   **Productivity:** Gitea, Nextcloud, YOURLS, Vaultwarden, GLPI.
-   **Media:** Plex, Tautulli, Sonarr, Radarr, Prowlarr, Jackett, Overseerr, qBittorrent.

## Installation

### Prerequisites

-   A server running **Debian** (Recommended) or **Ubuntu**.
-   **Root access** (sudo is not enough, run as root).
-   A valid domain name pointed to your server's IP.

### Quick Start

1.  Clone the repository:
    ```bash
    git clone https://github.com/Cylae/server_script.git cylae-manager
    cd cylae-manager
    ```

2.  Run the installer:
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

3.  Follow the on-screen instructions to configure your domain and email.

## Usage

After installation, the manager will launch automatically. You can start it anytime by running:

```bash
cyl-manager
```
(Ensure you are in the virtual environment or install it globally)

## Architecture

The project is structured as a Python package `cyl_manager`.
-   `core/`: Core logic for system, docker, security, and config.
-   `services/`: Service definitions and installation logic.
-   `cli.py`: The command-line interface.

---

# Cylae Server Manager (Français)

Cylae Server Manager est un outil CLI puissant basé sur Python, conçu pour simplifier le déploiement et la gestion d'un écosystème de serveur auto-hébergé. Il utilise Docker pour fournir une plateforme modulaire, sécurisée et facile à utiliser pour héberger divers services, y compris une pile multimédia complète.

## Fonctionnalités

-   **Gestion des services :** Installez, supprimez et gérez des services auto-hébergés populaires en un seul clic.
-   **Pile Multimédia :** Gestion intégrée pour Plex, Sonarr, Radarr, et plus encore.
-   **Sécurité :** Durcissement intégré avec UFW et Fail2Ban.
-   **Infrastructure :** Configuration automatique du proxy inverse Nginx et des certificats SSL Let's Encrypt.
-   **Sauvegarde et restauration :** (Bientôt disponible)
-   **Convivial :** Menu CLI interactif.

## Services

-   **Cœur :** Portainer, Netdata, FileBrowser, Uptime Kuma, WireGuard, Serveur Mail.
-   **Productivité :** Gitea, Nextcloud, YOURLS, Vaultwarden, GLPI.
-   **Multimédia :** Plex, Tautulli, Sonarr, Radarr, Prowlarr, Jackett, Overseerr, qBittorrent.

## Installation

### Prérequis

-   Un serveur exécutant **Debian** (Recommandé) ou **Ubuntu**.
-   **Accès root** (sudo ne suffit pas, exécutez en tant que root).
-   Un nom de domaine valide pointant vers l'IP de votre serveur.

### Démarrage rapide

1.  Clonez le dépôt :
    ```bash
    git clone https://github.com/Cylae/server_script.git cylae-manager
    cd cylae-manager
    ```

2.  Lancez l'installateur :
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

3.  Suivez les instructions à l'écran pour configurer votre domaine et votre email.

## Utilisation

Après l'installation, le gestionnaire se lancera automatiquement. Vous pouvez le démarrer à tout moment en exécutant :

```bash
cyl-manager
```

## Architecture

Le projet est structuré comme un paquet Python `cyl_manager`.
-   `core/` : Logique de base pour le système, docker, la sécurité et la configuration.
-   `services/` : Définitions des services et logique d'installation.
-   `cli.py` : L'interface en ligne de commande.
