from .base import BaseService
from ..core import docker_manager
from ..core import config
from ..core import system
from . import proxy
import os

class PortainerService(BaseService):
    @property
    def name(self):
        return "portainer"

    @property
    def display_name(self):
        return "Portainer"

    def install(self):
        print(f"Installing {self.display_name}...")

        # Ensure network
        docker_manager.create_network("server-net")

        compose = """
services:
  portainer:
    image: portainer/portainer-ce:latest
    container_name: portainer
    command: -H unix:///var/run/docker.sock
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - /opt/portainer/data:/data
    ports:
      - "9000:9000"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        if docker_manager.deploy_service(self.name, "/opt/portainer", compose):
            print("Portainer container deployed.")

            # Proxy
            domain = config.get_auth_value("DOMAIN")
            email = config.get_auth_value("EMAIL")
            if domain:
                full_domain = f"portainer.{domain}"
                print(f"Configuring proxy for {full_domain}...")
                proxy.update_nginx(full_domain, "9000")
                if email:
                    proxy.secure_domain(full_domain, email)
                print(f"Portainer accessible at http://{full_domain} (or https if secured)")
            else:
                 print("No domain configured. Portainer accessible at http://<IP>:9000")

    def uninstall(self):
        docker_manager.remove_service(self.name)

    def is_installed(self):
        return docker_manager.is_running("portainer")
