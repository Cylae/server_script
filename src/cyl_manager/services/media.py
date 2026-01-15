from typing import Dict, Any, Final, Optional
from cyl_manager.services.base import BaseService
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.config import settings

@ServiceRegistry.register
class PlexService(BaseService):
    name: str = "plex"
    pretty_name: str = "Plex Media Server"

    def generate_compose(self) -> Dict[str, Any]:
        # Optimize transcoding: Use RAM for HIGH profile, Disk for LOW profile
        transcode_vol = "/tmp:/transcode" if not self.is_low_spec() else f"{settings.DATA_DIR}/plex/transcode:/transcode"

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
                        "VERSION": "docker",
                        "PLEX_CLAIM": "claim-TOKEN", # Placeholder, user should update config
                        # Optimize database cache size for Plex
                        # "PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS": "6" if not self.is_low_spec() else "2"
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/plex:/config",
                        f"{settings.MEDIA_ROOT}:/media",
                        transcode_vol
                    ],
                    "deploy": self.get_resource_limits(
                        high_mem="8G", high_cpu="4.0",
                        low_mem="2G", low_cpu="1.0"
                    )
                }
            }
        }

    def get_url(self) -> Optional[str]:
        return "http://localhost:32400/web"

@ServiceRegistry.register
class TautulliService(BaseService):
    name: str = "tautulli"
    pretty_name: str = "Tautulli"

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
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(
                        high_mem="512M", low_mem="256M"
                    )
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

    def get_url(self) -> Optional[str]:
        return f"http://{settings.DOMAIN}:8181"

class ArrService(BaseService):
    """Base class for *Arr services to reduce duplication."""
    port: int

    def generate_compose(self) -> Dict[str, Any]:
        env = self.get_common_env()
        # .NET Core optimization to reduce overhead
        env["COMPlus_EnableDiagnostics"] = "0"

        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": f"lscr.io/linuxserver/{self.name}:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "environment": env,
                    "volumes": [
                        f"{settings.DATA_DIR}/{self.name}:/config",
                        f"{settings.MEDIA_ROOT}:/media"
                    ],
                    "ports": [f"{self.port}:{self.port}"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(
                        high_mem="1G", low_mem="512M"
                    )
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

    def get_url(self) -> Optional[str]:
        return f"http://{settings.DOMAIN}:{self.port}"

@ServiceRegistry.register
class SonarrService(ArrService):
    name: str = "sonarr"
    pretty_name: str = "Sonarr"
    port: int = 8989

@ServiceRegistry.register
class RadarrService(ArrService):
    name: str = "radarr"
    pretty_name: str = "Radarr"
    port: int = 7878

@ServiceRegistry.register
class ProwlarrService(ArrService):
    name: str = "prowlarr"
    pretty_name: str = "Prowlarr"
    port: int = 9696

@ServiceRegistry.register
class JackettService(ArrService):
    name: str = "jackett"
    pretty_name: str = "Jackett"
    port: int = 9117

@ServiceRegistry.register
class OverseerrService(ArrService):
    name: str = "overseerr"
    pretty_name: str = "Overseerr"
    port: int = 5055

@ServiceRegistry.register
class QbittorrentService(BaseService):
    name: str = "qbittorrent"
    pretty_name: str = "qBittorrent"

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
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(
                        high_mem="2G", low_mem="512M"
                    )
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

    def get_url(self) -> Optional[str]:
        return f"http://{settings.DOMAIN}:8080"
