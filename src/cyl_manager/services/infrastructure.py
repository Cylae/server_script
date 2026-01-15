from typing import Dict, Any, Final, Optional, List
import secrets
import string
from rich.prompt import Prompt, Confirm

from cyl_manager.services.base import BaseService
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.config import settings, save_settings

@ServiceRegistry.register
class MariaDBService(BaseService):
    name: str = "mariadb"
    pretty_name: str = "MariaDB Database"

    def _generate_password(self, length: int = 24) -> str:
        alphabet = string.ascii_letters + string.digits
        return ''.join(secrets.choice(alphabet) for _ in range(length))

    def configure(self) -> None:
        """
        Configure MariaDB passwords.
        """
        # Check if passwords are already set, ask if user wants to change them
        if settings.MYSQL_ROOT_PASSWORD or settings.MYSQL_USER_PASSWORD:
             if not Confirm.ask("MariaDB passwords are already configured. Do you want to change them?", default=False):
                 return

        if Confirm.ask("Do you want to manually set MariaDB passwords? (Otherwise random ones will be generated)", default=False):
            root_pw = Prompt.ask("Enter MariaDB Root Password", password=True)
            if root_pw:
                save_settings("MYSQL_ROOT_PASSWORD", root_pw)

            user_pw = Prompt.ask("Enter MariaDB User 'cylae' Password", password=True)
            if user_pw:
                save_settings("MYSQL_USER_PASSWORD", user_pw)
        else:
             # If user chooses random (or defaults), generate them now if missing so summary works
             if not settings.MYSQL_ROOT_PASSWORD:
                 save_settings("MYSQL_ROOT_PASSWORD", self._generate_password())
             if not settings.MYSQL_USER_PASSWORD:
                 save_settings("MYSQL_USER_PASSWORD", self._generate_password())

    def generate_compose(self) -> Dict[str, Any]:
        # Get or generate passwords (if not configured via configure hook, double check here)
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

    def get_install_summary(self) -> Optional[str]:
        return (
            f"Host: {self.name}\n"
            f"Port: 3306\n"
            f"Root Password: {settings.MYSQL_ROOT_PASSWORD}\n"
            f"User: cylae\n"
            f"User Password: {settings.MYSQL_USER_PASSWORD}"
        )

    def get_url(self) -> Optional[str]:
        return "Internal: 3306"

    def get_ports(self) -> List[str]:
        return ["3306/tcp"]

@ServiceRegistry.register
class NginxProxyService(BaseService):
    name: str = "nginx-proxy"
    pretty_name: str = "Nginx Proxy Manager"

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

    def get_install_summary(self) -> Optional[str]:
        return (
            f"Admin Interface: http://{settings.DOMAIN}:81\n"
            "Default Email: admin@example.com\n"
            "Default Password: changeme"
        )

    def get_url(self) -> Optional[str]:
        return f"http://{settings.DOMAIN}:81"

    def get_ports(self) -> List[str]:
        return ["80/tcp", "443/tcp", "81/tcp"]
