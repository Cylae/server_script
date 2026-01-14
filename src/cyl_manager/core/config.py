import os
import threading
from pathlib import Path
from typing import Optional, List

from pydantic_settings import BaseSettings, SettingsConfigDict
from pydantic import Field

# Thread-safe lock for configuration updates
_CONFIG_LOCK = threading.Lock()

def _get_env_file_path() -> Path:
    """Determine the path to the environment file."""
    env_path = os.getenv("CYLAE_ENV_FILE", "/etc/cylae/.env")
    return Path(env_path)

ENV_FILE_PATH = _get_env_file_path()

class Settings(BaseSettings):
    """
    Application configuration settings managed via Pydantic.
    """
    # General Settings
    DOMAIN: str = Field(default="example.com", description="Main domain for the server")
    EMAIL: str = Field(default="admin@example.com", description="Administrator email")
    DOCKER_NET: str = Field(default="server-net", description="Docker network name")
    MEDIA_ROOT: str = Field(default="/opt/media", description="Root directory for media files")

    # Database Configuration
    MYSQL_ROOT_PASSWORD: str = Field(default="", description="Root password for MariaDB")
    MYSQL_USER_PASSWORD: str = Field(default="", description="User password for MariaDB")

    # Application Paths
    CONFIG_DIR: str = Field(default="/etc/cylae", description="Directory for configuration files")
    DATA_DIR: str = Field(default="/opt/cylae", description="Directory for application data")

    model_config = SettingsConfigDict(
        env_file=(".env", str(ENV_FILE_PATH)),
        env_file_encoding="utf-8",
        extra="ignore",
        case_sensitive=True
    )

class SettingsProxy:
    """
    A proxy to ensure that we always access the latest settings instance,
    even if the configuration is reloaded at runtime.
    """
    def __init__(self) -> None:
        self._settings = Settings()

    def reload(self) -> None:
        """Reloads the underlying settings object from environment/files."""
        self._settings = Settings()

    def __getattr__(self, name: str):
        return getattr(self._settings, name)

    # Allow updating attributes directly on the proxy (not recommended vs save_settings but supported)
    def __setattr__(self, name: str, value):
        if name == "_settings":
            super().__setattr__(name, value)
        else:
            setattr(self._settings, name, value)

# Global settings proxy instance
settings = SettingsProxy()

def reload_settings() -> None:
    """Reloads the settings from the environment file."""
    settings.reload()

def save_settings(key: str, value: str, env_path: Optional[Path] = None) -> None:
    """
    Updates a specific setting in the environment file safely.

    Args:
        key: The configuration key to update.
        value: The new value for the configuration key.
        env_path: Optional path to the environment file. Defaults to ENV_FILE_PATH.
    """
    target_path = env_path if env_path is not None else ENV_FILE_PATH

    with _CONFIG_LOCK:
        # Ensure parent directory exists
        if not target_path.parent.exists():
            target_path.parent.mkdir(parents=True, exist_ok=True)

        lines: List[str] = []
        if target_path.exists():
            try:
                lines = target_path.read_text(encoding="utf-8").splitlines()
            except IOError:
                # Log warning or handle error appropriately in a real app
                pass

        updated = False
        new_lines: List[str] = []

        # Parse and update existing lines
        for line in lines:
            stripped_line = line.strip()
            # Handle comments and empty lines
            if not stripped_line or stripped_line.startswith("#"):
                new_lines.append(line)
                continue

            # Simple parsing for KEY=VALUE
            if "=" in line:
                k, _ = line.split("=", 1)
                if k.strip() == key:
                    new_lines.append(f"{key}={value}")
                    updated = True
                else:
                    new_lines.append(line)
            else:
                new_lines.append(line)

        # Append new key if not found
        if not updated:
            new_lines.append(f"{key}={value}")

        # Write back to file atomically-ish (direct write for now as per requirements)
        try:
            target_path.write_text("\n".join(new_lines) + "\n", encoding="utf-8")
        except IOError as e:
            raise RuntimeError(f"Failed to write configuration to {target_path}: {e}") from e

        # Update environment variable for the current process
        os.environ[key] = value

        # Reload the global settings proxy
        reload_settings()
