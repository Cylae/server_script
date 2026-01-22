from typing import Dict, Any, Final, Optional, List
from cyl_manager.services.base import BaseService
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.config import settings

@ServiceRegistry.register
class GiteaService(BaseService):
    name: str = "gitea"
    pretty_name: str = "Gitea"

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
                    "security_opt": self.get_security_opts(),
                    "logging": self.get_logging_config(),
                    "environment": env,
                    "volumes": [
                        f"{settings.DATA_DIR}/gitea:/data",
                        "/etc/timezone:/etc/timezone:ro",
                        "/etc/localtime:/etc/localtime:ro"
                    ],
                    "ports": ["3000:3000", "222:22"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(high_mem="1G", low_mem="512M")
                }
            },
            "networks": {
                settings.DOCKER_NET: {"external": True}
            }
        }

    def get_install_summary(self) -> Optional[str]:
        return (
            f"URL: http://{settings.DOMAIN}:3000\n"
            f"SSH Port: 222\n"
            "Note: Create your first user via the web interface."
        )

    def get_url(self) -> Optional[str]:
        return f"https://git.{settings.DOMAIN}"

    def get_ports(self) -> List[str]:
        return ["3000/tcp", "222/tcp"]
