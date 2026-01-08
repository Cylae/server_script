# 🚀 Cylae Server Manager (v7.1)

![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Bash](https://img.shields.io/badge/language-Bash-4EAA25.svg) ![Docker](https://img.shields.io/badge/container-Docker-2496ED.svg) ![Status](https://img.shields.io/badge/status-Stable-success.svg)

> **La solution d'auto-hébergement "Clé en main" ultime.**
> *Édition Universelle | Auto-Optimisation | Modulaire | Sécurisé par défaut*

[🇫🇷 Français](#-pourquoi-ce-script) | [🇬🇧 English](#english)

---

## 🌟 Pourquoi ce script ?

Vous avez un VPS tout frais (Debian/Ubuntu) et vous voulez héberger vos propres services (Nextcloud, Gitea, Bitwarden/Vaultwarden, VPN...).
Normalement, vous passeriez des heures à configurer Nginx, installer Docker, sécuriser SSH, créer des bases de données et gérer les certificats SSL.

**Cylae Server Manager** fait tout cela pour vous en **quelques minutes**.

### 🔐 Accès et Identifiants (Interactif)
Lors de l'installation de chaque service, le script vous demandera de définir vos propres **nom d'utilisateur et mot de passe administrateur**.
*   Si vous laissez les champs vides, des identifiants sécurisés par défaut seront générés.
*   Vous pouvez retrouver **tous les mots de passe** via le menu du script (Option **`c`**).

### 🔄 Réinstallation et Réparation
Le script est **idempotent** : vous pouvez le relancer autant de fois que nécessaire (par exemple, pour mettre à jour une configuration ou réparer un service).
*   Il détectera les services existants.
*   Il **préservera vos mots de passe** (stockés dans `.auth_details`).
*   Il mettra à jour les conteneurs et les configurations sans perte de données.
*   Il vérifie automatiquement l'intégrité des services après installation.

Il est conçu pour être **Le Meilleur Script JAMAIS conçu** :
*   **Intelligent** : Détecte votre matériel (RAM) et optimise la config MySQL/PHP en conséquence.
*   **Modulaire** : Installez/Supprimez des services proprement sans laisser de résidus.
*   **Sécurisé** : Durcit SSH, configure le Pare-feu (UFW), et configure Nginx avec des en-têtes de sécurité.
*   **SSL Automatique** : Configure automatiquement **Let's Encrypt** pour tous vos domaines (Option **`r`**).
*   **Automatisé** : Se met à jour, met à jour votre système et vos conteneurs docker chaque nuit.
*   **Unifié** : Fournit un tableau de bord magnifique pour accéder à tous vos services.

---

## 🛠️ Installation

**Prérequis :**
*   Un serveur frais **Debian 11/12** (Recommandé) ou **Ubuntu 20.04/22.04**.
*   Accès Root.
*   Un nom de domaine pointant vers l'IP de votre serveur.

### Démarrage Rapide

1.  **Cloner le dépôt :**
    ```bash
    git clone https://github.com/your-username/cylae-server-manager.git /opt/cylae-manager
    cd /opt/cylae-manager
    ```

2.  **Lancer le script :**
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

3.  **Suivre l'assistant :**
    *   Entrez votre nom de domaine.
    *   Sélectionnez les services à installer depuis le menu.
    *   Définissez vos identifiants administrateur.

---

## 📦 Catalogue de Services

Tous les services sont déployés via **Docker** pour une isolation et une stabilité maximales, servis derrière **Nginx** avec **SSL Let's Encrypt** automatique.

| Service | Description | URL |
| :--- | :--- | :--- |
| **Gitea** | Hébergement Git léger (alternative à Github). | `https://git.votre-domaine.com` |
| **Nextcloud** | Hébergement & partage de fichiers (alternative à Google Drive). | `https://cloud.votre-domaine.com` |
| **Vaultwarden** | Gestionnaire de mots de passe (compatible Bitwarden). | `https://pass.votre-domaine.com` |
| **Serveur Mail** | Serveur mail complet. | `https://mail.votre-domaine.com` |
| **Uptime Kuma** | Outil de surveillance pour suivre la disponibilité des services. | `https://status.votre-domaine.com` |
| **WireGuard** | VPN moderne et rapide avec interface web (wg-easy). | `https://vpn.votre-domaine.com` |
| **File Browser** | Gestionnaire de fichiers web. | `https://files.votre-domaine.com` |
| **YOURLS** | Réducteur d'URL. | `https://x.votre-domaine.com` |
| **Portainer** | Interface graphique pour gérer les conteneurs Docker. | `https://portainer.votre-domaine.com` |
| **Netdata** | Surveillance des performances en temps réel. | `https://netdata.votre-domaine.com` |

> **Note :** Les bases de données sont gérées via une instance MariaDB centralisée (bare-metal) pour la performance, accessible via **Adminer** sur le tableau de bord.

---

## ⚙️ Fonctionnalités Avancées

### 🧠 Auto-Optimisation Intelligente
Le script vérifie votre RAM à chaque exécution :
*   **Profil Bas (< 4Go)** : Optimise pour la stabilité. Réduit les buffers de base de données et les workers PHP.
*   **Profil Haut (>= 4Go)** : Optimise pour la vitesse. Augmente la taille des caches et les limites de connexion.

### 🛡️ Sécurité
*   **Durcissement SSH** : Option pour désactiver la connexion par mot de passe et changer le port SSH.
*   **Pare-feu** : UFW est configuré pour refuser tout le trafic entrant sauf SSH, HTTP/S, et les ports de services spécifiques.
*   **Isolation** : Les conteneurs Docker tournent dans un réseau dédié.
*   **Mises à jour** :
    *   Mises à jour Système Quotidiennes (`apt-get upgrade`)
    *   Mises à jour Docker Quotidiennes (`Watchtower`)
    *   Auto-Mises à jour Quotidiennes (`git pull`)

### 📂 Structure des Dossiers
*   **Config** : `/etc/cyl_manager.conf`
*   **Logs** : `/var/log/server_manager.log`
*   **Identifiants** : `/root/.auth_details` (Contient les mots de passe générés)
*   **Données de Service** : `/opt/<service_name>`
*   **Sauvegardes** : `/var/backups/cyl_manager`

---

## ❓ Dépannage

**Q : J'ai ajouté un service mais l'URL ne fonctionne pas.**
R : Assurez-vous d'avoir créé l'enregistrement DNS (CNAME) pour le sous-domaine. Utilisez l'option `d` dans le menu pour voir les enregistrements requis. Puis lancez l'option **`r`** (Sync Infrastructure) pour configurer **SSL (Let's Encrypt)**.

**Q : Comment accéder à la Base de Données ?**
R : Allez sur votre tableau de bord principal (`https://admin.votre-domaine.com`) et cliquez sur "DB Admin". Connectez-vous avec `root` et le mot de passe trouvé dans `/root/.auth_details`.

**Q : Le script a échoué pendant l'installation.**
R : Vérifiez les logs dans `/var/log/server_manager.log` pour des messages d'erreur détaillés.

**Q : Comment restaurer une sauvegarde ?**
R : Les sauvegardes sont stockées dans `/var/backups/cyl_manager`.
*   **Base de données** : `mysql < db_backup.sql`
*   **Fichiers** : Extrayez l'archive vers la racine : `tar -xzf files_backup.tar.gz -C /`

---

## 🤝 Contribuer
N'hésitez pas à ouvrir des issues ou des pull requests pour rendre ce script encore meilleur !

*v7.1 - Édition Ultime*

---
---

<br>

<div id="english"></div>

# 🚀 Cylae Server Manager (v7.1)

> **The Ultimate "Turnkey" Self-Hosting Solution.**
> *Universal Edition | Auto-Tuning | Modular | Secure by Default*

[🇫🇷 Français](#-pourquoi-ce-script) | [🇬🇧 English](#english)

---

## 🌟 Why this script?

You have a fresh VPS (Debian/Ubuntu) and you want to host your own services (Nextcloud, Gitea, Bitwarden/Vaultwarden, VPN...).
Normally, you would spend hours configuring Nginx, setting up Docker, securing SSH, creating databases, and managing SSL certificates.

**Cylae Server Manager** does it all for you in **minutes**.

### 🔐 Access and Credentials (Interactive)
When installing each service, the script will ask you to set your own **admin username and password**.
*   If you leave them empty, secure defaults will be generated.
*   You can retrieve **all passwords** via the script menu (Option **`c`**).

### 🔄 Reinstall & Repair
The script is **idempotent**: you can run it as many times as needed (e.g., to update configuration or repair a service).
*   It detects existing services.
*   It **preserves your passwords** (stored in `.auth_details`).
*   It updates containers and configurations without data loss.
*   It automatically verifies service integrity after installation.

It is designed to be **The Best Script EVER**:
*   **Intelligent**: Detects your hardware (RAM) and tunes MySQL/PHP config accordingly.
*   **Modular**: Install/Remove services cleanly without leaving junk behind.
*   **Secure**: Hardens SSH, configures Firewall (UFW), and sets up Nginx with security headers.
*   **Automatic SSL**: Automatically configures **Let's Encrypt** for all your domains (Option **`r`**).
*   **Automated**: Updates itself, your system, and your docker containers every night.
*   **Unified**: Provides a beautiful dashboard to access all your services.

---

## 🛠️ Installation

**Prerequisites:**
*   A fresh **Debian 11/12** (Recommended) or **Ubuntu 20.04/22.04** server.
*   Root access.
*   A domain name pointing to your server IP.

### Quick Start

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/cylae-server-manager.git /opt/cylae-manager
    cd /opt/cylae-manager
    ```

2.  **Run the script:**
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

3.  **Follow the wizard:**
    *   Enter your domain name.
    *   Select services to install from the menu.
    *   Define your admin credentials.

---

## 📦 Services Catalog

All services are deployed via **Docker** for maximum isolation and stability, served behind **Nginx** with automatic **Let's Encrypt SSL**.

| Service | Description | URL |
| :--- | :--- | :--- |
| **Gitea** | Lightweight Git hosting (Github alternative). | `https://git.yourdomain.com` |
| **Nextcloud** | File hosting & sharing (Google Drive alternative). | `https://cloud.yourdomain.com` |
| **Vaultwarden** | Password manager (Bitwarden compatible). | `https://pass.yourdomain.com` |
| **Mail Server** | Full stack mail server. | `https://mail.yourdomain.com` |
| **Uptime Kuma** | Monitoring tool to track uptime of services. | `https://status.yourdomain.com` |
| **WireGuard** | Modern, fast VPN with web UI (wg-easy). | `https://vpn.yourdomain.com` |
| **File Browser** | Web-based file manager. | `https://files.yourdomain.com` |
| **YOURLS** | URL Shortener. | `https://x.yourdomain.com` |
| **Portainer** | GUI for managing Docker containers. | `https://portainer.yourdomain.com` |
| **Netdata** | Real-time performance monitoring. | `https://netdata.yourdomain.com` |

> **Note:** Databases are managed via a centralized MariaDB instance (bare-metal) for performance, accessible via **Adminer** on the dashboard.

---

## ⚙️ Advanced Features

### 🧠 Smart Auto-Tuning
The script checks your RAM on every run:
*   **Low Profile (< 4GB)**: Optimizes for stability. Reduces database buffers and PHP workers.
*   **High Profile (>= 4GB)**: Optimizes for speed. Increases cache sizes and connection limits.

### 🛡️ Security
*   **SSH Hardening**: Option to disable password login and change SSH port.
*   **Firewall**: UFW is configured to deny all incoming traffic except SSH, HTTP/S, and specific service ports.
*   **Isolation**: Docker containers run in a dedicated network.
*   **Updates**:
    *   Daily System Updates (`apt-get upgrade`)
    *   Daily Docker Updates (`Watchtower`)
    *   Daily Self-Updates (`git pull`)

### 📂 Directory Structure
*   **Config**: `/etc/cyl_manager.conf`
*   **Logs**: `/var/log/server_manager.log`
*   **Credentials**: `/root/.auth_details` (Contains generated passwords)
*   **Service Data**: `/opt/<service_name>`
*   **Backups**: `/var/backups/cyl_manager`

---

## ❓ Troubleshooting

**Q: I added a service but the URL doesn't work.**
A: Ensure you have created the DNS record (CNAME) for the subdomain. Use option `d` in the menu to see required records. Then run option **`r`** (Sync Infrastructure) to configure **SSL (Let's Encrypt)**.

**Q: How do I access the Database?**
A: Go to your main dashboard (`https://admin.yourdomain.com`) and click "DB Admin". Login with `root` and the password found in `/root/.auth_details`.

**Q: The script failed during installation.**
A: Check the logs at `/var/log/server_manager.log` for detailed error messages.

**Q: How do I restore a backup?**
A: Backups are stored in `/var/backups/cyl_manager`.
*   **Database**: `mysql < db_backup.sql`
*   **Files**: Extract the tarball to root: `tar -xzf files_backup.tar.gz -C /`

---

## 🤝 Contributing
Feel free to open issues or pull requests to make this script even better!

*v7.1 - Ultimate Edition*
