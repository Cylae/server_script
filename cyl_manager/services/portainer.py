from .base import BaseService
from ..core.docker import deploy_service, remove_service
import os
import subprocess
from ..core.utils import msg

class PortainerService(BaseService):
    name = "portainer"
    pretty_name = "Portainer"
    port = "9000"

    def install(self):
        subdomain = f"portainer.{self.domain}"

        # Check for legacy container
        # simplistic check
        try:
             subprocess.run("docker stop portainer", shell=True, stderr=subprocess.DEVNULL)
             subprocess.run("docker rm portainer", shell=True, stderr=subprocess.DEVNULL)
        except:
             pass

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
      - {self.docker_net}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

    def remove(self):
        subdomain = f"portainer.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
