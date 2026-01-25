from typing import Dict, Any, Final, Optional, List
import subprocess
from cyl_manager.services.base import BaseService
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.config import settings
from cyl_manager.core.system import SystemManager
from cyl_manager.core.logging import logger

@ServiceRegistry.register
class MailService(BaseService):
    name: str = "mailserver"
    pretty_name: str = "Mail Server (Docker Mailserver)"

    def install(self) -> None:
        """
        Overrides install to pre-check for port conflicts (specifically port 25).
        """
        logger.info("Checking for port 25 conflicts...")
        # Check if port 25 is in use by common MTAs
        # We can't easily check actual port binding without lsof/netstat which might not be installed
        # So we check for common service names.
        conflicting_services = ["postfix", "exim4", "sendmail", "exim"]

        for service in conflicting_services:
            try:
                # Check if service is active
                cmd = ["systemctl", "is-active", "--quiet", service]
                # If return code is 0, it's active.
                if subprocess.call(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
                    logger.warning(f"Detected conflicting service: {service}")
                    logger.info(f"Stopping and disabling {service} to free port 25...")

                    # Stop the service
                    SystemManager.run_command(["systemctl", "stop", service], check=False)
                    # Disable it
                    SystemManager.run_command(["systemctl", "disable", service], check=False)
                    logger.info(f"{service} stopped and disabled.")
            except Exception as e:
                # systemctl might not exist or other error, ignore and hope for the best
                logger.debug(f"Error checking service {service}: {e}")

        super().install()

    def generate_compose(self) -> Dict[str, Any]:
        # Optimization: Apply "Survival Mode" heuristics for Mailserver.
        # On LOW profile, we disable ClamAV and SpamAssassin to prevent the
        # "Infinite Wait Loop" caused by OOM kills during startup.
        is_low = self.is_low_spec()
        enable_heavy_procs = "1" if not is_low else "0"

        # On low spec, we also disable Fail2Ban to save RAM, unless explicitly requested otherwise.
        # Fail2Ban uses python/gamin and can consume 100-200MB+ which is critical on 1GB VPS.
        # However, for security, some might want it. We'll default to 0 for LOW profile to ensure stability first.
        enable_fail2ban = "1" if not is_low else "0"

        return {
            "services": {
                self.name: {
                    "image": "docker.io/mailserver/docker-mailserver:latest",
                    "container_name": self.name,
                    "hostname": f"mail.{settings.DOMAIN}",
                    "domainname": settings.DOMAIN,
                    "restart": "always",
                    "stop_grace_period": "1m",
                    "security_opt": self.get_security_opts(),
                    "logging": self.get_logging_config(),
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
            "services": {
                self.name: {
                    "image": "diouxx/glpi",
                    "container_name": self.name,
                    "restart": "always",
                    "security_opt": self.get_security_opts(),
                    "logging": self.get_logging_config(),
                    "environment": {
                        "TIMEZONE": SystemManager.get_timezone()
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/glpi/conf:/var/www/html/config",
                        f"{settings.DATA_DIR}/glpi/files:/var/www/html/files",
                        f"{settings.DATA_DIR}/glpi/plugins:/var/www/html/plugins"
                    ],
                    "ports": ["8088:80"],
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
        return f"http://{settings.DOMAIN}:8088"

    def get_ports(self) -> List[str]:
        return ["8088/tcp"]

@ServiceRegistry.register
class RoundcubeService(BaseService):
    name: str = "roundcube"
    pretty_name: str = "Roundcube Webmail"

    def generate_compose(self) -> Dict[str, Any]:
        return {
            "services": {
                self.name: {
                    "image": "roundcube/roundcubemail:latest",
                    "container_name": self.name,
                    "restart": "always",
                    "security_opt": self.get_security_opts(),
                    "logging": self.get_logging_config(),
                    "environment": {
                        "ROUNDCUBEMAIL_DEFAULT_HOST": "mailserver",
                        "ROUNDCUBEMAIL_SMTP_SERVER": "mailserver",
                        "ROUNDCUBEMAIL_DB_TYPE": "sqlite",
                        "ROUNDCUBEMAIL_DB_DIR": "/var/www/html/db",
                        "ROUNDCUBEMAIL_UPLOAD_MAX_FILESIZE": "10M"
                    },
                    "volumes": [
                        f"{settings.DATA_DIR}/roundcube/db:/var/www/html/db",
                        f"{settings.DATA_DIR}/roundcube/config:/var/www/html/config"
                    ],
                    "ports": ["8090:80"],
                    "networks": [settings.DOCKER_NET],
                    "deploy": self.get_resource_limits(
                        high_mem="512M", high_cpu="0.5",
                        low_mem="256M", low_cpu="0.25"
                    )
                }
            },
            "networks": {settings.DOCKER_NET: {"external": True}}
        }

    def get_install_summary(self) -> Optional[str]:
        return f"URL: http://{settings.DOMAIN}:8090"

    def get_url(self) -> Optional[str]:
        return f"http://{settings.DOMAIN}:8090"

    def get_ports(self) -> List[str]:
        return ["8090/tcp"]
