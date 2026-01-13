from ..services.base import BaseService
from ..core.registry import registry
from ..core.config import config

@registry.register
class GiteaService(BaseService):
    @property
    def name(self): return "gitea"
    @property
    def pretty_name(self): return "Gitea"
    @property
    def image(self): return "gitea/gitea:latest"
    @property
    def ports(self): return {"3000": "3000", "222": "22"}
    @property
    def volumes(self): return {f"{self.data_dir}": "/data"}

@registry.register
class PortainerService(BaseService):
    @property
    def name(self): return "portainer"
    @property
    def pretty_name(self): return "Portainer"
    @property
    def image(self): return "portainer/portainer-ce:latest"
    @property
    def ports(self): return {"9000": "9000"}
    @property
    def volumes(self): return {
        "/var/run/docker.sock": "/var/run/docker.sock",
        f"{self.data_dir}": "/data"
    }

@registry.register
class NextcloudService(BaseService):
    @property
    def name(self): return "nextcloud"
    @property
    def pretty_name(self): return "Nextcloud"
    @property
    def image(self): return "lscr.io/linuxserver/nextcloud:latest"
    @property
    def ports(self): return {"443": "443"}
    @property
    def volumes(self): return {
        f"{self.data_dir}/config": "/config",
        f"{self.data_dir}/data": "/data"
    }
