import docker
from docker.errors import DockerException, APIError
from .exceptions import ServiceError
from .logging import logger
from .config import settings

class DockerManager:
    def __init__(self):
        try:
            self.client = docker.from_env()
        except DockerException as e:
            raise ServiceError(f"Could not connect to Docker: {e}")

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
