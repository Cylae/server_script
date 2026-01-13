from .base import BaseService
from ..core.docker import deploy_service, remove_service
import os
import subprocess

class TautulliService(BaseService):
    name = "tautulli"
    pretty_name = "Tautulli"
    port = "8181"

    def install(self):
        subdomain = f"tautulli.{self.domain}"
        os.makedirs("/opt/media", exist_ok=True) # ensure media dir

        mem_limit = self.get_resource_limit(default_high="1024M", default_low="256M")
        env = self.get_common_env()

        compose_content = f"""
services:
  tautulli:
    image: lscr.io/linuxserver/tautulli:latest
    container_name: tautulli
    environment:
      - PUID={env['PUID']}
      - PGID={env['PGID']}
      - TZ={env['TZ']}
    volumes:
      - /opt/tautulli/config:/config
    ports:
      - "8181:8181"
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
        subdomain = f"tautulli.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class SonarrService(BaseService):
    name = "sonarr"
    pretty_name = "Sonarr"
    port = "8989"

    def install(self):
        subdomain = f"sonarr.{self.domain}"
        os.makedirs("/opt/media", exist_ok=True)

        mem_limit = self.get_resource_limit(default_high="2048M", default_low="512M")
        env = self.get_common_env()

        compose_content = f"""
services:
  sonarr:
    image: lscr.io/linuxserver/sonarr:latest
    container_name: sonarr
    environment:
      - PUID={env['PUID']}
      - PGID={env['PGID']}
      - TZ={env['TZ']}
    volumes:
      - /opt/sonarr/config:/config
      - /opt/media:/data
    ports:
      - "8989:8989"
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
        subdomain = f"sonarr.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class RadarrService(BaseService):
    name = "radarr"
    pretty_name = "Radarr"
    port = "7878"

    def install(self):
        subdomain = f"radarr.{self.domain}"
        os.makedirs("/opt/media", exist_ok=True)

        mem_limit = self.get_resource_limit(default_high="2048M", default_low="512M")
        env = self.get_common_env()

        compose_content = f"""
services:
  radarr:
    image: lscr.io/linuxserver/radarr:latest
    container_name: radarr
    environment:
      - PUID={env['PUID']}
      - PGID={env['PGID']}
      - TZ={env['TZ']}
    volumes:
      - /opt/radarr/config:/config
      - /opt/media:/data
    ports:
      - "7878:7878"
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
        subdomain = f"radarr.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class ProwlarrService(BaseService):
    name = "prowlarr"
    pretty_name = "Prowlarr"
    port = "9696"

    def install(self):
        subdomain = f"prowlarr.{self.domain}"
        os.makedirs("/opt/media", exist_ok=True)

        mem_limit = self.get_resource_limit(default_high="1024M", default_low="256M")
        env = self.get_common_env()

        compose_content = f"""
services:
  prowlarr:
    image: lscr.io/linuxserver/prowlarr:latest
    container_name: prowlarr
    environment:
      - PUID={env['PUID']}
      - PGID={env['PGID']}
      - TZ={env['TZ']}
    volumes:
      - /opt/prowlarr/config:/config
    ports:
      - "9696:9696"
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
        subdomain = f"prowlarr.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class JackettService(BaseService):
    name = "jackett"
    pretty_name = "Jackett"
    port = "9117"

    def install(self):
        subdomain = f"jackett.{self.domain}"
        os.makedirs("/opt/media", exist_ok=True)

        mem_limit = self.get_resource_limit(default_high="1024M", default_low="256M")
        env = self.get_common_env()

        compose_content = f"""
services:
  jackett:
    image: lscr.io/linuxserver/jackett:latest
    container_name: jackett
    environment:
      - PUID={env['PUID']}
      - PGID={env['PGID']}
      - TZ={env['TZ']}
      - AUTO_UPDATE=true
    volumes:
      - /opt/jackett/config:/config
      - /opt/media:/data
    ports:
      - "9117:9117"
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
        subdomain = f"jackett.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class OverseerrService(BaseService):
    name = "overseerr"
    pretty_name = "Overseerr"
    port = "5055"

    def install(self):
        subdomain = f"request.{self.domain}"
        os.makedirs("/opt/media", exist_ok=True)

        mem_limit = self.get_resource_limit(default_high="1024M", default_low="256M")
        env = self.get_common_env()

        compose_content = f"""
services:
  overseerr:
    image: lscr.io/linuxserver/overseerr:latest
    container_name: overseerr
    environment:
      - PUID={env['PUID']}
      - PGID={env['PGID']}
      - TZ={env['TZ']}
    volumes:
      - /opt/overseerr/config:/config
    ports:
      - "5055:5055"
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
        subdomain = f"request.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class QbittorrentService(BaseService):
    name = "qbittorrent"
    pretty_name = "qBittorrent"
    port = "8080" # WebUI port

    def install(self):
        subdomain = f"qbittorrent.{self.domain}"
        os.makedirs("/opt/media", exist_ok=True)

        mem_limit = self.get_resource_limit(default_high="2048M", default_low="512M")
        env = self.get_common_env()

        compose_content = f"""
services:
  qbittorrent:
    image: lscr.io/linuxserver/qbittorrent:latest
    container_name: qbittorrent
    environment:
      - PUID={env['PUID']}
      - PGID={env['PGID']}
      - TZ={env['TZ']}
      - WEBUI_PORT=8080
    volumes:
      - /opt/qbittorrent/config:/config
      - /opt/media:/data
    ports:
      - "8080:8080"
      - "6881:6881"
      - "6881:6881/udp"
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
        subprocess.run("ufw allow 6881", shell=True, stdout=subprocess.DEVNULL)

    def remove(self):
        subdomain = f"qbittorrent.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
        subprocess.run("ufw delete allow 6881", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
