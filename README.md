# Cylae Server Manager

Cylae Server Manager is a modular, Python-based tool for deploying and managing self-hosted services using Docker.

## Features

- **Modular Architecture:** Services are defined as plugins.
- **Hardware Awareness:** Automatically adjusts resource limits based on system specs (Low/High profile).
- **Docker Integration:** Uses the official Docker Python SDK.
- **Secure:** Generates passwords, manages permissions, and uses `.env` for configuration.
- **Extensible:** Easy to add new services by subclassing `BaseService`.

## Installation

```bash
git clone https://github.com/Cylae/server_script.git cylae-manager
cd cylae-manager
sudo ./install.sh
```

## Usage

Start the CLI:

```bash
cyl-manager
```

### Commands

- `cyl-manager list`: List all available services.
- `cyl-manager install <service_name>`: Install a specific service.
- `cyl-manager remove <service_name>`: Remove a service.
- `cyl-manager info`: Show system hardware information.

## Adding a New Service

Create a new class in `cyl_manager/services/` that inherits from `BaseService` and decorate it with `@registry.register`.

```python
from ..services.base import BaseService
from ..core.registry import registry

@registry.register
class MyService(BaseService):
    @property
    def name(self): return "myservice"

    @property
    def pretty_name(self): return "My Service"

    @property
    def image(self): return "myservice:latest"
```
