from abc import ABC, abstractmethod
from typing import Dict, Any, Optional
import subprocess
import tempfile
import yaml
import time
import json
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

        # Validation Loop
        try:
            self.wait_for_health()
        except ServiceError as e:
            logger.error(f"Health check failed for {self.name}: {e}")
            # Optionally rollback? For now, we just log and raise.
            raise

        logger.info(f"{self.pretty_name} installed and healthy.")

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

    def wait_for_health(self, timeout=300):
        """Waits for the container to become healthy."""
        logger.info(f"Waiting for {self.name} to become healthy...")
        start_time = time.time()

        while time.time() - start_time < timeout:
            try:
                # Inspect container state
                cmd = ["docker", "inspect", "--format={{json .State}}", self.name]
                result = subprocess.run(cmd, capture_output=True, text=True)

                if result.returncode != 0:
                    # Container might not exist yet if just started?
                    time.sleep(2)
                    continue

                state = json.loads(result.stdout)

                # Check status
                status = state.get("Status")
                health = state.get("Health", {})
                health_status = health.get("Status")

                if status == "running":
                    if health_status:
                        if health_status == "healthy":
                            return True
                        elif health_status == "unhealthy":
                            logger.warning(f"{self.name} is unhealthy.")
                            # Don't return immediately, give it a chance to recover?
                            # Usually once unhealthy it stays so unless something changes.
                            # But let's wait a bit more.
                    else:
                        # No healthcheck defined, assume running is good enough
                        return True
                elif status == "exited" or status == "dead":
                     raise ServiceError(f"Container {self.name} crashed with status {status}")

            except ServiceError:
                raise # Re-raise explicit ServiceErrors (like crash)
            except Exception as e:
                logger.debug(f"Error checking health: {e}")

            time.sleep(5)

        raise ServiceError(f"Timeout waiting for {self.name} to become healthy")

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

    @abstractmethod
    def generate_compose(self) -> Dict[str, Any]:
        """Generates the Docker Compose dictionary."""
        pass
