from .base import Service
from ..core.docker_runner import deploy_compose, remove_compose, DOCKER_NET
from ..core.proxy import update_nginx
from ..core.config import config
import os

class NetdataService(Service):
    def __init__(self):
        super().__init__("netdata")
        self.install_path = "/opt/netdata"

    def install(self):
        compose_content = f"""
services:
  netdata:
    image: netdata/netdata
    container_name: netdata
    restart: always
    network_mode: host
    pid: host
    cap_add:
      - SYS_PTRACE
      - SYS_ADMIN
    security_opt:
      - apparmor:unconfined
    volumes:
      - netdatalib:/var/lib/netdata
      - netdatacache:/var/cache/netdata
      - /etc/passwd:/host/etc/passwd:ro
      - /etc/group:/host/etc/group:ro
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /etc/os-release:/host/etc/os-release:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro

volumes:
  netdatalib:
  netdatacache:
"""
        # Note: netdata typically uses host networking, so we don't attach to DOCKER_NET usually, or we do both.
        # The legacy script uses network_mode: host.

        deploy_compose(self.name, compose_content, self.install_path)

        # Nginx
        domain = config.get("DOMAIN")
        if domain:
            subdomain = f"netdata.{domain}"
            update_nginx(subdomain, 19999, "proxy") # Netdata default port

    def remove(self, delete_data=False):
        remove_compose(self.name, self.install_path, delete_data)

        # Cleanup Nginx
        domain = config.get("DOMAIN")
        if domain:
            subdomain = f"netdata.{domain}"
            enabled_path = f"/etc/nginx/sites-enabled/{subdomain}"
            avail_path = f"/etc/nginx/sites-available/{subdomain}"
            if os.path.exists(enabled_path):
                os.remove(enabled_path)
            if os.path.exists(avail_path):
                os.remove(avail_path)

    def status(self):
        return os.path.exists(os.path.join(self.install_path, "docker-compose.yml"))
