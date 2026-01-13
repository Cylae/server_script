from typing import Dict, Any
from .base import BaseService
from .registry import ServiceRegistry
from ..core.config import settings

@ServiceRegistry.register
class PlexService(BaseService):
    name = "plex"
    pretty_name = "Plex Media Server"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "lscr.io/linuxserver/plex:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "network_mode": "host",
                    "environment": {
                        **self.get_common_env(),
                        "VERSION": "docker"
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/plex:/config",
                        f"{settings.MEDIA_ROOT}:/media"
                    ],
                    "deploy": self.get_resource_limits(
                        high_mem="4G", high_cpu="2.0",
                        low_mem="1G", low_cpu="0.5"
                    )
                }
            }
        }

@ServiceRegistry.register
class TautulliService(BaseService):
    name = "tautulli"
    pretty_name = "Tautulli"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "lscr.io/linuxserver/tautulli:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "environment": self.get_common_env(),
                    "volumes": [f"{settings.DATA_DIR}/tautulli:/config"],
                    "ports": ["8181:8181"],
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

class ArrService(BaseService):
    """Base class for *Arr services to reduce duplication."""
    port: int

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": f"lscr.io/linuxserver/{self.name}:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "environment": self.get_common_env(),
                    "volumes": [
                        f"{settings.DATA_DIR}/{self.name}:/config",
                        f"{settings.MEDIA_ROOT}:/media"
                    ],
                    "ports": [f"{self.port}:{self.port}"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits()
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

@ServiceRegistry.register
class SonarrService(ArrService):
    name = "sonarr"
    pretty_name = "Sonarr"
    port = 8989

@ServiceRegistry.register
class RadarrService(ArrService):
    name = "radarr"
    pretty_name = "Radarr"
    port = 7878

@ServiceRegistry.register
class ProwlarrService(ArrService):
    name = "prowlarr"
    pretty_name = "Prowlarr"
    port = 9696

@ServiceRegistry.register
class JackettService(ArrService):
    name = "jackett"
    pretty_name = "Jackett"
    port = 9117

@ServiceRegistry.register
class OverseerrService(ArrService):
    name = "overseerr"
    pretty_name = "Overseerr"
    port = 5055

@ServiceRegistry.register
class QbittorrentService(BaseService):
    name = "qbittorrent"
    pretty_name = "qBittorrent"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "lscr.io/linuxserver/qbittorrent:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "environment": {
                        **self.get_common_env(),
                        "WEBUI_PORT": "8080"
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/qbittorrent:/config",
                        f"{settings.MEDIA_ROOT}/downloads:/downloads"
                    ],
                    "ports": ["8080:8080", "6881:6881", "6881:6881/udp"],
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }
