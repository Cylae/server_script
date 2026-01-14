import os
import threading
from pathlib import Path
from pydantic_settings import BaseSettings, SettingsConfigDict

ENV_FILE_PATH = Path(os.getenv("CYLAE_ENV_FILE", "/etc/cylae/.env"))
_CONFIG_LOCK = threading.Lock()

class Settings(BaseSettings):
    DOMAIN: str = "example.com"
    EMAIL: str = "admin@example.com"
    DOCKER_NET: str = "server-net"
    MEDIA_ROOT: str = "/opt/media"

    # Database
    MYSQL_ROOT_PASSWORD: str = ""
    MYSQL_USER_PASSWORD: str = ""

    # Paths
    CONFIG_DIR: str = "/etc/cylae"
    DATA_DIR: str = "/opt/cylae"

    model_config = SettingsConfigDict(
        env_file=(".env", str(ENV_FILE_PATH)),
        env_file_encoding="utf-8",
        extra="ignore"
    )

settings = Settings()

def save_settings(key: str, value: str, env_path: Path = None):
    """Updates a setting in the environment file."""
    if env_path is None:
        env_path = ENV_FILE_PATH

    with _CONFIG_LOCK:
        env_path.parent.mkdir(parents=True, exist_ok=True)

        lines = []
        if env_path.exists():
            lines = env_path.read_text().splitlines()

        updated = False
        new_lines = []
        for line in lines:
            if line.startswith(f"{key}="):
                new_lines.append(f"{key}={value}")
                updated = True
            else:
                new_lines.append(line)

        if not updated:
            new_lines.append(f"{key}={value}")

        env_path.write_text("\n".join(new_lines) + "\n", encoding="utf-8")

        # Also update the in-memory settings
        os.environ[key] = value

        # Reload settings object
        # Since pydantic settings are immutable by default, we just re-instantiate the global object
        # Note: This only affects this process.
        global settings
        settings = Settings()
