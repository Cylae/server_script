from .base import BaseService
from ..core import docker_manager, config, system
from . import proxy

class NetdataService(BaseService):
    @property
    def name(self): return "netdata"
    @property
    def display_name(self): return "Netdata"
    def install(self):
        print("Installing Netdata...")
        compose = """
services:
  netdata:
    image: netdata/netdata
    container_name: netdata
    cap_add:
      - SYS_PTRACE
    security_opt:
      - apparmor:unconfined
    volumes:
      - /opt/netdata/config:/etc/netdata
      - /opt/netdata/lib:/var/lib/netdata
      - /opt/netdata/cache:/var/cache/netdata
      - /:/host/root:ro,rslave
      - /etc/passwd:/host/etc/passwd:ro
      - /etc/group:/host/etc/group:ro
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
    network_mode: host
    restart: unless-stopped
"""
        docker_manager.deploy_service(self.name, "/opt/netdata", compose)
        domain = config.get_auth_value("DOMAIN")
        if domain:
            full = f"netdata.{domain}"
            proxy.update_nginx(full, "19999")
            if config.get_auth_value("EMAIL"): proxy.secure_domain(full, config.get_auth_value("EMAIL"))

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running(self.name)

class UptimeKumaService(BaseService):
    @property
    def name(self): return "uptimekuma"
    @property
    def display_name(self): return "Uptime Kuma"
    def install(self):
        print("Installing Uptime Kuma...")
        docker_manager.create_network("server-net")
        compose = """
services:
  uptime-kuma:
    image: louislam/uptime-kuma:1
    container_name: uptimekuma
    volumes:
      - /opt/uptimekuma/data:/app/data
    ports:
      - "3001:3001"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/uptimekuma", compose)
        domain = config.get_auth_value("DOMAIN")
        if domain:
            full = f"status.{domain}"
            proxy.update_nginx(full, "3001")
            if config.get_auth_value("EMAIL"): proxy.secure_domain(full, config.get_auth_value("EMAIL"))

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running("uptimekuma")
