from typing import Optional
from pathlib import Path
from pydantic_settings import BaseSettings
from dotenv import load_dotenv
import os

# Load .env file from /etc/cylae/ or current directory
ENV_PATH = Path("/etc/cylae/.env")
if not ENV_PATH.exists():
    ENV_PATH = Path(os.getcwd()) / ".env"

load_dotenv(dotenv_path=ENV_PATH)

class Config:
    DOMAIN: str = os.getenv("DOMAIN", "example.com")
    EMAIL: str = os.getenv("EMAIL", "admin@example.com")
    DOCKER_NET: str = os.getenv("DOCKER_NET", "server-net")
    DATA_ROOT: str = os.getenv("DATA_ROOT", "/opt")
    BACKUP_ROOT: str = os.getenv("BACKUP_ROOT", "/var/backups/cylae")
    LOG_LEVEL: str = os.getenv("LOG_LEVEL", "INFO")

    @classmethod
    def save(cls, key: str, value: str):
        """Updates a configuration value in the .env file."""
        # Create directory if it doesn't exist
        if str(ENV_PATH) == "/etc/cylae/.env":
            Path("/etc/cylae").mkdir(parents=True, exist_ok=True)

        lines = []
        if ENV_PATH.exists():
            with open(ENV_PATH, "r") as f:
                lines = f.readlines()

        updated = False
        new_lines = []
        for line in lines:
            if line.startswith(f"{key}="):
                new_lines.append(f"{key}={value}\n")
                updated = True
            else:
                new_lines.append(line)

        if not updated:
            new_lines.append(f"{key}={value}\n")

        with open(ENV_PATH, "w") as f:
            f.writelines(new_lines)

        # Update current environment variable as well
        os.environ[key] = value

config = Config()
