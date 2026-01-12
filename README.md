# Cylae Server Manager

Cylae Server Manager is a robust, modular, and automated tool for deploying and managing a self-hosted media ecosystem using Docker. Originally a Bash script, this version has been completely rewritten in Python for better maintainability, testability, and stability.

## Features

- **Automated Deployment**: Installs Docker, Portainer, Plex, and other services with a single command.
- **Modular Architecture**: Services are defined as independent modules.
- **Secure by Default**: Generates strong passwords and manages permissions correctly.
- **Idempotent**: Can be run multiple times without breaking existing configurations.

## Installation

```bash
git clone https://github.com/Cylae/server_script.git cylae-manager
cd cylae-manager
sudo ./install.sh
```

## Supported Services

- **Portainer**: Docker management UI.
- **Plex**: Media server.
- *(More services are being ported from the legacy Bash version)*

## Development

The project is written in Python 3.

### Running Tests

```bash
python3 -m pytest tests/
```

---

# Cylae Server Manager (Français)

Cylae Server Manager est un outil robuste, modulaire et automatisé pour déployer et gérer un écosystème multimédia auto-hébergé à l'aide de Docker. Initialement un script Bash, cette version a été entièrement réécrite en Python pour une meilleure maintenabilité, testabilité et stabilité.

## Fonctionnalités

- **Déploiement Automatisé**: Installe Docker, Portainer, Plex et d'autres services en une seule commande.
- **Architecture Modulaire**: Les services sont définis comme des modules indépendants.
- **Sécurisé par Défaut**: Génère des mots de passe forts et gère correctement les permissions.
- **Idempotent**: Peut être exécuté plusieurs fois sans casser les configurations existantes.

## Installation

```bash
git clone https://github.com/Cylae/server_script.git cylae-manager
cd cylae-manager
sudo ./install.sh
```

## Services Supportés

- **Portainer**: Interface de gestion Docker.
- **Plex**: Serveur multimédia.
- *(D'autres services sont en cours de portage depuis la version Bash)*
