from typing import Dict, Any
from .base import BaseService
from .registry import ServiceRegistry
from ..core.config import settings

@ServiceRegistry.register
class GiteaService(BaseService):
    name = "gitea"
    pretty_name = "Gitea"

    def generate_compose(self) -> Dict[str, Any]:
        env = self.get_common_env()
        env.update({
            "USER_UID": env["PUID"],
            "USER_GID": env["PGID"],
            "GITEA__database__DB_TYPE": "sqlite3",
            "GITEA__database__PATH": "/data/gitea/gitea.db",
            "GITEA__server__DOMAIN": settings.DOMAIN,
            "GITEA__server__ROOT_URL": f"https://git.{settings.DOMAIN}/",
            "GITEA__security__INSTALL_LOCK": "true"
        })

        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "gitea/gitea:latest",
                    "container_name": self.name,
                    "restart": "always",
                    "environment": env,
                    "volumes": [
                        f"{settings.DATA_DIR}/gitea:/data",
                        "/etc/timezone:/etc/timezone:ro",
                        "/etc/localtime:/etc/localtime:ro"
                    ],
                    "ports": ["3000:3000", "222:22"],
                    "networks": [settings.DOCKER_NET]
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }
