# Cylae Server Manager 🚀

![Cylae Banner](https://img.shields.io/badge/Status-Stable-brightgreen?style=for-the-badge) ![Python](https://img.shields.io/badge/Python-3.9%2B-blue?style=for-the-badge&logo=python) ![Docker](https://img.shields.io/badge/Docker-Enabled-blue?style=for-the-badge&logo=docker)

> **The Ultimate Self-Hosted Media & Service Ecosystem Manager.**
> *Robust. Modular. Secure.*

---

## 🇬🇧 English Documentation

### Overview
Cylae Server Manager is a **production-grade** automation framework designed to deploy and manage a complete self-hosted ecosystem (Plex, Sonarr, Radarr, Nextcloud, etc.) on Debian/Ubuntu systems. It leverages **Docker Compose** for isolation and reproducibility, ensuring your server remains clean and stable.

### Key Features
*   **🔌 Plug & Play:** Automated installation of Docker, dependencies, and network setup.
*   **🛡️ Secure by Default:**
    *   Automatic management of `ufw` firewall rules for installed services.
    *   Strict permission management and random password generation.
    *   Non-root container execution where possible.
*   **🧠 Intelligent Hardware Profiling:** Automatically detects system resources (RAM, CPU) and adjusts container limits (`LOW` vs `HIGH` profile).
*   **🔑 Credentials Management:** View access URLs and credentials summary directly from the menu.
*   **⚡ Concurrency Control:** optimized parallel deployment for high-end systems, serial safety for low-end boxes.
*   **📦 Modular Architecture:** Easily extensible Python-based service registry.

### ⚠️ Cloud Providers (GCP, AWS, Azure)
If you are hosting this on a cloud provider like Google Cloud Platform:
1.  **VPC Firewall:** You **must** manually allow ingress traffic on the ports used by your services (e.g., `80`, `443`, `81`, `3000`, `32400`) in your Cloud Console.
2.  **OS Firewall:** This script manages the local `ufw` firewall automatically.

### Installation
Run the following command as root:

```bash
sudo ./install.py
```

This will:
1.  Check for root privileges.
2.  Install system dependencies (Python, Git, Docker, ufw).
3.  Configure basic firewall rules (SSH allowed).
4.  Set up a virtual environment.
5.  Install the CLI tool globally as `cyl-manager`.

### Usage
Once installed, access the interactive menu:

```bash
cyl-manager menu
```

Or use the CLI directly:

```bash
# Install specific service (automatically opens ports)
cyl-manager install plex

# Check status (now includes URLs)
cyl-manager status

# Install everything
cyl-manager install-all
```

**New in v2.1:**
- **Auto-Firewall:** Installing a service automatically opens the required ports in `ufw`.
- **Service Configuration:** Interactive prompts for services like MariaDB.
- **Credentials Summary:** View all your service URLs and initial credentials in the "Service Credentials" menu.
- **URL Display:** Main menu now shows the active URL/Subdomain for running services.

---

## 🇫🇷 Documentation Française

### Vue d'ensemble
Cylae Server Manager est un framework d'automatisation de **niveau production** conçu pour déployer et gérer un écosystème auto-hébergé complet (Plex, Sonarr, Radarr, Nextcloud, etc.) sur des systèmes Debian/Ubuntu. Il utilise **Docker Compose** pour l'isolation et la reproductibilité, garantissant que votre serveur reste propre et stable.

### Fonctionnalités Clés
*   **🔌 Plug & Play :** Installation automatisée de Docker, des dépendances et de la configuration réseau.
*   **🛡️ Sécurisé par Défaut :**
    *   Gestion automatique des règles de pare-feu `ufw` pour les services installés.
    *   Gestion stricte des permissions et génération de mots de passe aléatoires.
*   **🧠 Profilage Matériel Intelligent :** Détecte automatiquement les ressources système (RAM, CPU) et ajuste les limites des conteneurs (profil `LOW` vs `HIGH`).
*   **🔑 Gestion des Identifiants :** Visualisez les URLs d'accès et le résumé des identifiants directement depuis le menu.
*   **⚡ Contrôle de Concurrence :** Déploiement parallèle optimisé pour les systèmes performants, sécurité sérielle pour les machines modestes.
*   **📦 Architecture Modulaire :** Registre de services basé sur Python facilement extensible.

### ⚠️ Fournisseurs Cloud (GCP, AWS, Azure)
Si vous hébergez ceci sur un fournisseur cloud comme Google Cloud Platform :
1.  **Pare-feu VPC :** Vous **devez** autoriser manuellement le trafic entrant sur les ports utilisés par vos services (ex: `80`, `443`, `81`, `3000`, `32400`) dans votre console Cloud.
2.  **Pare-feu OS :** Ce script gère automatiquement le pare-feu local `ufw`.

### Installation
Exécutez la commande suivante en tant que root :

```bash
sudo ./install.py
```

Cela va :
1.  Vérifier les privilèges root.
2.  Installer les dépendances système (Python, Git, Docker, ufw).
3.  Configurer les règles de base du pare-feu (SSH autorisé).
4.  Configurer un environnement virtuel.
5.  Installer l'outil CLI globalement sous le nom `cyl-manager`.

### Utilisation
Une fois installé, accédez au menu interactif :

```bash
cyl-manager menu
```

Ou utilisez directement la CLI :

```bash
# Installer un service spécifique (ouvre automatiquement les ports)
cyl-manager install plex

# Vérifier le statut (inclut maintenant les URLs)
cyl-manager status

# Tout installer
cyl-manager install-all
```

**Nouveauté v2.1 :**
- **Auto-Pare-feu :** L'installation d'un service ouvre automatiquement les ports requis dans `ufw`.
- **Configuration des Services :** Invites interactives pour des services comme MariaDB.
- **Résumé des Identifiants :** Visualisez toutes vos URLs de service et identifiants initiaux dans le menu "Service Credentials".
- **Affichage URL :** Le menu principal affiche maintenant l'URL/Sous-domaine actif pour les services en cours d'exécution.

---

<p align="center">
  Made with ❤️ by the Cylae Team
</p>
