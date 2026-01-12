from .base import BaseService
from ..core import docker_manager, config, system
from . import proxy
import os

class ArrService(BaseService):
    def __init__(self, name, display_name, port, image):
        self._name = name
        self._display_name = display_name
        self._port = port
        self._image = image

    @property
    def name(self):
        return self._name

    @property
    def display_name(self):
        return self._display_name

    def install(self):
        print(f"Installing {self.display_name}...")
        docker_manager.create_network("server-net")

        puid = os.environ.get("SUDO_UID", "1000")
        pgid = os.environ.get("SUDO_GID", "1000")

        compose = f"""
services:
  {self.name}:
    image: {self._image}
    container_name: {self.name}
    environment:
      - PUID={puid}
      - PGID={pgid}
      - TZ=Etc/UTC
    volumes:
      - /opt/{self.name}/config:/config
      - /opt/media:/data
    ports:
      - "{self._port}:{self._port}"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, f"/opt/{self.name}", compose)
        print(f"{self.display_name} deployed.")

        # Proxy
        domain = config.get_auth_value("DOMAIN")
        email = config.get_auth_value("EMAIL")
        if domain:
            full_domain = f"{self.name}.{domain}"
            proxy.update_nginx(full_domain, self._port)
            if email:
                proxy.secure_domain(full_domain, email)
            print(f"Accessible at http://{full_domain}")

    def uninstall(self):
        docker_manager.remove_service(self.name)

    def is_installed(self):
        return docker_manager.is_running(self.name)

# Specific Classes
class SonarrService(ArrService):
    def __init__(self):
        super().__init__("sonarr", "Sonarr", "8989", "linuxserver/sonarr:latest")

class RadarrService(ArrService):
    def __init__(self):
        super().__init__("radarr", "Radarr", "7878", "linuxserver/radarr:latest")

class ProwlarrService(ArrService):
    def __init__(self):
        super().__init__("prowlarr", "Prowlarr", "9696", "linuxserver/prowlarr:latest")

class JackettService(ArrService):
    def __init__(self):
        super().__init__("jackett", "Jackett", "9117", "linuxserver/jackett:latest")

class OverseerrService(ArrService):
    def __init__(self):
        super().__init__("overseerr", "Overseerr", "5055", "sctx/overseerr:latest")

class TautulliService(ArrService):
    def __init__(self):
        super().__init__("tautulli", "Tautulli", "8181", "linuxserver/tautulli:latest")
