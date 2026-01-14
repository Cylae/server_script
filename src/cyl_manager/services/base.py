from abc import ABC, abstractmethod
from typing import Dict, Any, Optional
import subprocess
import yaml
from pathlib import Path
from ..core.docker import DockerManager
from ..core.system import SystemManager
from ..core.config import settings
from ..core.logging import logger
from ..core.exceptions import ServiceError

class BaseService(ABC):
    name: str = "base_service"
    pretty_name: str = "Base Service"

    def __init__(self):
        self.docker = DockerManager()
        self.system = SystemManager()
        self.profile = self.system.get_hardware_profile()

    @property
    def is_installed(self) -> bool:
        return self.docker.is_installed(self.name)

    def install(self):
        logger.info(f"Installing {self.pretty_name}...")
        self.docker.ensure_network()
        compose_content = self.generate_compose()
        self._deploy_compose(compose_content)
        logger.info(f"{self.pretty_name} installed successfully.")

    def wait_for_health(self, retries=30, delay=2) -> bool:
        """
        Polls the container status to check for health.
        """
        return self.docker.wait_for_health(self.name, retries=retries, delay=delay)

    def remove(self):
        logger.info(f"Removing {self.pretty_name}...")
        self.docker.stop_and_remove(self.name)
        logger.info(f"{self.pretty_name} removed.")

    def _deploy_compose(self, compose_content: Dict[str, Any]):
        """Deploys a docker-compose configuration."""
        compose_dir = Path(settings.DATA_DIR) / "compose" / self.name
        compose_dir.mkdir(parents=True, exist_ok=True)
        compose_path = compose_dir / "docker-compose.yml"

        with open(compose_path, "w") as f:
            yaml.dump(compose_content, f)

        try:
            # Use subprocess to call docker compose as it handles up/down logic well
            cmd = ["docker", "compose", "-f", str(compose_path), "up", "-d"]
            subprocess.run(cmd, check=True, capture_output=True)
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to deploy compose file: {e.stderr.decode()}")
            raise ServiceError(f"Deployment failed for {self.name}")

    def get_common_env(self) -> Dict[str, str]:
        uid, gid = self.system.get_uid_gid()
        return {
            "PUID": uid,
            "PGID": gid,
            "TZ": self.system.get_timezone()
        }

    def get_resource_limits(self, high_mem="1G", high_cpu="0.5", low_mem="512M", low_cpu="0.25") -> Dict[str, Any]:
        """Returns Docker Compose deploy resources based on hardware profile."""
        if self.profile == "HIGH":
            return {
                "resources": {
                    "limits": {
                        "memory": high_mem,
                        "cpus": high_cpu
                    }
                }
            }
        else:
            return {
                "resources": {
                    "limits": {
                        "memory": low_mem,
                        "cpus": low_cpu
                    }
                }
            }

    def is_low_spec(self) -> bool:
        return self.profile == "LOW"

    @abstractmethod
    def generate_compose(self) -> Dict[str, Any]:
        """Generates the Docker Compose dictionary."""
        pass
