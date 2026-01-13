from .base import BaseService
from ..core.docker import deploy_service, remove_service
import os

class PlexService(BaseService):
    name = "plex"
    pretty_name = "Plex Media Server"
    port = "32400"

    def install(self):
        subdomain = f"plex.{self.domain}"

        # Ensure media directories exist
        os.makedirs("/opt/media", exist_ok=True)

        # Get dynamic resource limits
        # Optimized for 2GB RAM Host: 1024M max for Plex
        mem_limit = self.get_resource_limit(default_high="4096M", default_low="1024M")

        compose_content = f"""
services:
  plex:
    image: linuxserver/plex:latest
    container_name: plex
    environment:
      - PUID={self.uid}
      - PGID={self.gid}
      - TZ={self.tz}
      - VERSION=docker
    volumes:
      - /opt/plex/config:/config
      - /opt/media:/data
    ports:
      - "32400:32400"
    networks:
      - server-net
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: {mem_limit}

networks:
  server-net:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

    def remove(self):
        subdomain = f"plex.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
