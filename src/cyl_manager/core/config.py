import os
from pathlib import Path
from pydantic_settings import BaseSettings, SettingsConfigDict

class Settings(BaseSettings):
    DOMAIN: str = "example.com"
    EMAIL: str = "admin@example.com"
    DOCKER_NET: str = "server-net"
    MEDIA_ROOT: str = "/opt/media"

    # Database
    MYSQL_ROOT_PASSWORD: str = ""

    # Paths
    CONFIG_DIR: str = "/etc/cylae"
    DATA_DIR: str = "/opt/cylae"

    model_config = SettingsConfigDict(
        env_file=(".env", "/etc/cylae/.env"),
        env_file_encoding="utf-8",
        extra="ignore"
    )

settings = Settings()

def save_settings(key: str, value: str):
    """Updates a setting in the /etc/cylae/.env file."""
    env_path = Path("/etc/cylae/.env")
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

    env_path.write_text("\n".join(new_lines) + "\n")
