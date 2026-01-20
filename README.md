# Cylae Server Manager

[![Stability Verified: CLI & Web Interface Tested](https://img.shields.io/badge/Stability-Verified-green)](https://github.com/Cylae/server_script)

Cylae Server Manager is a modular, production-grade Python framework designed to deploy, manage, and monitor a self-hosted media ecosystem (Plex, Sonarr, Radarr, etc.) on Linux servers. It features an intelligent hardware detection system (GDHD) to optimize configurations for your specific hardware, ensuring stability whether you are running on a low-spec VPS or a high-performance dedicated server.

## Features

-   **Modular Architecture:** Clean separation of concerns with a core library and service modules.
-   **Global Dynamic Hardware Detection (GDHD):** Automatically detects hardware resources (CPU, RAM, Swap) and applies optimized configurations (Low/High profiles).
-   **Web Dashboard:** A Flask-based web interface to monitor server status and hardware usage.
-   **CLI Management:** Robust command-line interface for installing, removing, and managing services.
-   **Docker & Docker Compose:** Fully containerized deployments for isolation and ease of management.
-   **Security First:** Automatic firewall configuration (UFW) and non-root execution where possible.

## Installation

### Prerequisites

-   A Debian/Ubuntu-based Linux system.
-   Root (sudo) privileges.
-   Internet connection.

### Quick Start

1.  Clone the repository:
    ```bash
    git clone https://github.com/Cylae/server_script.git
    cd server_script
    ```

2.  Run the installer:
    ```bash
    sudo python3 install.py
    ```
    This will install system dependencies, set up a virtual environment, and create the `cyl-manager` CLI command.

## Usage

### Command Line Interface (CLI)

After installation, you can use the `cyl-manager` command globally:

-   **Interactive Menu:**
    ```bash
    cyl-manager menu
    ```

-   **Install a specific service:**
    ```bash
    cyl-manager install <service_name>
    ```
    Example: `cyl-manager install plex`

-   **Install ALL services:**
    ```bash
    cyl-manager install_all
    ```

-   **Check Status:**
    ```bash
    cyl-manager status
    ```

-   **Remove a service:**
    ```bash
    cyl-manager remove <service_name>
    ```

### Web Dashboard

The web dashboard provides a real-time view of your server's health.

1.  Start the dashboard server:
    ```bash
    sudo python3 server.py
    ```

2.  Access the dashboard in your browser:
    ```
    http://<your-server-ip>:5000
    ```

## Architecture

The codebase is organized into a modular package structure under `src/cyl_manager`:

-   **`core/`**: Core system logic (Hardware detection, Docker management, Config, Firewall).
    -   `system.py`: Implements the GDHD logic.
    -   `orchestrator.py`: Manages concurrent service installations.
-   **`services/`**: Service definitions (Plex, Sonarr, etc.). Each service is a class inheriting from `BaseService`.
-   **`web/`**: The Flask application for the web dashboard.
-   **`ui/`**: TUI components (Rich-based menus).
-   **`cli.py`**: Typer-based CLI entry point.

## Development

1.  Install development dependencies:
    ```bash
    pip install -r requirements.txt
    ```

2.  Run tests:
    ```bash
    pytest
    ```
    This runs the full test suite, including logic verification for GDHD and the web dashboard.

3.  Type checking:
    ```bash
    mypy src/ install.py server.py
    ```

4.  Linting:
    ```bash
    pylint src/cyl_manager install.py server.py
    ```

## License

MIT License
