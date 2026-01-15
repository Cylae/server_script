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
*   **🧠 Intelligent Hardware Profiling:** Automatically detects system resources (RAM, CPU) and adjusts container limits (`LOW` vs `HIGH` profile).
*   **🛡️ Secure by Default:** Strict permission management, random password generation, and non-root container execution where possible.
*   **⚡ Concurrency Control:** optimized parallel deployment for high-end systems, serial safety for low-end boxes.
*   **📦 Modular Architecture:** Easily extensible Python-based service registry.

### Installation
Run the following command as root:

```bash
sudo ./install.py
```

This will:
1.  Check for root privileges.
2.  Install system dependencies (Python, Git, Docker).
3.  Set up a virtual environment.
4.  Install the CLI tool globally as `cyl-manager`.

### Usage
Once installed, access the interactive menu:

```bash
cyl-manager menu
```

Or use the CLI directly:

```bash
# Install specific service
cyl-manager install plex

# Check status
cyl-manager status

# Install everything
cyl-manager install-all
```

---

## 🇫🇷 Documentation Française

### Vue d'ensemble
Cylae Server Manager est un framework d'automatisation de **niveau production** conçu pour déployer et gérer un écosystème auto-hébergé complet (Plex, Sonarr, Radarr, Nextcloud, etc.) sur des systèmes Debian/Ubuntu. Il utilise **Docker Compose** pour l'isolation et la reproductibilité, garantissant que votre serveur reste propre et stable.

### Fonctionnalités Clés
*   **🔌 Plug & Play :** Installation automatisée de Docker, des dépendances et de la configuration réseau.
*   **🧠 Profilage Matériel Intelligent :** Détecte automatiquement les ressources système (RAM, CPU) et ajuste les limites des conteneurs (profil `LOW` vs `HIGH`).
*   **🛡️ Sécurisé par Défaut :** Gestion stricte des permissions, génération de mots de passe aléatoires et exécution de conteneurs non-root lorsque c'est possible.
*   **⚡ Contrôle de Concurrence :** Déploiement parallèle optimisé pour les systèmes performants, sécurité sérielle pour les machines modestes.
*   **📦 Architecture Modulaire :** Registre de services basé sur Python facilement extensible.

### Installation
Exécutez la commande suivante en tant que root :

```bash
sudo ./install.py
```

Cela va :
1.  Vérifier les privilèges root.
2.  Installer les dépendances système (Python, Git, Docker).
3.  Configurer un environnement virtuel.
4.  Installer l'outil CLI globalement sous le nom `cyl-manager`.

### Utilisation
Une fois installé, accédez au menu interactif :

```bash
cyl-manager menu
```

Ou utilisez directement la CLI :

```bash
# Installer un service spécifique
cyl-manager install plex

# Vérifier le statut
cyl-manager status

# Tout installer
cyl-manager install-all
```

---

<p align="center">
  Made with ❤️ by the Cylae Team
</p>
