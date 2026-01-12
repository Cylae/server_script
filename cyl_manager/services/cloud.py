from .base import BaseService
from ..core import docker_manager, config, system
from . import proxy, database
import os

class NextcloudService(BaseService):
    @property
    def name(self): return "nextcloud"
    @property
    def display_name(self): return "Nextcloud"

    def install(self):
        print("Installing Nextcloud...")
        docker_manager.create_network("server-net")

        # Database setup
        db_pass = config.get_or_create_password("NEXTCLOUD_DB_PASS")
        db_srv = database.MariaDBService()
        db_srv.ensure_db("nextcloud", "nextcloud", db_pass)

        compose = f"""
services:
  nextcloud:
    image: linuxserver/nextcloud:latest
    container_name: nextcloud
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    volumes:
      - /opt/nextcloud/config:/config
      - /opt/nextcloud/data:/data
    ports:
      - "4443:443"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/nextcloud", compose)

        # Proxy
        domain = config.get_auth_value("DOMAIN")
        email = config.get_auth_value("EMAIL")
        if domain:
            full_domain = f"nextcloud.{domain}"
            # Nextcloud lsio image uses https on 443
            proxy.update_nginx(full_domain, "4443", proxy_protocol="https")
            if email:
                proxy.secure_domain(full_domain, email)
            print(f"Accessible at http://{full_domain}")

    def uninstall(self):
        docker_manager.remove_service(self.name)
    def is_installed(self):
        return docker_manager.is_running(self.name)


class FileBrowserService(BaseService):
    @property
    def name(self): return "filebrowser"
    @property
    def display_name(self): return "FileBrowser"
    def install(self):
        print("Installing FileBrowser...")
        docker_manager.create_network("server-net")
        compose = """
services:
  filebrowser:
    image: filebrowser/filebrowser:latest
    container_name: filebrowser
    volumes:
      - /:/srv
      - /opt/filebrowser/filebrowser.db:/database.db
    ports:
      - "8082:80"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/filebrowser", compose)
        # Proxy
        domain = config.get_auth_value("DOMAIN")
        email = config.get_auth_value("EMAIL")
        if domain:
            full_domain = f"files.{domain}"
            proxy.update_nginx(full_domain, "8082")
            if email: proxy.secure_domain(full_domain, email)

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running(self.name)

class VaultwardenService(BaseService):
    @property
    def name(self): return "vaultwarden"
    @property
    def display_name(self): return "Vaultwarden"
    def install(self):
        print("Installing Vaultwarden...")
        docker_manager.create_network("server-net")
        compose = """
services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    volumes:
      - /opt/vaultwarden/data:/data
    ports:
      - "8081:80"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/vaultwarden", compose)
        domain = config.get_auth_value("DOMAIN")
        email = config.get_auth_value("EMAIL")
        if domain:
            full_domain = f"bitwarden.{domain}"
            proxy.update_nginx(full_domain, "8081")
            if email: proxy.secure_domain(full_domain, email)

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running(self.name)
