from ..services.base import BaseService
from ..core.registry import registry
from ..core.config import config

@registry.register
class WireGuardService(BaseService):
    @property
    def name(self): return "wireguard"
    @property
    def pretty_name(self): return "WireGuard"
    @property
    def image(self): return "lscr.io/linuxserver/wireguard:latest"
    @property
    def ports(self): return {"51820": "51820/udp"}
    @property
    def volumes(self): return {
        f"{self.data_dir}/config": "/config",
        "/lib/modules": "/lib/modules"
    }
    @property
    def env_vars(self):
        vars = super().env_vars
        vars["PEERS"] = "1"
        vars["SERVERURL"] = f"vpn.{config.DOMAIN}"
        return vars

@registry.register
class UptimeKumaService(BaseService):
    @property
    def name(self): return "uptimekuma"
    @property
    def pretty_name(self): return "Uptime Kuma"
    @property
    def image(self): return "louislam/uptime-kuma:1"
    @property
    def ports(self): return {"3001": "3001"}
    @property
    def volumes(self): return {f"{self.data_dir}/data": "/app/data"}

@registry.register
class VaultwardenService(BaseService):
    @property
    def name(self): return "vaultwarden"
    @property
    def pretty_name(self): return "Vaultwarden"
    @property
    def image(self): return "vaultwarden/server:latest"
    @property
    def ports(self): return {"8000": "80"}
    @property
    def volumes(self): return {f"{self.data_dir}/data": "/data"}

@registry.register
class FileBrowserService(BaseService):
    @property
    def name(self): return "filebrowser"
    @property
    def pretty_name(self): return "FileBrowser"
    @property
    def image(self): return "filebrowser/filebrowser:latest"
    @property
    def ports(self): return {"8081": "80"}
    @property
    def volumes(self): return {
        f"{self.data_dir}/data": "/srv",
        f"{self.data_dir}/config.json": "/etc/config.json",
        f"{self.data_dir}/database.db": "/etc/database.db"
    }

@registry.register
class MailService(BaseService):
    @property
    def name(self): return "mailserver"
    @property
    def pretty_name(self): return "Docker MailServer"
    @property
    def image(self): return "mailserver/docker-mailserver:latest"
    @property
    def ports(self): return {
        "25": "25",
        "143": "143",
        "587": "587",
        "993": "993"
    }
    @property
    def volumes(self): return {
        f"{self.data_dir}/mail-data": "/var/mail",
        f"{self.data_dir}/mail-state": "/var/mail-state",
        f"{self.data_dir}/config": "/tmp/docker-mailserver"
    }
