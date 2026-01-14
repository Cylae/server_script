from typing import Dict, Any
import secrets
import string
from .base import BaseService
from .registry import ServiceRegistry
from ..core.config import settings, save_settings

@ServiceRegistry.register
class MariaDBService(BaseService):
    name = "mariadb"
    pretty_name = "MariaDB Database"

    def _generate_password(self, length=24):
        alphabet = string.ascii_letters + string.digits
        return ''.join(secrets.choice(alphabet) for i in range(length))

    def generate_compose(self) -> Dict[str, Any]:
        # Get or generate passwords
        root_password = settings.MYSQL_ROOT_PASSWORD
        if not root_password:
            root_password = self._generate_password()
            save_settings("MYSQL_ROOT_PASSWORD", root_password)

        user_password = settings.MYSQL_USER_PASSWORD
        if not user_password:
            user_password = self._generate_password()
            save_settings("MYSQL_USER_PASSWORD", user_password)

        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "lscr.io/linuxserver/mariadb:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "environment": {
                        **self.get_common_env(),
                        "MYSQL_ROOT_PASSWORD": root_password,
                        "MYSQL_DATABASE": "cylae",
                        "MYSQL_USER": "cylae",
                        "MYSQL_PASSWORD": user_password
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/mariadb:/config"
                    ],
                    "ports": ["3306:3306"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(high_mem="1G", low_mem="512M")
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

@ServiceRegistry.register
class NginxProxyService(BaseService):
    name = "nginx-proxy"
    pretty_name = "Nginx Proxy Manager"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "version": "3",
            "services": {
                self.name: {
                    "image": "jc21/nginx-proxy-manager:latest",
                    "container_name": self.name,
                    "restart": "unless-stopped",
                    "environment": {
                        "DB_SQLITE_FILE": "/data/database.sqlite",
                        "DISABLE_IPV6": "true"
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/npm/data:/data",
                        f"{settings.DATA_DIR}/npm/letsencrypt:/etc/letsencrypt"
                    ],
                    "ports": [
                        "80:80",
                        "81:81",
                        "443:443"
                    ],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(high_mem="512M", low_mem="256M")
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }
