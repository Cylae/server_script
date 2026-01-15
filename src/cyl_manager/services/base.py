from abc import ABC, abstractmethod
from typing import Dict, Any, Optional
import subprocess
import yaml
from pathlib import Path
from cyl_manager.core.docker import DockerManager
from cyl_manager.core.system import SystemManager
from cyl_manager.core.config import settings
from cyl_manager.core.logging import logger
from cyl_manager.core.exceptions import ServiceError

class BaseService(ABC):
    """
    Abstract Base Class for all services.
    Enforces a standard interface for installation, removal, and health checking.
    """
    name: str = "base_service"
    pretty_name: str = "Base Service"

    def __init__(self) -> None:
        self.docker = DockerManager()
        self.profile = SystemManager.get_hardware_profile()

    @property
    def is_installed(self) -> bool:
        """
        Checks if the service is currently installed (running or stopped).
        """
        return self.docker.is_installed(self.name)

    def configure(self) -> None:
        """
        Optional hook to interactively prompt the user for configuration.
        This is called before installation.
        """
        pass

    def get_install_summary(self) -> Optional[str]:
        """
        Optional hook to return a summary string after installation.
        Useful for displaying passwords, URLs, or next steps.
        """
        return None

    def get_url(self) -> Optional[str]:
        """
        Optional hook to return the primary URL or Subdomain for the service.
        Used for display in the main menu.
        """
        return None

    def install(self) -> None:
        """
        Installs or updates the service via Docker Compose.
        """
        logger.info(f"Installing {self.pretty_name}...")
        self.docker.ensure_network()
        compose_content = self.generate_compose()
        self._deploy_compose(compose_content)
        logger.info(f"{self.pretty_name} installed successfully.")

    def wait_for_health(self, retries: int = 30, delay: int = 2) -> bool:
        """
        Polls the container status to check for health.

        Args:
            retries: Number of attempts.
            delay: Seconds to wait between attempts.

        Returns:
            bool: True if healthy, False otherwise.
        """
        return self.docker.wait_for_health(self.name, retries=retries, delay=delay)

    def remove(self) -> None:
        """
        Stops and removes the service container.
        """
        logger.info(f"Removing {self.pretty_name}...")
        self.docker.stop_and_remove(self.name)
        logger.info(f"{self.pretty_name} removed.")

    def _deploy_compose(self, compose_content: Dict[str, Any]) -> None:
        """
        Deploys a docker-compose configuration.

        Args:
            compose_content: Dictionary representing the Docker Compose file.

        Raises:
            ServiceError: If deployment fails.
        """
        compose_dir = Path(settings.DATA_DIR) / "compose" / self.name

        # Ensure the directory exists
        try:
            compose_dir.mkdir(parents=True, exist_ok=True)
        except OSError as e:
            raise ServiceError(f"Failed to create directory {compose_dir}: {e}") from e

        compose_path = compose_dir / "docker-compose.yml"

        # Write the compose file
        try:
            with compose_path.open("w", encoding="utf-8") as f:
                yaml.dump(compose_content, f, sort_keys=False)
        except IOError as e:
            raise ServiceError(f"Failed to write compose file for {self.name}: {e}") from e

        try:
            # Use subprocess to call docker compose as it handles up/down logic well
            # 'docker compose' (v2) is preferred over 'docker-compose'
            cmd = ["docker", "compose", "-f", str(compose_path), "up", "-d"]

            # Using SystemManager.run_command which is cleaner and logged
            SystemManager.run_command(cmd, check=True)

        except subprocess.CalledProcessError as e:
            error_msg = e.stderr if e.stderr else str(e)
            # Ensure error_msg is a string before logging/raising
            if isinstance(error_msg, bytes):
                error_msg = error_msg.decode('utf-8', errors='replace')

            logger.error(f"Failed to deploy compose file: {error_msg}")
            raise ServiceError(f"Deployment failed for {self.name}: {error_msg}")

    def get_common_env(self) -> Dict[str, str]:
        """
        Returns common environment variables (PUID, PGID, TZ).
        """
        uid, gid = SystemManager.get_uid_gid()
        return {
            "PUID": uid,
            "PGID": gid,
            "TZ": SystemManager.get_timezone()
        }

    def get_resource_limits(
        self,
        high_mem: str = "1G",
        high_cpu: str = "0.5",
        low_mem: str = "512M",
        low_cpu: str = "0.25"
    ) -> Dict[str, Any]:
        """
        Returns Docker Compose deploy resources based on hardware profile.

        Args:
            high_mem: Memory limit for HIGH profile.
            high_cpu: CPU limit for HIGH profile.
            low_mem: Memory limit for LOW profile.
            low_cpu: CPU limit for LOW profile.

        Returns:
            Dict[str, Any]: Docker Compose deploy block.
        """
        if self.profile == SystemManager.PROFILE_HIGH:
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
        """Checks if the current system is running on a low specification profile."""
        return self.profile == SystemManager.PROFILE_LOW

    @abstractmethod
    def generate_compose(self) -> Dict[str, Any]:
        """
        Generates the Docker Compose dictionary.

        Returns:
            Dict[str, Any]: The Docker Compose configuration.
        """
        pass
