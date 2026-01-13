from typing import Dict, Any
from .base import BaseService
from .registry import ServiceRegistry
from ..core.config import settings

@ServiceRegistry.register
class PortainerService(BaseService):
    name = "portainer"
    pretty_name = "Portainer"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "portainer/portainer-ce:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "security_opt": ["no-new-privileges:true"],
                    "volumes": [
                        "/etc/localtime:/etc/localtime:ro",
                        "/var/run/docker.sock:/var/run/docker.sock:ro",
                        f"{settings.DATA_DIR}/portainer:/data"
                    ],
                    "ports": ["9000:9000"],
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }
