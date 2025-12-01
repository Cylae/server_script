# 🚀 CYL.AE Server Manager (v6.0)

> **The Ultimate "Set & Forget" Self-Hosting Solution.**  
> *Performance Edition | Auto-Tuning | Fully Modular*

---

## 🇬🇧 English Version

### What is this?
**CYL.AE Server Manager** is a powerful, all-in-one Bash script designed to turn a fresh Debian/Ubuntu server into a production-ready fortress in minutes. 

It's not just an installer; it's an intelligent **Lifecycle Manager**. It detects your hardware resources to optimize performance, manages your services via Docker, handles SSL certificates automatically, and even updates itself while you sleep.

### ✨ Key Features

*   **🧠 Intelligent Auto-Tuning**: Detects your RAM (Low/High profile) and dynamically tunes MariaDB and PHP-FPM configs for maximum performance or stability.
*   **⚡ TCP BBR Enabled**: Automatically enables Google's BBR congestion control for blazing fast network speeds.
*   **🛡️ Ironclad Security**: Configures UFW firewall, Fail2Ban, and auto-generates strong passwords for every service.
*   **🔄 Zero-Downtime Architecture**: The script is idempotent. Run it as many times as you want; it won't restart services unless necessary.
*   **🤖 Auto-Pilot Mode**: Includes a background cron job that updates the OS, Docker containers (via Watchtower), and SSL certs every night at 4:00 AM.
*   **🔒 SSL Everywhere**: Automatic Let's Encrypt certificates for all your subdomains.

### 🚀 Quick Start

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
| **Gitea** | `git.cyl.ae` | Lightweight, self-hosted Git service (GitHub alternative). |
| **Nextcloud** | `cloud.cyl.ae` | Your personal cloud for files, contacts, and calendar. |
| **Vaultwarden** | `pass.cyl.ae` | Secure password manager (Bitwarden compatible). |
| **Uptime Kuma** | `status.cyl.ae` | Beautiful monitoring dashboard for your services. |
| **Portainer** | `portainer.cyl.ae` | GUI to manage your Docker containers easily. |
| **Netdata** | `netdata.cyl.ae` | Real-time performance monitoring (CPU, RAM, Network). |
| **Mail Server** | `mail.cyl.ae` | Full-stack mail server (Postfix, Dovecot, Roundcube). |
| **YOURLS** | `x.cyl.ae` | Your own URL shortener. |
| **FTP** | N/A | Classic FTP server for legacy file transfer needs. |

### 🛠️ Advanced

*   **Dashboard**: Access `cyl.ae` to see a centralized dashboard of all your installed services.
*   **Backups**: Option 11 performs a full backup (Database SQL dumps + Files) to `/var/backups/cyl_manager`.
*   **Force Re-init**: Option 13 allows you to force a full system re-initialization if you need to reset configurations.

---

## 🇫🇷 Version Française

### C'est quoi ?
**CYL.AE Server Manager** est un script Bash tout-en-un puissant, conçu pour transformer un serveur Debian/Ubuntu vierge en une forteresse de production en quelques minutes.

Ce n'est pas juste un installeur, c'est un **Gestionnaire de Cycle de Vie** intelligent. Il détecte vos ressources matérielles pour optimiser les performances, gère vos services via Docker, s'occupe des certificats SSL automatiquement, et se met même à jour pendant que vous dormez.

### ✨ Fonctionnalités Clés

*   **🧠 Auto-Tuning Intelligent** : Détecte votre RAM (Profil Bas/Haut) et ajuste dynamiquement les configs MariaDB et PHP-FPM pour une performance ou une stabilité maximale.
*   **⚡ TCP BBR Activé** : Active automatiquement l'algorithme de congestion BBR de Google pour une vitesse réseau fulgurante.
*   **🛡️ Sécurité Béton** : Configure le pare-feu UFW, Fail2Ban, et génère automatiquement des mots de passe forts pour chaque service.
*   **🔄 Architecture Zéro-Coupure** : Le script est idempotent. Lancez-le autant de fois que vous voulez ; il ne redémarrera pas les services sauf si nécessaire.
*   **🤖 Mode Pilote Automatique** : Inclut une tâche de fond qui met à jour l'OS, les conteneurs Docker (via Watchtower) et les certificats SSL chaque nuit à 04h00.
*   **🔒 SSL Partout** : Certificats Let's Encrypt automatiques pour tous vos sous-domaines.

### 🚀 Démarrage Rapide

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

*   **Dashboard** : Accédez à `cyl.ae` pour voir un tableau de bord centralisé de tous vos services installés.
*   **Sauvegardes** : L'option 11 effectue une sauvegarde complète (Dumps SQL + Fichiers) dans `/var/backups/cyl_manager`.
*   **Force Re-init** : L'option 13 vous permet de forcer une réinitialisation complète du système si vous avez besoin de remettre les configurations à zéro.

---
*Made with ❤️ for Cylae.*
