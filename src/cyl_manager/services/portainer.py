from typing import Dict, Any, Final, Optional
from cyl_manager.services.base import BaseService
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.config import settings

@ServiceRegistry.register
class PortainerService(BaseService):
    name: str = "portainer"
    pretty_name: str = "Portainer"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "portainer/portainer-ce:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "security_opt": self.get_security_opts(),
                    "logging": self.get_logging_config(),
                    "volumes": [
                        "/etc/localtime:/etc/localtime:ro",
                        "/var/run/docker.sock:/var/run/docker.sock:ro",
                        f"{settings.DATA_DIR}/portainer:/data"
                    ],
                    "ports": ["9000:9000"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(high_mem="512M", low_mem="256M")
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }

    def get_url(self) -> Optional[str]:
        return f"http://{settings.DOMAIN}:9000"

    def get_install_summary(self) -> Optional[str]:
        return (
            f"URL: http://{settings.DOMAIN}:9000\n"
            "Create your admin account on first login."
        )
