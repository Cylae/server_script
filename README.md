# 🚀 Cylae Server Manager (v8.1 - Optimized Edition)

![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Bash](https://img.shields.io/badge/language-Bash-4EAA25.svg) ![Docker](https://img.shields.io/badge/container-Docker-2496ED.svg) ![Status](https://img.shields.io/badge/status-Optimized-success.svg)

> **The Ultimate "Turnkey" Self-Hosting Solution.**
> *Bazinga! It's Optimized. | Universal Edition | Auto-Tuning | Modular | Secure by Default*

[🇬🇧 English](#english) | [🇫🇷 Français](#français)

---

<div id="english"></div>

## 🌟 Why this script?

You have a fresh VPS (Debian/Ubuntu) and you want to host your own services (Nextcloud, Gitea, Bitwarden/Vaultwarden, VPN...).
Normally, you would spend hours configuring Nginx, installing Docker, securing SSH, creating databases, and managing SSL certificates. And even then, it wouldn't be *perfect*.

**Cylae Server Manager** does it all for you in **minutes**, with a level of precision that would make Sheldon Cooper proud.

### 🔐 Access and Credentials
Once services are installed, you can retrieve **all generated passwords** via the script menu.
1. Run the script: `./install.sh`
2. Choose option **`c`** (SHOW CREDENTIALS).
3. The script will display passwords for Database, Mail (postmaster), WireGuard, etc.

*Note: The raw file is stored in `/root/.auth_details` (root access only).*

### ⚡ Hyper-Optimized Performance
This isn't just a script; it's a finely tuned instrument.
*   **Kernel Tuning:** TCP Fast Open, BBR Congestion Control, and optimized backlog settings for maximum throughput.
*   **Nginx Turbo:** HTTP/2 enabled, Brotli/Gzip compression optimized, OCSP Stapling, and strict HSTS security headers.
*   **Nextcloud Speed:** Automatically configures **Redis** for transactional locking and caching. It flows like a superfluid.
*   **PHP Opcache:** Tuned for production workloads with zero-timestamp validation for speed.

### 🔄 Reinstall & Repair
The script is **idempotent**: you can run it as many times as needed.
*   It detects existing services.
*   It **preserves your passwords** (stored in `.auth_details`).
*   It updates containers and configurations without data loss.

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

---

## 📦 Services Catalog

All services are deployed via **Docker** for maximum isolation and stability, served behind **Nginx** with automatic **Let's Encrypt SSL**.

| Service | Description | URL |
| :--- | :--- | :--- |
| **Gitea** | Lightweight Git hosting (Github alternative). | `https://git.yourdomain.com` |
| **Nextcloud** | File hosting & sharing (Google Drive alternative). | `https://cloud.yourdomain.com` |
| **Vaultwarden** | Password manager (Bitwarden compatible). | `https://pass.yourdomain.com` |
| **Mail Server** | Full stack mail server. Default user: `postmaster@yourdomain.com`. | `https://mail.yourdomain.com` |
| **Uptime Kuma** | Monitoring tool to track uptime of services. | `https://status.yourdomain.com` |
| **WireGuard** | Modern, fast VPN with web UI (wg-easy). | `https://vpn.yourdomain.com` |
| **File Browser** | Web-based file manager. | `https://files.yourdomain.com` |
| **YOURLS** | URL Shortener. | `https://x.yourdomain.com` |
| **Portainer** | GUI for managing Docker containers. | `https://portainer.yourdomain.com` |
| **Netdata** | Real-time performance monitoring. | `https://netdata.yourdomain.com` |
| **FTP** | High performance FTP Server (vsftpd). | `ftp://ftp.yourdomain.com` |
| **GLPI** | IT Asset Management & Ticketing System. | `https://support.yourdomain.com` |

> **Note:** Databases are managed via a centralized MariaDB instance (bare-metal) for performance, accessible via **Adminer** on the dashboard.

---

## ⚙️ Advanced Features

### 🛡️ Security
*   **SSH Hardening**: Option to disable password login and change SSH port.
*   **Firewall**: UFW is configured to deny all incoming traffic except SSH, HTTP/S, and specific service ports.
*   **Isolation**: Docker containers run in a dedicated network.
*   **Updates**: Daily System Updates, Docker Updates (Watchtower), and Self-Updates.

### 📂 Directory Structure
*   **Config**: `/etc/cyl_manager.conf`
*   **Logs**: `/var/log/server_manager.log`
*   **Credentials**: `/root/.auth_details`
*   **Service Data**: `/opt/<service_name>`
*   **Backups**: `/var/backups/cyl_manager`

---

## ❓ Troubleshooting

**Q: I added a service but the URL doesn't work.**
A: Ensure you have created the DNS record (CNAME) for the subdomain. Use option `d` in the menu to see required records. Then run option `r` (Sync All) to refresh Nginx and SSL.

**Q: How do I access the Database?**
A: Go to your main dashboard (`https://admin.yourdomain.com`) and click "DB Admin". Login with `root` and the password found in `/root/.auth_details`.

---
---

<br>

<div id="français"></div>

# 🚀 Cylae Server Manager (v8.1 - Édition Optimisée)

> **La solution d'auto-hébergement "Clé en main" ultime.**
> *Bazinga ! C'est optimisé. | Édition Universelle | Auto-Optimisation | Modulaire | Sécurisé par défaut*

[🇬🇧 English](#english) | [🇫🇷 Français](#français)

---

## 🌟 Pourquoi ce script ?

Vous avez un VPS tout frais (Debian/Ubuntu) et vous voulez héberger vos propres services (Nextcloud, Gitea, Bitwarden/Vaultwarden, VPN...).
Normalement, vous passeriez des heures à configurer Nginx, installer Docker, sécuriser SSH, créer des bases de données et gérer les certificats SSL. Et même là, ce ne serait pas *parfait*.

**Cylae Server Manager** fait tout cela pour vous en **quelques minutes**, avec un niveau de précision qui rendrait Sheldon Cooper fier.

### 🔐 Accès et Identifiants
Une fois les services installés, vous pouvez retrouver **tous les mots de passe générés** via le menu du script.
1. Lancez le script : `./install.sh`
2. Choisissez l'option **`c`** (SHOW CREDENTIALS).
3. Le script affichera les mots de passe pour la Base de données, le Mail (postmaster), WireGuard, etc.

*Note : Le fichier brut est stocké dans `/root/.auth_details` (accessible uniquement en root).*

### ⚡ Performance Hyper-Optimisée
Ce n'est pas juste un script ; c'est un instrument finement réglé.
*   **Kernel Tuning :** TCP Fast Open, BBR Congestion Control, et réglages "backlog" optimisés pour un débit maximal.
*   **Nginx Turbo :** HTTP/2 activé, compression Brotli/Gzip optimisée, OCSP Stapling, et en-têtes de sécurité HSTS stricts.
*   **Nextcloud Speed :** Configure automatiquement **Redis** pour le verrouillage transactionnel et le cache. Ça coule comme un superfluide.
*   **PHP Opcache :** Réglé pour des charges de production avec validation d'horodatage désactivée pour la vitesse.

### 🔄 Réinstallation et Réparation
Le script est **idempotent** : vous pouvez le relancer autant de fois que nécessaire.
*   Il détectera les services existants.
*   Il **préservera vos mots de passe** (stockés dans `.auth_details`).
*   Il mettra à jour les conteneurs et les configurations sans perte de données.

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

---

## 📦 Catalogue de Services

Tous les services sont déployés via **Docker** pour une isolation et une stabilité maximales, servis derrière **Nginx** avec **SSL Let's Encrypt** automatique.

| Service | Description | URL |
| :--- | :--- | :--- |
| **Gitea** | Hébergement Git léger (alternative à Github). | `https://git.votre-domaine.com` |
| **Nextcloud** | Hébergement & partage de fichiers (alternative à Google Drive). | `https://cloud.votre-domaine.com` |
| **Vaultwarden** | Gestionnaire de mots de passe (compatible Bitwarden). | `https://pass.votre-domaine.com` |
| **Serveur Mail** | Serveur mail complet. Utilisateur par défaut : `postmaster@votre-domaine.com`. | `https://mail.votre-domaine.com` |
| **Uptime Kuma** | Outil de surveillance pour suivre la disponibilité des services. | `https://status.votre-domaine.com` |
| **WireGuard** | VPN moderne et rapide avec interface web (wg-easy). | `https://vpn.votre-domaine.com` |
| **File Browser** | Gestionnaire de fichiers web. | `https://files.votre-domaine.com` |
| **YOURLS** | Réducteur d'URL. | `https://x.votre-domaine.com` |
| **Portainer** | Interface graphique pour gérer les conteneurs Docker. | `https://portainer.votre-domaine.com` |
| **Netdata** | Surveillance des performances en temps réel. | `https://netdata.votre-domaine.com` |
| **FTP** | Serveur FTP haute performance (vsftpd). | `ftp://ftp.votre-domaine.com` |
| **GLPI** | Système de tickets et gestion de parc informatique. | `https://support.votre-domaine.com` |

> **Note :** Les bases de données sont gérées via une instance MariaDB centralisée (bare-metal) pour la performance, accessible via **Adminer** sur le tableau de bord.

---

## ⚙️ Fonctionnalités Avancées

### 🛡️ Sécurité
*   **Durcissement SSH** : Option pour désactiver la connexion par mot de passe et changer le port SSH.
*   **Pare-feu** : UFW est configuré pour refuser tout le trafic entrant sauf SSH, HTTP/S, et les ports de services spécifiques.
*   **Isolation** : Les conteneurs Docker tournent dans un réseau dédié.
*   **Mises à jour** : Mises à jour Système, Docker (Watchtower) et Auto-Mises à jour Quotidiennes.

### 📂 Structure des Dossiers
*   **Config** : `/etc/cyl_manager.conf`
*   **Logs** : `/var/log/server_manager.log`
*   **Identifiants** : `/root/.auth_details`
*   **Données de Service** : `/opt/<service_name>`
*   **Sauvegardes** : `/var/backups/cyl_manager`

---

## ❓ Dépannage

**Q : J'ai ajouté un service mais l'URL ne fonctionne pas.**
R : Assurez-vous d'avoir créé l'enregistrement DNS (CNAME) pour le sous-domaine. Utilisez l'option `d` dans le menu pour voir les enregistrements requis. Puis lancez l'option `r` (Sync All) pour rafraîchir Nginx et SSL.

**Q : Comment accéder à la Base de Données ?**
R : Allez sur votre tableau de bord principal (`https://admin.votre-domaine.com`) et cliquez sur "DB Admin". Connectez-vous avec `root` et le mot de passe trouvé dans `/root/.auth_details`.

---
## 🤝 Contribuer
N'hésitez pas à ouvrir des issues ou des pull requests pour rendre ce script encore meilleur !

*v8.1 - Édition Optimisée*
