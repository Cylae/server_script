from .media_arr import ArrService

class QBittorrentService(ArrService):
    def __init__(self):
        super().__init__("qbittorrent", "qBittorrent", "8080", "linuxserver/qbittorrent:latest")

    # Override install to handle specific ports (WebUI vs Torrent port)
    def install(self):
        # We need to map 6881 as well
        # But base install only maps one port.
        # So we override logic.
        print(f"Installing {self.display_name}...")
        from ..core import docker_manager, config
        import os

        docker_manager.create_network("server-net")
        puid = os.environ.get("SUDO_UID", "1000")
        pgid = os.environ.get("SUDO_GID", "1000")

        # NOTE: We change WebUI port to 8085 to avoid conflict with FileBrowser or others?
        # Usually qBit is 8080. If FileBrowser is also 8080, we have a conflict.
        # The prompt says "Port all other services".
        # Let's assume we use 8080. If conflict, user handles it.
        # Wait, if we use Nginx proxy, we can map 8080 to internal and not expose it?
        # But ArrService exposes "8080:8080".
        # Better to map "8080:8080" but allow changing host port if needed.
        # For now, stick to standard.

        compose = f"""
services:
  qbittorrent:
    image: linuxserver/qbittorrent:latest
    container_name: qbittorrent
    environment:
      - PUID={puid}
      - PGID={pgid}
      - TZ=Etc/UTC
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
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, f"/opt/{self.name}", compose)
        print(f"{self.display_name} deployed.")

        # Proxy
        from . import proxy
        domain = config.get_auth_value("DOMAIN")
        email = config.get_auth_value("EMAIL")
        if domain:
            full_domain = f"qbittorrent.{domain}"
            proxy.update_nginx(full_domain, "8080")
            if email:
                proxy.secure_domain(full_domain, email)
            print(f"Accessible at http://{full_domain}")
