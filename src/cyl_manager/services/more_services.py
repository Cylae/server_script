from typing import Dict, Any
from .base import BaseService
from .registry import ServiceRegistry
from ..core.config import settings

@ServiceRegistry.register
class MailService(BaseService):
    name = "mailserver"
    pretty_name = "Mail Server (Docker Mailserver)"

    def generate_compose(self) -> Dict[str, Any]:
        is_high_perf = "1" if self.profile == "HIGH" else "0"

        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "docker.io/mailserver/docker-mailserver:latest",
                    "container_name": self.name,
                    "hostname": f"mail.{settings.DOMAIN}",
                    "domainname": settings.DOMAIN,
                    "restart": "always",
                    "stop_grace_period": "1m",
                    "cap_add": ["NET_ADMIN"],
                    "environment": {
                        "ENABLE_SPAMASSASSIN": is_high_perf,
                        "ENABLE_CLAMAV": is_high_perf,
                        "ENABLE_FAIL2BAN": "1",
                        "ENABLE_POSTGREY": "0",
                        "ONE_DIR": "1",
                        "DMS_DEBUG": "0"
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/mail/data:/var/mail",
                        f"{settings.DATA_DIR}/mail/state:/var/mail-state",
                        f"{settings.DATA_DIR}/mail/logs:/var/log/mail",
                        f"{settings.DATA_DIR}/mail/config:/tmp/docker-mailserver",
                        "/etc/localtime:/etc/localtime:ro"
                    ],
                    "ports": ["25:25", "143:143", "587:587", "993:993"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(
                        high_mem="2G", high_cpu="1.0",
                        low_mem="1G", low_cpu="0.5"
                    )
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

@ServiceRegistry.register
class GLPIService(BaseService):
    name = "glpi"
    pretty_name = "GLPI (IT Asset Management)"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "diouxx/glpi",
                    "container_name": self.name,
                    "restart": "always",
                    "environment": {
                        "TIMEZONE": self.system.get_timezone()
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/glpi/conf:/var/www/html/config",
                        f"{settings.DATA_DIR}/glpi/files:/var/www/html/files",
                        f"{settings.DATA_DIR}/glpi/plugins:/var/www/html/plugins"
                    ],
                    "ports": ["8090:80"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits()
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }
