# Cylae Server Manager

A modular, robust, and modern framework for deploying and managing a self-hosted media and service ecosystem using Docker.

## Features

*   **Modular Architecture:** Services are defined as independent modules.
*   **Dynamic Configuration:** Uses environment variables and `.env` files.
*   **Hardware Awareness:** Automatically detects system resources and adjusts service profiles.
*   **Media Stack:** Plex, *Arr suite, Overseerr, etc.
*   **Utility Stack:** Portainer, Gitea, Nextcloud, WireGuard, and more.
*   **Modern CLI:** Built with `typer` and `rich` for a great user experience.
*   **Interactive Menu:** Easy-to-use TUI for managing services.

## Installation

### Prerequisites

*   Debian or Ubuntu based system.
*   Root privileges (required for Docker management).

### One-Step Install

```bash
git clone https://github.com/Cylae/server_script.git
cd server_script
sudo ./install.sh
```

## Usage

Once installed, you can access the CLI using the `cyl-manager` command.

### Interactive Menu

The easiest way to manage your server is via the interactive menu:

```bash
sudo cyl-manager menu
```

### CLI Commands

You can also use specific commands for automation or quick actions:

```bash
# List all services and their status
sudo cyl-manager status

# Install a specific service
sudo cyl-manager install plex
sudo cyl-manager install sonarr

# Remove a service
sudo cyl-manager remove portainer
```

## Configuration

Configuration is stored in `/etc/cylae/.env`. Key settings include:

*   `DOMAIN`: Your main domain name.
*   `EMAIL`: Admin email for SSL notifications.
*   `MEDIA_ROOT`: Path to media files (default: `/opt/media`).

## Development

To contribute or modify the code:

1.  Clone the repository.
2.  Install dependencies: `pip install -e .[dev]`
3.  Run tests: `pytest`

## License

MIT License
