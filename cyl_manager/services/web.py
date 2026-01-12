from .base import BaseService
from ..core import docker_manager, config, system
from . import proxy, database

class GiteaService(BaseService):
    @property
    def name(self): return "gitea"
    @property
    def display_name(self): return "Gitea"
    def install(self):
        print("Installing Gitea...")
        docker_manager.create_network("server-net")
        db_pass = config.get_or_create_password("GITEA_DB_PASS")
        database.MariaDBService().ensure_db("gitea", "gitea", db_pass)

        compose = """
services:
  gitea:
    image: gitea/gitea:latest
    container_name: gitea
    environment:
      - USER_UID=1000
      - USER_GID=1000
    volumes:
      - /opt/gitea/data:/data
      - /etc/timezone:/etc/timezone:ro
      - /etc/localtime:/etc/localtime:ro
    ports:
      - "3000:3000"
      - "2222:22"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/gitea", compose)
        domain = config.get_auth_value("DOMAIN")
        if domain:
            full = f"git.{domain}"
            proxy.update_nginx(full, "3000")
            if config.get_auth_value("EMAIL"): proxy.secure_domain(full, config.get_auth_value("EMAIL"))

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running(self.name)

class GLPIService(BaseService):
    @property
    def name(self): return "glpi"
    @property
    def display_name(self): return "GLPI"
    def install(self):
        print("Installing GLPI...")
        docker_manager.create_network("server-net")
        db_pass = config.get_or_create_password("GLPI_DB_PASS")
        database.MariaDBService().ensure_db("glpi", "glpi", db_pass)
        # Using a reliable image, e.g., diouxx/glpi or generic?
        # Assuming diouxx/glpi as it's popular
        compose = """
services:
  glpi:
    image: diouxx/glpi
    container_name: glpi
    ports:
      - "8083:80"
    volumes:
      - /opt/glpi/glpi:/var/www/html/glpi
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/glpi", compose)
        domain = config.get_auth_value("DOMAIN")
        if domain:
            full = f"glpi.{domain}"
            proxy.update_nginx(full, "8083")
            if config.get_auth_value("EMAIL"): proxy.secure_domain(full, config.get_auth_value("EMAIL"))

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running(self.name)

class YourLSService(BaseService):
    @property
    def name(self): return "yourls"
    @property
    def display_name(self): return "YourLS"
    def install(self):
        print("Installing YourLS...")
        docker_manager.create_network("server-net")
        db_pass = config.get_or_create_password("YOURLS_DB_PASS")
        database.MariaDBService().ensure_db("yourls", "yourls", db_pass)

        domain = config.get_auth_value("DOMAIN") or "localhost"

        compose = f"""
services:
  yourls:
    image: yourls:latest
    container_name: yourls
    environment:
      - YOURLS_DB_HOST=mariadb
      - YOURLS_DB_USER=yourls
      - YOURLS_DB_PASS={db_pass}
      - YOURLS_DB_NAME=yourls
      - YOURLS_SITE=http://{domain}
      - YOURLS_USER=admin
      - YOURLS_PASS=admin
    ports:
      - "8084:80"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/yourls", compose)
        if domain and domain != "localhost":
            full = f"go.{domain}" # Usually subdirectory or subdomain
            # Yourls is tricky with domains. Assuming subdomain.
            proxy.update_nginx(full, "8084")
            if config.get_auth_value("EMAIL"): proxy.secure_domain(full, config.get_auth_value("EMAIL"))

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running(self.name)
