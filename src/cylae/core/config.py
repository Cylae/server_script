import os
import json
import logging
from typing import Dict, Any

logger = logging.getLogger(__name__)

CONFIG_FILE = os.environ.get("PROFILE_FILE", "/etc/cyl_manager.conf")
AUTH_FILE = "/root/.auth_details"

class ConfigManager:
    def __init__(self):
        self.config = {}
        self.load()

    def load(self):
        """Loads configuration from file. For simplicity, we use shell sourcing or key=value parsing."""
        if os.path.exists(CONFIG_FILE):
            try:
                with open(CONFIG_FILE, 'r') as f:
                    for line in f:
                        if '=' in line and not line.strip().startswith('#'):
                            key, value = line.strip().split('=', 1)
                            # Remove quotes if present
                            value = value.strip('"\'')
                            self.config[key] = value
            except Exception as e:
                logger.error(f"Failed to load config: {e}")

    def get(self, key: str, default: Any = None) -> Any:
        return self.config.get(key, default)

    def set(self, key: str, value: str):
        self.config[key] = value
        self.save()

    def save(self):
        """Saves configuration back to file in key=value format."""
        try:
            with open(CONFIG_FILE, 'w') as f:
                for key, value in self.config.items():
                    f.write(f'{key}="{value}"\n')
        except PermissionError:
            logger.error("Permission denied writing to config file.")

    def ensure_domain(self):
        """Ensures DOMAIN is set."""
        domain = self.get("DOMAIN")
        if not domain:
            print("Configuration required.")
            while not domain:
                domain = input("Enter your domain name (e.g., example.com): ").strip()
            self.set("DOMAIN", domain)
            self.set("EMAIL", f"admin@{domain}") # Default email
            print(f"Domain set to {domain}")
        return domain

config = ConfigManager()
