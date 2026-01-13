import os
import subprocess
import time
from .base import BaseService
from ..core.docker import deploy_service, remove_service, is_installed
from ..core.database import ensure_db, generate_password, save_credential, get_auth_value
from ..core.utils import ask, msg, fatal
from ..core.config import get

class NextcloudService(BaseService):
    name = "nextcloud"
    pretty_name = "Nextcloud"
    port = "8080"

    def install(self):
        subdomain = f"cloud.{self.domain}"

        pass_val = ask("Enter database password for Nextcloud (leave empty to generate)")
        if pass_val:
            save_credential("nextcloud_db_pass", pass_val)
        else:
            pass_val = get_auth_value("nextcloud_db_pass")
            if not pass_val:
                pass_val = generate_password()
                save_credential("nextcloud_db_pass", pass_val)

        ensure_db(self.name, self.name, pass_val)

        mem_limit = self.get_resource_limit(default_high="1024M", default_low="512M")

        # Get Gateway IP of the docker network
        try:
            res = subprocess.run(f"docker network inspect {self.docker_net} | jq -r '.[0].IPAM.Config[0].Gateway'", shell=True, capture_output=True, text=True)
            host_ip = res.stdout.strip()
        except:
             host_ip = "172.17.0.1" # Fallback

        compose_content = f"""
services:
  redis:
    image: redis:alpine
    container_name: nextcloud_redis
    networks:
      - {self.docker_net}
    restart: always
    command: redis-server --requirepass {pass_val}
    deploy:
      resources:
        limits:
          memory: 128M

  app:
    image: nextcloud
    container_name: nextcloud_app
    restart: always
    networks:
      - {self.docker_net}
    ports:
      - 127.0.0.1:8080:80
    depends_on:
      - redis
    volumes:
      - ./nextcloud:/var/www/html
    environment:
      - MYSQL_PASSWORD={pass_val}
      - MYSQL_DATABASE={self.name}
      - MYSQL_USER={self.name}
      - MYSQL_HOST={host_ip}
      - REDIS_HOST=nextcloud_redis
      - REDIS_HOST_PASSWORD={pass_val}
      - NEXTCLOUD_TRUSTED_DOMAINS={subdomain}
    deploy:
      resources:
        limits:
          memory: {mem_limit}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

        msg("Waiting for Nextcloud to initialize...")
        timeout = 120
        count = 0
        initialized = False
        while count < timeout:
            res = subprocess.run("docker exec -u www-data nextcloud_app php occ status", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if res.returncode == 0:
                initialized = True
                break
            time.sleep(2)
            count += 2

        if not initialized:
            fatal(f"Nextcloud failed to initialize within {timeout} seconds.")

        subprocess.run('docker exec -u www-data nextcloud_app php occ config:system:set trusted_proxies 0 --value="127.0.0.1"', shell=True, stdout=subprocess.DEVNULL)
        subprocess.run('docker exec -u www-data nextcloud_app php occ config:system:set overwriteprotocol --value="https"', shell=True, stdout=subprocess.DEVNULL)

        msg("Nextcloud Database Credentials:")
        print(f"   User: {self.name}")
        print(f"   Pass: {pass_val}")

    def remove(self):
        subdomain = f"cloud.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class VaultwardenService(BaseService):
    name = "vaultwarden"
    pretty_name = "Vaultwarden"
    port = "8082"

    def install(self):
        subdomain = f"pass.{self.domain}"

        mem_limit = self.get_resource_limit(default_high="512M", default_low="256M")

        compose_content = f"""
services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    restart: always
    environment:
      - SIGNUPS_ALLOWED=true
    volumes:
      - ./data:/data
    networks:
      - {self.docker_net}
    ports:
      - "127.0.0.1:8082:80"
    deploy:
      resources:
        limits:
          memory: {mem_limit}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

    def remove(self):
        subdomain = f"pass.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class UptimeKumaService(BaseService):
    name = "uptimekuma"
    pretty_name = "Uptime Kuma"
    port = "3001"

    def install(self):
        subdomain = f"status.{self.domain}"

        mem_limit = self.get_resource_limit(default_high="512M", default_low="256M")

        compose_content = f"""
services:
  uptime-kuma:
    image: louislam/uptime-kuma:1
    container_name: uptime-kuma
    restart: always
    volumes:
      - ./data:/app/data
    networks:
      - {self.docker_net}
    ports:
      - "127.0.0.1:3001:3001"
    deploy:
      resources:
        limits:
          memory: {mem_limit}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

    def remove(self):
        subdomain = f"status.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class WireGuardService(BaseService):
    name = "wireguard"
    pretty_name = "WireGuard"
    port = "51821"

    def install(self):
        subdomain = f"vpn.{self.domain}"

        wg_pass = ask("Enter WG Password (leave empty for auto/reuse)")
        if wg_pass:
            save_credential("wg_pass", wg_pass)
        else:
            wg_pass = get_auth_value("wg_pass")
            if not wg_pass:
                wg_pass = generate_password()
                save_credential("wg_pass", wg_pass)

        # Get public IP
        try:
             host_ip = subprocess.check_output("curl -s https://api.ipify.org", shell=True).decode().strip()
        except:
             host_ip = "127.0.0.1" # Fallback

        mem_limit = self.get_resource_limit(default_high="256M", default_low="128M")

        compose_content = f"""
services:
  wg-easy:
    environment:
      - WG_HOST={host_ip}
      - PASSWORD={wg_pass}
      - WG_PORT=51820
      - WG_DEFAULT_ADDRESS=10.8.0.x
      - WG_DEFAULT_DNS=1.1.1.1
      - WG_ALLOWED_IPS=0.0.0.0/0
      - WG_PERSISTENT_KEEPALIVE=25
    image: ghcr.io/wg-easy/wg-easy
    container_name: wg-easy
    volumes:
      - ./data:/etc/wireguard
    ports:
      - "51820:51820/udp"
      - "127.0.0.1:51821:51821/tcp"
    restart: unless-stopped
    cap_add:
      - NET_ADMIN
      - SYS_MODULE
    sysctls:
      - net.ipv4.ip_forward=1
      - net.ipv4.conf.all.src_valid_mark=1
    networks:
      - {self.docker_net}
    deploy:
      resources:
        limits:
          memory: {mem_limit}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)
        subprocess.run("ufw allow 51820/udp", shell=True, stdout=subprocess.DEVNULL)
        msg(f"WireGuard Password: {wg_pass}")

    def remove(self):
        subdomain = f"vpn.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
        subprocess.run("ufw delete allow 51820/udp", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

class FileBrowserService(BaseService):
    name = "filebrowser"
    pretty_name = "FileBrowser"
    port = "8083"

    def install(self):
        subdomain = f"files.{self.domain}"

        os.makedirs(f"/opt/{self.name}", exist_ok=True)
        # Touch db
        with open(f"/opt/{self.name}/filebrowser.db", "a"):
            os.utime(f"/opt/{self.name}/filebrowser.db", None)

        with open(f"/opt/{self.name}/settings.json", "w") as f:
            f.write('{"port": 80, "baseURL": "", "address": "", "log": "stdout", "database": "/database.db", "root": "/srv"}')

        mem_limit = self.get_resource_limit(default_high="256M", default_low="128M")

        compose_content = f"""
services:
  filebrowser:
    image: filebrowser/filebrowser
    container_name: filebrowser
    restart: always
    volumes:
      - /var/www:/srv
      - ./filebrowser.db:/database.db
      - ./settings.json:/.filebrowser.json
    networks:
      - {self.docker_net}
    ports:
      - "127.0.0.1:8083:80"
    deploy:
      resources:
        limits:
          memory: {mem_limit}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)
        msg("Default login: admin/admin")

    def remove(self):
        subdomain = f"files.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class YourlsService(BaseService):
    name = "yourls"
    pretty_name = "YOURLS"
    port = "8084"

    def install(self):
        subdomain = f"x.{self.domain}"

        pass_val = ask("Enter password for YOURLS admin (leave empty to generate)")
        if pass_val:
            save_credential("yourls_pass", pass_val)
        else:
            pass_val = get_auth_value("yourls_pass")
            if not pass_val:
                pass_val = generate_password()
                save_credential("yourls_pass", pass_val)

        ensure_db(self.name, self.name, pass_val)

        try:
            res = subprocess.run(f"docker network inspect {self.docker_net} | jq -r '.[0].IPAM.Config[0].Gateway'", shell=True, capture_output=True, text=True)
            host_ip = res.stdout.strip()
        except:
             host_ip = "172.17.0.1"

        cookie = generate_password(16) # Simple random hex-like

        mem_limit = self.get_resource_limit(default_high="256M", default_low="128M")

        compose_content = f"""
services:
  yourls:
    image: yourls:latest
    container_name: yourls
    restart: always
    networks:
      - {self.docker_net}
    ports:
      - "127.0.0.1:8084:80"
    volumes:
      - ./data:/var/www/html
    environment:
      - YOURLS_DB_HOST={host_ip}
      - YOURLS_DB_USER={self.name}
      - YOURLS_DB_PASS={pass_val}
      - YOURLS_DB_NAME={self.name}
      - YOURLS_SITE=https://{subdomain}
      - YOURLS_USER=admin
      - YOURLS_PASS={pass_val}
      - YOURLS_COOKIEKEY={cookie}
    deploy:
      resources:
        limits:
          memory: {mem_limit}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)
        msg(f"YOURLS Admin Password: {pass_val}")

    def remove(self):
        subdomain = f"x.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
