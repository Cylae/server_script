from .base import BaseService
from ..core import docker_manager, config, system
from . import proxy

class WireGuardService(BaseService):
    @property
    def name(self): return "wireguard"
    @property
    def display_name(self): return "WireGuard (wg-easy)"
    def install(self):
        print("Installing WireGuard...")
        docker_manager.create_network("server-net")

        # Get public IP
        host_ip = system.get_public_ip()
        password = config.get_or_create_password("WG_PASSWORD")

        compose = f"""
services:
  wg-easy:
    image: weejewel/wg-easy
    container_name: wireguard
    environment:
      - WG_HOST={host_ip}
      - PASSWORD={password}
    volumes:
      - /opt/wireguard:/etc/wireguard
    ports:
      - "51820:51820/udp"
      - "51821:51821"
    cap_add:
      - NET_ADMIN
      - SYS_MODULE
    sysctls:
      - net.ipv4.ip_forward=1
      - net.ipv4.conf.all.src_valid_mark=1
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/wireguard", compose)
        domain = config.get_auth_value("DOMAIN")
        if domain:
            full = f"vpn.{domain}"
            proxy.update_nginx(full, "51821")
            if config.get_auth_value("EMAIL"): proxy.secure_domain(full, config.get_auth_value("EMAIL"))

    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running("wireguard")

class FTPService(BaseService):
    @property
    def name(self): return "ftp"
    @property
    def display_name(self): return "FTP Server"
    def install(self):
        print("Installing FTP...")
        docker_manager.create_network("server-net")
        user = "admin"
        password = config.get_or_create_password("FTP_PASSWORD")

        compose = f"""
services:
  ftp:
    image: fauria/vsftpd
    container_name: ftp
    environment:
      - FTP_USER={user}
      - FTP_PASS={password}
      - PASV_ADDRESS={system.get_public_ip()}
      - PASV_MIN_PORT=21100
      - PASV_MAX_PORT=21110
    volumes:
      - /opt/ftp_data:/home/vsftpd
    ports:
      - "20:20"
      - "21:21"
      - "21100-21110:21100-21110"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/ftp", compose)
    def uninstall(self): docker_manager.remove_service(self.name)
    def is_installed(self): return docker_manager.is_running(self.name)

class MailService(BaseService):
    @property
    def name(self): return "mail"
    @property
    def display_name(self): return "Mail Server (Docker Mailserver)"
    def install(self):
        print("Installing Mail Server...")
        docker_manager.create_network("server-net")
        domain = config.get_auth_value("DOMAIN") or "localhost.localdomain"

        compose = f"""
services:
  mailserver:
    image: mailserver/docker-mailserver:latest
    container_name: mailserver
    hostname: mail
    domainname: {domain}
    environment:
      - ENABLE_SPAMASSASSIN=1
      - ENABLE_CLAMAV=1
      - ENABLE_FAIL2BAN=1
      - ENABLE_POSTGREY=1
      - ONE_DIR=1
      - DMS_DEBUG=0
    volumes:
      - /opt/mail/mail-data/:/var/mail/
      - /opt/mail/mail-state/:/var/mail-state/
      - /opt/mail/mail-logs/:/var/log/mail/
      - /opt/mail/config/:/tmp/docker-mailserver/
      - /etc/localtime:/etc/localtime:ro
    ports:
      - "25:25"
      - "143:143"
      - "587:587"
      - "993:993"
    cap_add:
      - NET_ADMIN
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/mail", compose)
        print("Note: You must configure users manually via 'docker exec -ti mailserver setup email add ...'")

    def uninstall(self): docker_manager.remove_service("mailserver")
    def is_installed(self): return docker_manager.is_running("mailserver")
