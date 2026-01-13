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
        mem_limit = self.get_resource_limit(default_high="4096M", default_low="2048M")

        # Note: escaping { and } for f-string, but we need literal ${...} for docker-compose
        # So we use double {{ }} for python f-string escaping where we want literal {

        compose_content = f"""
services:
  plex:
    image: linuxserver/plex:latest
    container_name: plex
    environment:
      - PUID=${{SUDO_UID:-$(id -u)}}
      - PGID=${{SUDO_GID:-$(getent group docker | cut -d: -f3)}}
      - TZ=$(cat /etc/timezone)
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
