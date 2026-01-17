from typing import Dict, Any, Final, Optional, List
from cyl_manager.services.base import BaseService
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.config import settings
from cyl_manager.core.system import SystemManager

@ServiceRegistry.register
class MailService(BaseService):
    name: str = "mailserver"
    pretty_name: str = "Mail Server (Docker Mailserver)"

    def generate_compose(self) -> Dict[str, Any]:
        # Optimization: Disable heavy processes on low-spec hardware
        # This prevents startup timeouts and infinite "Waiting for mailserver" loops
        is_low = self.is_low_spec()
        enable_heavy_procs = "1" if not is_low else "0"

        # On low spec, we also disable Fail2Ban to save RAM, unless explicitly requested otherwise.
        # Fail2Ban uses python/gamin and can consume 100-200MB+ which is critical on 1GB VPS.
        # However, for security, some might want it. We'll default to 0 for LOW profile to ensure stability first.
        enable_fail2ban = "1" if not is_low else "0"

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
                        "ENABLE_SPAMASSASSIN": enable_heavy_procs,
                        "ENABLE_CLAMAV": enable_heavy_procs,
                        "ENABLE_FAIL2BAN": enable_fail2ban,
                        "ENABLE_POSTGREY": "0",
                        "ONE_DIR": "1",
                        "DMS_DEBUG": "0",
                        # Optimizations for low memory environments
                        "CLAMAV_MESSAGE_SIZE_LIMIT": "20M",
                        "VIRUSMAILS_DELETE_DELAY": "7"
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
                        low_mem="1G", low_cpu="1.0"
                    )
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

    def get_url(self) -> Optional[str]:
        return f"mail.{settings.DOMAIN}"

    def get_ports(self) -> List[str]:
        return ["25/tcp", "143/tcp", "587/tcp", "993/tcp"]

@ServiceRegistry.register
class GLPIService(BaseService):
    name: str = "glpi"
    pretty_name: str = "GLPI (IT Asset Management)"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "diouxx/glpi",
                    "container_name": self.name,
                    "restart": "always",
                    "environment": {
                        "TIMEZONE": SystemManager.get_timezone()
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/glpi/conf:/var/www/html/config",
                        f"{settings.DATA_DIR}/glpi/files:/var/www/html/files",
                        f"{settings.DATA_DIR}/glpi/plugins:/var/www/html/plugins"
                    ],
                    "ports": ["8090:80"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(
                        high_mem="1G", high_cpu="0.5",
                        low_mem="512M", low_cpu="0.5"
                    )
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

    def get_url(self) -> Optional[str]:
        return f"http://{settings.DOMAIN}:8090"

    def get_ports(self) -> List[str]:
        return ["8090/tcp"]
