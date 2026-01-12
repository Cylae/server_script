from .base import Service
from ..core.docker_runner import deploy_compose, remove_compose, DOCKER_NET
from ..core.proxy import update_nginx
from ..core.config import config
import os

class PortainerService(Service):
    def __init__(self):
        super().__init__("portainer")
        self.install_path = "/opt/portainer"

    def install(self):
        compose_content = f"""
services:
  portainer:
    image: portainer/portainer-ce
    container_name: portainer
    restart: always
    ports:
      - "127.0.0.1:9000:9000"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./data:/data
    networks:
      - {DOCKER_NET}
networks:
  {DOCKER_NET}:
    external: true
"""
        deploy_compose(self.name, compose_content, self.install_path)

        # Nginx
        domain = config.get("DOMAIN")
        if domain:
            subdomain = f"portainer.{domain}"
            update_nginx(subdomain, 9000, "proxy")

    def remove(self, delete_data=False):
        remove_compose(self.name, self.install_path, delete_data)

        # Cleanup Nginx?
        domain = config.get("DOMAIN")
        if domain:
            subdomain = f"portainer.{domain}"
            enabled_path = f"/etc/nginx/sites-enabled/{subdomain}"
            avail_path = f"/etc/nginx/sites-available/{subdomain}"
            if os.path.exists(enabled_path):
                os.remove(enabled_path)
            if os.path.exists(avail_path):
                os.remove(avail_path)

    def status(self):
        # Simple check if path exists
        return os.path.exists(os.path.join(self.install_path, "docker-compose.yml"))
