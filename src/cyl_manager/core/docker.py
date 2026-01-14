import docker
from docker.errors import DockerException, APIError
from .exceptions import ServiceError
from .logging import logger
from .config import settings

class DockerManager:
    _client_instance = None

    def __init__(self):
        # Optimization: Reuse the Docker client connection (Singleton pattern)
        # to avoid expensive re-initialization (socket connection, env parsing)
        # on every service instantiation.
        if DockerManager._client_instance is None:
            try:
                DockerManager._client_instance = docker.from_env()
            except DockerException as e:
                raise ServiceError(f"Could not connect to Docker: {e}")
        self.client = DockerManager._client_instance

    def is_installed(self, container_name: str) -> bool:
        try:
            self.client.containers.get(container_name)
            return True
        except docker.errors.NotFound:
            return False
        except APIError as e:
            logger.error(f"Error checking container {container_name}: {e}")
            return False

    def ensure_network(self):
        try:
            self.client.networks.get(settings.DOCKER_NET)
        except docker.errors.NotFound:
            logger.info(f"Creating Docker network: {settings.DOCKER_NET}")
            self.client.networks.create(settings.DOCKER_NET, driver="bridge")

    def wait_for_health(self, container_name: str, retries=30, delay=2) -> bool:
        """
        Polls the container status to check for health.
        """
        import time
        logger.info(f"Waiting for {container_name} to be healthy...")
        for _ in range(retries):
            try:
                container = self.client.containers.get(container_name)
                # Check for health status if available, otherwise check if running
                if container.status == "running":
                    # If healthcheck is defined, check it
                    health = container.attrs.get("State", {}).get("Health", {}).get("Status")
                    if health:
                        if health == "healthy":
                            logger.info(f"{container_name} is healthy.")
                            return True
                    else:
                        # No healthcheck defined, assume running means healthy
                        logger.info(f"{container_name} is running (no healthcheck).")
                        return True
            except docker.errors.NotFound:
                pass
            except APIError as e:
                logger.warning(f"Error checking health for {container_name}: {e}")

            time.sleep(delay)

        logger.warning(f"Timeout waiting for {container_name} to be healthy.")
        return False

    def stop_and_remove(self, container_name: str):
        try:
            container = self.client.containers.get(container_name)
            logger.info(f"Stopping {container_name}...")
            container.stop()
            logger.info(f"Removing {container_name}...")
            container.remove()
        except docker.errors.NotFound:
            pass
        except APIError as e:
            raise ServiceError(f"Failed to remove {container_name}: {e}")
