from .base import BaseService
from ..core import docker_manager
from ..core import config
from . import proxy
import os

class PlexService(BaseService):
    @property
    def name(self):
        return "plex"

    @property
    def display_name(self):
        return "Plex Media Server"

    def install(self):
        print("Installing Plex...")

        # Determine PUID/PGID
        puid = os.environ.get("SUDO_UID", str(os.getuid()))
        # Try to get docker group id
        try:
            import grp
            pgid = str(grp.getgrnam('docker').gr_gid)
        except:
            pgid = os.environ.get("SUDO_GID", str(os.getgid()))

        # Claim token?
        # In a real comprehensive app, we would ask for a claim token.
        # check config
        claim = config.get_auth_value("PLEX_CLAIM", "")

        compose = f"""
services:
  plex:
    image: linuxserver/plex:latest
    container_name: plex
    environment:
      - PUID={puid}
      - PGID={pgid}
      - VERSION=docker
      - PLEX_CLAIM={claim}
    volumes:
      - /opt/plex/config:/config
      - /opt/media:/data
    ports:
      - "32400:32400"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/plex", compose)
        print("Plex container deployed.")

        # Proxy
        domain = config.get_auth_value("DOMAIN")
        email = config.get_auth_value("EMAIL")
        if domain:
            full_domain = f"plex.{domain}"
            print(f"Configuring proxy for {full_domain}...")
            proxy.update_nginx(full_domain, "32400", service_type="cloud") # Use cloud/large body for Plex
            if email:
                proxy.secure_domain(full_domain, email)
            print(f"Plex accessible at http://{full_domain} (or https if secured)")

    def uninstall(self):
        docker_manager.remove_service(self.name)

    def is_installed(self):
        return docker_manager.is_running("plex")
