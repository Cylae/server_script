from abc import ABC, abstractmethod
from typing import Dict, Any, Optional
from pathlib import Path
import os
import subprocess
from jinja2 import Template
from ..core.config import config
from ..core.logger import logger
from ..core.docker_manager import docker_manager
from ..core.system import determine_profile, get_uid_gid, get_timezone
from ..core.proxy import proxy_manager
from ..core.security import security_manager

class BaseService(ABC):
    def __init__(self):
        # OPTIMIZATION: Move heavy system checks to properties or lazy loader
        self._profile = None
        self._uid = None
        self._gid = None
        self._tz = None

    @property
    def profile(self):
        if self._profile is None:
            self._profile = determine_profile()
        return self._profile

    @property
    def uid(self):
        if self._uid is None:
            self._uid, self._gid = get_uid_gid()
        return self._uid

    @property
    def gid(self):
        if self._gid is None:
            self._uid, self._gid = get_uid_gid()
        return self._gid

    @property
    def tz(self):
        if self._tz is None:
            self._tz = get_timezone()
        return self._tz

    @property
    @abstractmethod
    def name(self) -> str:
        pass

    @property
    @abstractmethod
    def pretty_name(self) -> str:
        pass

    @property
    def install_dir(self) -> Path:
        return Path(config.DATA_ROOT) / self.name

    @property
    def compose_file(self) -> Path:
        return self.install_dir / "docker-compose.yml"

    @property
    def data_dir(self) -> Path:
        return self.install_dir / "data"

    @property
    @abstractmethod
    def image(self) -> str:
        pass

    @property
    def ports(self) -> Dict[str, str]:
        return {}

    @property
    def volumes(self) -> Dict[str, str]:
        return {}

    @property
    def env_vars(self) -> Dict[str, str]:
        return {
            "PUID": self.uid,
            "PGID": self.gid,
            "TZ": self.tz
        }

    @property
    def resource_limits(self) -> Dict[str, Any]:
        if self.profile == "LOW":
            return {"cpus": "0.5", "memory": "512M"}
        else:
            return {"cpus": "1.0", "memory": "2048M"}

    def get_template_context(self) -> Dict[str, Any]:
        return {
            "image": self.image,
            "container_name": self.name,
            "ports": self.ports,
            "volumes": self.volumes,
            "environment": self.env_vars,
            "network": config.DOCKER_NET,
            "restart": "unless-stopped",
            "resources": self.resource_limits
        }

    def generate_compose(self) -> str:
        template_str = """
services:
  {{ container_name }}:
    image: {{ image }}
    container_name: {{ container_name }}
    restart: {{ restart }}
    {% if ports %}
    ports:
      {% for host, container in ports.items() %}
      - "{{ host }}:{{ container }}"
      {% endfor %}
    {% endif %}
    {% if volumes %}
    volumes:
      {% for host, container in volumes.items() %}
      - "{{ host }}:{{ container }}"
      {% endfor %}
    {% endif %}
    {% if environment %}
    environment:
      {% for key, value in environment.items() %}
      - {{ key }}={{ value }}
      {% endfor %}
    {% endif %}
    deploy:
      resources:
        limits:
          cpus: '{{ resources.cpus }}'
          memory: {{ resources.memory }}
    networks:
      - {{ network }}

networks:
  {{ network }}:
    external: true
"""
        template = Template(template_str)
        return template.render(self.get_template_context())

    def install(self):
        logger.info(f"Installing {self.pretty_name} ({self.profile} profile)...")
        self.install_dir.mkdir(parents=True, exist_ok=True)

        with open(self.compose_file, "w") as f:
            f.write(self.generate_compose())

        try:
            subprocess.run(["docker", "compose", "up", "-d"], cwd=self.install_dir, check=True)
            logger.info(f"{self.pretty_name} installed successfully.")
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to install {self.pretty_name}: {e}")
            return

        self.post_install()

    def post_install(self):
        # Open firewall
        if self.ports:
            security_manager.enable_firewall(list(self.ports.keys()))

        # Configure Nginx
        # We assume the first port is the web interface
        if self.ports:
            web_port = list(self.ports.keys())[0]
            # Use name as subdomain e.g., plex.example.com
            proxy_manager.generate_config(self.name, web_port)

    def remove(self, delete_data: bool = False):
        logger.info(f"Removing {self.pretty_name}...")
        if self.compose_file.exists():
            try:
                subprocess.run(["docker", "compose", "down"], cwd=self.install_dir, check=False)
            except Exception as e:
                logger.warning(f"Error bringing down {self.pretty_name}: {e}")

        if delete_data:
            if self.install_dir.exists():
                import shutil
                shutil.rmtree(self.install_dir)
                logger.info(f"Data for {self.pretty_name} deleted.")
        else:
            if self.compose_file.exists():
                self.compose_file.unlink()

    def is_installed(self) -> bool:
        if self.compose_file.exists():
            return True
        return docker_manager.is_service_running(self.name)
