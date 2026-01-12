# Cylae Server Manager (CSM)

![Version](https://img.shields.io/badge/Version-9.0-blue) ![Stability](https://img.shields.io/badge/Stability-Production--Grade-green)

A professional, "turnkey" solution for deploying a self-hosted infrastructure on Debian or Ubuntu. Designed for absolute stability, security, and ease of use.

## 🚀 Quick Start (The One-Liner)

Copy and paste this command into your terminal. It handles everything:

```bash
sudo apt update && sudo apt install -y git && cd /opt && sudo git clone https://github.com/Cylae/server_script.git cylae-manager && cd cylae-manager && sudo chmod +x install.sh && sudo ./install.sh
```

---

## 📋 Prerequisites (Pre-Flight Check)

The script now enforces strict resource checks to prevent stability issues.

1.  **A Fresh Server:**
    *   **OS:** Debian 11/12 or Ubuntu 20.04/22.04/24.04.
    *   **Architecture:** x86_64 / amd64.
    *   **Hardware:**
        *   Minimum: 2 vCPU, 2GB RAM (Strict check: <5GB disk space will block installation).
        *   Recommended: 2 vCPU, 4GB RAM (High Performance mode).
2.  **Domain Name:** You must own a domain (e.g., `example.com`) pointing to your server's public IP.
3.  **Root Access:** You need `root` or `sudo` privileges.
4.  **Ports Open:** Ensure ports `80` (HTTP) and `443` (HTTPS) are open in your provider's firewall.

---

## 🛠 Features

*   **Modular Design:** Install only what you need.
*   **Docker-Native:** All services run in isolated containers.
*   **Secure by Default:**
    *   Automatic SSL (Let's Encrypt).
    *   Firewall (UFW) & Fail2Ban configured out-of-the-box.
    *   Kernel hardening and network stack tuning (BBR enabled).
*   **Centralized Management:**
    *   Single Dashboard.
    *   Unified Database (MariaDB).
    *   Automated Backups & Updates.

### Supported Services
| Service | Description | URL |
| :--- | :--- | :--- |
| **Nextcloud** | Cloud storage & collaboration | `cloud.example.com` |
| **Gitea** | Git service (Github alternative) | `git.example.com` |
| **Vaultwarden** | Password manager (Bitwarden) | `pass.example.com` |
| **Portainer** | Docker container management | `portainer.example.com` |
| **Uptime Kuma** | Monitoring & Status Page | `status.example.com` |
| **WireGuard** | Modern VPN | `vpn.example.com` |
| **Mail Server** | Full stack email (Postfix/Dovecot) | `mail.example.com` |
| **FileBrowser** | Web-based file manager | `files.example.com` |
| **GLPI** | IT Asset Management | `support.example.com` |
| **Netdata** | Real-time performance monitoring | `netdata.example.com` |
| **YOURLS** | URL Shortener | `x.example.com` |
| **FTP** | File Transfer Protocol (vsftpd) | `ftp://example.com` |

---

## 📖 Step-by-Step Installation Guide

If you prefer to run commands manually instead of the one-liner:

### 1. Update and Install Git
```bash
sudo apt update
sudo apt install -y git
```

### 2. Clone the Repository
We recommend installing in `/opt/cylae-manager`.
```bash
cd /opt
sudo git clone https://github.com/Cylae/server_script.git cylae-manager
```

### 3. Run the Installer
```bash
cd cylae-manager
sudo chmod +x install.sh
sudo ./install.sh
```

### 4. Follow the Wizard
The script will ask for:
*   **Domain Name:** Enter your domain (e.g., `example.com`).
*   **Profile:** It automatically detects RAM and suggests a profile (Low/High).

---

## ⚙️ Configuration & Maintenance

### Managing Services
Run the script anytime to access the main menu:
```bash
sudo /usr/local/bin/server_manager.sh
```
Or simply:
```bash
cd /opt/cylae-manager && ./install.sh
```

### Credentials
Passwords are generated automatically and stored securely.
*   **View Credentials:** Select option `c. Show Credentials` in the menu.
*   **File Location:** `/root/.auth_details` (Root only).

### Backups (Smart Rotation)
Backups include the database and configuration files.
*   **Location:** `/var/backups/cyl_manager/`
*   **Policy:** Files older than 7 days are automatically deleted.
*   **Compression:** Uses parallel compression (pigz) for speed if available.
*   **Manual Backup:** Select option `b. Backup Data`.

### Updates
*   **System Update:** Select option `s. System Update` (Updates OS, Docker images, and the script).
*   **Auto-Update:** The system updates itself nightly at 04:00 AM.

---

## ❓ Deep Troubleshooting

### "The script fails immediately with exit code 1"
*   **Cause:** Usually permission errors or missing dependencies.
*   **Fix:** Ensure you run with `sudo`. The latest version automatically handles this. Check logs at `/var/log/server_manager.log`.

### "Port 80/443 already in use"
*   **Cause:** Another web server (e.g., default Apache) is running.
*   **Fix:** The script attempts to fix this, but you can manually run: `sudo apt remove apache2 -y`.

### "SSL Certificate Generation Failed"
*   **Cause:** DNS is not propagating or Firewall is blocking.
*   **Fix:**
    1.  Verify your domain points to the server IP: `ping example.com`
    2.  Ensure port 80 is open.
    3.  Run option `r. Refresh Infrastructure` to retry.

### "Docker service not starting"
*   **Cause:** Port conflict or configuration error.
*   **Fix:** Check container logs: `docker logs <container_name>`.

---

## 🏗 Architecture

*   **Core:** Bash scripts in `src/lib/`.
*   **Services:** Modular scripts in `src/services/`.
*   **State:**
    *   Config: `/etc/cyl_manager.conf`
    *   Auth: `/root/.auth_details`
    *   Data: `/opt/<service_name>`
*   **Proxy:** Nginx acts as a reverse proxy, handling SSL termination and routing traffic to Docker containers.

---

# Cylae Server Manager (CSM) [Français]

![Version](https://img.shields.io/badge/Version-9.0-blue) ![Stability](https://img.shields.io/badge/Stabilité-Production--Grade-green)

Une solution professionnelle "clé en main" pour déployer une infrastructure auto-hébergée sur Debian ou Ubuntu. Conçue pour une stabilité absolue, une sécurité renforcée et une facilité d'utilisation.

## 🚀 Démarrage Rapide (La Commande Unique)

Copiez et collez cette commande dans votre terminal. Elle s'occupe de tout :

```bash
sudo apt update && sudo apt install -y git && cd /opt && sudo git clone https://github.com/Cylae/server_script.git cylae-manager && cd cylae-manager && sudo chmod +x install.sh && sudo ./install.sh
```

---

## 📋 Prérequis

Le script impose désormais des vérifications strictes pour garantir la stabilité.

1.  **Un Serveur Frais :**
    *   **OS :** Debian 11/12 ou Ubuntu 20.04/22.04/24.04.
    *   **Architecture :** x86_64 / amd64.
    *   **Matériel :**
        *   Minimum : 2 vCPU, 2 Go RAM (Vérification stricte : <5 Go d'espace disque bloquera l'installation).
        *   Recommandé : 2 vCPU, 4 Go RAM (Mode Haute Performance).
2.  **Nom de Domaine :** Vous devez posséder un domaine (ex: `exemple.com`) pointant vers l'IP publique de votre serveur.
3.  **Accès Root :** Vous devez avoir les privilèges `root` ou `sudo`.
4.  **Ports Ouverts :** Assurez-vous que les ports `80` (HTTP) et `443` (HTTPS) sont ouverts dans le pare-feu de votre fournisseur.

---

## 🛠 Fonctionnalités

*   **Conception Modulaire :** Installez uniquement ce dont vous avez besoin.
*   **Docker-Natif :** Tous les services fonctionnent dans des conteneurs isolés.
*   **Sécurisé par Défaut :**
    *   SSL Automatique (Let's Encrypt).
    *   Pare-feu (UFW) & Fail2Ban configurés dès le départ.
    *   Durcissement du noyau et optimisation de la pile réseau (BBR activé).
*   **Gestion Centralisée :**
    *   Tableau de bord unique.
    *   Base de données unifiée (MariaDB).
    *   Sauvegardes et Mises à jour automatisées.

### Services Supportés
| Service | Description | URL |
| :--- | :--- | :--- |
| **Nextcloud** | Stockage cloud & collaboration | `cloud.exemple.com` |
| **Gitea** | Service Git (Alternative à Github) | `git.exemple.com` |
| **Vaultwarden** | Gestionnaire de mots de passe (Bitwarden) | `pass.exemple.com` |
| **Portainer** | Gestion de conteneurs Docker | `portainer.exemple.com` |
| **Uptime Kuma** | Monitoring & Page de statut | `status.exemple.com` |
| **WireGuard** | VPN Moderne | `vpn.exemple.com` |
| **Mail Server** | Serveur mail complet (Postfix/Dovecot) | `mail.exemple.com` |
| **FileBrowser** | Gestionnaire de fichiers web | `files.exemple.com` |
| **GLPI** | Gestion de Parc Informatique (ITAM) | `support.exemple.com` |
| **Netdata** | Monitoring en temps réel | `netdata.exemple.com` |
| **YOURLS** | Réducteur d'URL | `x.exemple.com` |
| **FTP** | Protocole de transfert de fichiers (vsftpd) | `ftp://exemple.com` |

---

## 📖 Guide d'Installation Étape par Étape

Si vous préférez exécuter les commandes manuellement :

### 1. Mettre à jour et Installer Git
```bash
sudo apt update
sudo apt install -y git
```

### 2. Cloner le Dépôt
Nous recommandons l'installation dans `/opt/cylae-manager`.
```bash
cd /opt
sudo git clone https://github.com/Cylae/server_script.git cylae-manager
```

### 3. Lancer l'Installateur
```bash
cd cylae-manager
sudo chmod +x install.sh
sudo ./install.sh
```

### 4. Suivre l'Assistant
Le script vous demandera :
*   **Nom de Domaine :** Entrez votre domaine (ex: `exemple.com`).
*   **Profil :** Il détecte automatiquement la RAM et suggère un profil (Bas/Haut).

---

## ⚙️ Configuration & Maintenance

### Gérer les Services
Lancez le script à tout moment pour accéder au menu principal :
```bash
sudo /usr/local/bin/server_manager.sh
```
Ou simplement :
```bash
cd /opt/cylae-manager && ./install.sh
```

### Identifiants
Les mots de passe sont générés automatiquement et stockés de manière sécurisée.
*   **Voir les Identifiants :** Sélectionnez l'option `c. Show Credentials` dans le menu.
*   **Emplacement du Fichier :** `/root/.auth_details` (Root uniquement).

### Sauvegardes (Rotation Intelligente)
Les sauvegardes incluent la base de données et les fichiers de configuration.
*   **Emplacement :** `/var/backups/cyl_manager/`
*   **Politique :** Les fichiers de plus de 7 jours sont supprimés automatiquement.
*   **Compression :** Utilise la compression parallèle (pigz) pour la rapidité si disponible.
*   **Sauvegarde Manuelle :** Sélectionnez l'option `b. Backup Data`.

### Mises à Jour
*   **Mise à jour Système :** Sélectionnez l'option `s. System Update` (Met à jour l'OS, les images Docker et le script).
*   **Mise à jour Auto :** Le système se met à jour automatiquement chaque nuit à 04:00 AM.

---

## ❓ Dépannage Approfondi

### "The script fails immediately with exit code 1"
*   **Cause :** Généralement des erreurs de permission ou des dépendances manquantes.
*   **Solution :** Assurez-vous de lancer avec `sudo`. La dernière version gère cela automatiquement. Vérifiez les logs dans `/var/log/server_manager.log`.

### "Port 80/443 already in use"
*   **Cause :** Un autre serveur web (ex: Apache par défaut) est en cours d'exécution.
*   **Solution :** Le script tente de corriger cela, mais vous pouvez lancer manuellement : `sudo apt remove apache2 -y`.

### "SSL Certificate Generation Failed"
*   **Cause :** Le DNS ne s'est pas propagé ou le Pare-feu bloque.
*   **Solution :**
    1.  Vérifiez que votre domaine pointe vers l'IP du serveur : `ping exemple.com`
    2.  Assurez-vous que le port 80 est ouvert.
    3.  Lancez l'option `r. Refresh Infrastructure` pour réessayer.

### "Docker service not starting"
*   **Cause :** Conflit de port ou erreur de configuration.
*   **Solution :** Vérifiez les logs du conteneur : `docker logs <nom_du_conteneur>`.

---

## 🏗 Architecture

*   **Cœur :** Scripts Bash dans `src/lib/`.
*   **Services :** Scripts modulaires dans `src/services/`.
*   **État :**
    *   Config : `/etc/cyl_manager.conf`
    *   Auth : `/root/.auth_details`
    *   Données : `/opt/<nom_du_service>`
*   **Proxy :** Nginx agit comme un reverse proxy, gérant la terminaison SSL et le routage vers les conteneurs Docker.

---

**Auteur :** Équipe Cylae
**Licence :** MIT
