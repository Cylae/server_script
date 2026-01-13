from ..services.base import BaseService
from ..core.registry import registry
from ..core.config import config

@registry.register
class TautulliService(BaseService):
    @property
    def name(self): return "tautulli"
    @property
    def pretty_name(self): return "Tautulli"
    @property
    def image(self): return "lscr.io/linuxserver/tautulli:latest"
    @property
    def ports(self): return {"8181": "8181"}
    @property
    def volumes(self): return {f"{self.data_dir}/config": "/config"}

@registry.register
class ProwlarrService(BaseService):
    @property
    def name(self): return "prowlarr"
    @property
    def pretty_name(self): return "Prowlarr"
    @property
    def image(self): return "lscr.io/linuxserver/prowlarr:latest"
    @property
    def ports(self): return {"9696": "9696"}
    @property
    def volumes(self): return {f"{self.data_dir}/config": "/config"}

@registry.register
class OverseerrService(BaseService):
    @property
    def name(self): return "overseerr"
    @property
    def pretty_name(self): return "Overseerr"
    @property
    def image(self): return "lscr.io/linuxserver/overseerr:latest"
    @property
    def ports(self): return {"5055": "5055"}
    @property
    def volumes(self): return {f"{self.data_dir}/config": "/config"}

@registry.register
class QbittorrentService(BaseService):
    @property
    def name(self): return "qbittorrent"
    @property
    def pretty_name(self): return "qBittorrent"
    @property
    def image(self): return "lscr.io/linuxserver/qbittorrent:latest"
    @property
    def ports(self): return {"8080": "8080", "6881": "6881"}
    @property
    def volumes(self): return {
        f"{self.data_dir}/config": "/config",
        "/opt/media/downloads": "/downloads"
    }
