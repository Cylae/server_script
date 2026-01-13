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

        # Get user/group ID safely in Python
        try:
            puid = os.getuid()
            # Try to get docker group id, fallback to current group
            import grp
            try:
                pgid = grp.getgrnam('docker').gr_gid
            except KeyError:
                pgid = os.getgid()
        except Exception:
            puid = 1000
            pgid = 1000

        # Get Timezone
        try:
            with open("/etc/timezone", "r") as f:
                tz = f.read().strip()
        except FileNotFoundError:
            tz = "UTC"

        compose_content = f"""
services:
  plex:
    image: linuxserver/plex:latest
    container_name: plex
    environment:
      - PUID={puid}
      - PGID={pgid}
      - TZ={tz}
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
