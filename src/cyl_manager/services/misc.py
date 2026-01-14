from typing import Dict, Any, Final
from cyl_manager.services.base import BaseService
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.config import settings

@ServiceRegistry.register
class NextcloudService(BaseService):
    name: str = "nextcloud"
    pretty_name: str = "Nextcloud"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "nextcloud",
                    "container_name": self.name,
                    "restart": "always",
                    "environment": self.get_common_env(),
                    "volumes": [
                        f"{settings.DATA_DIR}/nextcloud:/var/www/html"
                    ],
                    "ports": ["8084:80"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(high_mem="2G", low_mem="1G")
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }

@ServiceRegistry.register
class VaultwardenService(BaseService):
    name: str = "vaultwarden"
    pretty_name: str = "Vaultwarden"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "vaultwarden/server:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "environment": {"WEBSOCKET_ENABLED": "true"},
                    "volumes": [f"{settings.DATA_DIR}/vaultwarden:/data"],
                    "ports": ["8081:80"],
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }

@ServiceRegistry.register
class UptimeKumaService(BaseService):
    name: str = "uptime-kuma"
    pretty_name: str = "Uptime Kuma"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "louislam/uptime-kuma:1",
                    "container_name": self.name,
                    "restart": "always",
                    "volumes": [
                        f"{settings.DATA_DIR}/uptime-kuma:/app/data",
                        "/var/run/docker.sock:/var/run/docker.sock:ro"
                    ],
                    "ports": ["3001:3001"],
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }

@ServiceRegistry.register
class WireGuardService(BaseService):
    name: str = "wireguard"
    pretty_name: str = "WireGuard"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "lscr.io/linuxserver/wireguard:latest",
                    "container_name": self.name,
                    "cap_add": ["NET_ADMIN", "SYS_MODULE"],
                    "environment": {
                        **self.get_common_env(),
                        "SERVERURL": f"vpn.{settings.DOMAIN}",
                        "SERVERPORT": "51820",
                        "PEERS": "1",
                        "PEERDNS": "auto",
                        "INTERNAL_SUBNET": "10.13.13.0"
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/wireguard:/config",
                        "/lib/modules:/lib/modules"
                    ],
                    "ports": ["51820:51820/udp"],
                    "sysctls": {"net.ipv4.conf.all.src_valid_mark": 1},
                    "restart": "unless-stopped",
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }

@ServiceRegistry.register
class FileBrowserService(BaseService):
    name: str = "filebrowser"
    pretty_name: str = "FileBrowser"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "filebrowser/filebrowser:s6",
                    "container_name": self.name,
                    "environment": self.get_common_env(),
                    "volumes": [
                        f"{settings.DATA_DIR}/filebrowser/filebrowser.db:/database.db",
                        f"{settings.DATA_DIR}/filebrowser/.filebrowser.json:/.filebrowser.json",
                        f"{settings.MEDIA_ROOT}:/srv"
                    ],
                    "ports": ["8082:80"],
                    "restart": "unless-stopped",
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }

@ServiceRegistry.register
class YourlsService(BaseService):
    name: str = "yourls"
    pretty_name: str = "YOURLS"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "yourls",
                    "container_name": self.name,
                    "restart": "always",
                    "ports": ["8083:80"],
                    "environment": {"YOURLS_SITE": f"https://link.{settings.DOMAIN}"},
                    "volumes": [f"{settings.DATA_DIR}/yourls:/var/www/html"],
                    "networks": [settings.DOCKER_NET]
                }
            },
             "networks": {settings.DOCKER_NET: {"external": True}}
        }

@ServiceRegistry.register
class NetdataService(BaseService):
    name: str = "netdata"
    pretty_name: str = "Netdata"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "netdata/netdata",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "cap_add": ["SYS_PTRACE"],
                    "security_opt": ["apparmor:unconfined"],
                    "volumes": ["/var/run/docker.sock:/var/run/docker.sock:ro"],
                    "ports": ["19999:19999"],
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }
