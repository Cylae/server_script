from typing import Optional, Final
import time
import threading
import docker
from docker.errors import DockerException, APIError
from docker.models.containers import Container

from cyl_manager.core.exceptions import ServiceError
from cyl_manager.core.logging import logger
from cyl_manager.core.config import settings

class DockerManager:
    """
    Singleton manager for Docker client interactions.
    Handles container queries, network management, and health checks.
    """
    _instance: Optional['DockerManager'] = None
    _client: Optional[docker.DockerClient] = None
    _init_lock = threading.Lock()
    _network_lock = threading.Lock()

    def __new__(cls) -> 'DockerManager':
        if cls._instance is None:
            with cls._init_lock:
                if cls._instance is None:
                    cls._instance = super(DockerManager, cls).__new__(cls)
                    try:
                        # Initialize the client only once
                        cls._client = docker.from_env()
                    except DockerException as e:
                        # Critical error, cannot proceed without Docker
                        raise ServiceError(f"Could not connect to Docker Daemon: {e}. Is Docker running?") from e
        return cls._instance

    @property
    def client(self) -> docker.DockerClient:
        """Safe access to the Docker client instance."""
        if self._client is None:
             # Should be caught in __new__, but for type safety:
             raise ServiceError("Docker client is not initialized.")
        return self._client

    def is_installed(self, container_name: str) -> bool:
        """
        Checks if a container exists (running or stopped).

        Args:
            container_name: The name of the container to check.

        Returns:
            bool: True if container exists, False otherwise.
        """
        try:
            self.client.containers.get(container_name)
            return True
        except docker.errors.NotFound:
            return False
        except APIError as e:
            logger.error(f"Docker API Error checking container '{container_name}': {e}")
            return False

    def ensure_network(self) -> None:
        """
        Ensures the configured Docker network exists.
        Thread-safe to prevent race conditions during parallel orchestration.
        """
        network_name = settings.DOCKER_NET

        with self._network_lock:
            try:
                self.client.networks.get(network_name)
            except docker.errors.NotFound:
                logger.info(f"Creating Docker network: {network_name}")
                try:
                    self.client.networks.create(network_name, driver="bridge")
                except APIError as e:
                    # Double check in case of race condition despite lock (e.g. external process)
                    try:
                        self.client.networks.get(network_name)
                    except docker.errors.NotFound:
                        raise ServiceError(f"Failed to create network '{network_name}': {e}") from e

    def wait_for_health(self, container_name: str, retries: int = 30, delay: int = 2) -> bool:
        """
        Polls the container status to check for health.

        Args:
            container_name: Name of the container.
            retries: Number of retries.
            delay: Delay in seconds between retries.

        Returns:
            bool: True if healthy or running (if no healthcheck), False on timeout/error.
        """
        logger.info(f"Waiting for '{container_name}' to become healthy...")

        for attempt in range(retries):
            try:
                container: Container = self.client.containers.get(container_name)

                if container.status == "running":
                    # Check for healthcheck status if available
                    state = container.attrs.get("State", {})
                    health = state.get("Health", {}).get("Status")

                    if health:
                        if health == "healthy":
                            logger.info(f"Container '{container_name}' is healthy.")
                            return True
                        # If starting or unhealthy, continue waiting
                    else:
                        # No healthcheck defined, assume running means healthy
                        logger.info(f"Container '{container_name}' is running (no healthcheck defined).")
                        return True

            except docker.errors.NotFound:
                # Container might not be created yet, continue waiting
                pass
            except APIError as e:
                logger.warning(f"Docker API error checking health for '{container_name}': {e}")

            time.sleep(delay)

        logger.warning(f"Timeout waiting for '{container_name}' to be healthy after {retries * delay} seconds.")
        return False

    def stop_and_remove(self, container_name: str) -> None:
        """
        Stops and removes a container if it exists.

        Args:
            container_name: The name of the container to remove.
        """
        try:
            container: Container = self.client.containers.get(container_name)
            logger.info(f"Stopping container '{container_name}'...")
            container.stop()
            logger.info(f"Removing container '{container_name}'...")
            container.remove()
        except docker.errors.NotFound:
            pass # Already gone
        except APIError as e:
            raise ServiceError(f"Failed to remove container '{container_name}': {e}") from e
