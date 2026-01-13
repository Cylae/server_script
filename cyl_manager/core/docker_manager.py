import docker
import subprocess
import shutil
from typing import Dict, Any, Optional
from .logger import logger
from .config import config

class DockerManager:
    def __init__(self):
        self.client = None
        self._connect()

    def _connect(self):
        try:
            self.client = docker.from_env()
        except docker.errors.DockerException:
            logger.warning("Docker daemon not reachable. Some features will be disabled until Docker is installed.")

    def ensure_docker_installed(self):
        """Installs Docker if not present (Debian/Ubuntu)."""
        if shutil.which("docker"):
            if not self.client:
                self._connect()
            return

        logger.info("Installing Docker...")
        commands = [
            "apt-get update -q",
            "apt-get install -y ca-certificates curl gnupg",
            "mkdir -p /etc/apt/keyrings",
            "curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg",
            "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable\" | tee /etc/apt/sources.list.d/docker.list > /dev/null",
            "apt-get update -q",
            "apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin"
        ]

        for cmd in commands:
            subprocess.run(cmd, shell=True, check=True)

        self._connect()
        logger.info("Docker installed successfully.")

    def ensure_network(self, network_name: str = config.DOCKER_NET):
        if not self.client: return
        try:
            self.client.networks.get(network_name)
        except docker.errors.NotFound:
            logger.info(f"Creating Docker network: {network_name}")
            self.client.networks.create(network_name, driver="bridge")

    def is_service_running(self, container_name: str) -> bool:
        if not self.client: return False
        try:
            container = self.client.containers.get(container_name)
            return container.status == "running"
        except docker.errors.NotFound:
            return False

    def get_container_port(self, container_name: str) -> Optional[int]:
        if not self.client: return None
        try:
            container = self.client.containers.get(container_name)
            # This is a naive check, usually we look at bindings
            # But for conflict detection we might just need to know if it exists
            return None
        except docker.errors.NotFound:
            return None

docker_manager = DockerManager()
