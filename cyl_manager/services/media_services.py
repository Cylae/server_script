from ..services.base import BaseService
from ..core.registry import registry
from ..core.config import config

@registry.register
class PlexService(BaseService):
    @property
    def name(self): return "plex"

    @property
    def pretty_name(self): return "Plex Media Server"

    @property
    def image(self): return "lscr.io/linuxserver/plex:latest"

    @property
    def ports(self):
        return {"32400": "32400"}

    @property
    def volumes(self):
        return {
            f"{self.data_dir}/config": "/config",
            "/opt/media/tv": "/tv",
            "/opt/media/movies": "/movies"
        }

    @property
    def env_vars(self):
        vars = super().env_vars
        vars["VERSION"] = "docker"
        return vars

@registry.register
class SonarrService(BaseService):
    @property
    def name(self): return "sonarr"

    @property
    def pretty_name(self): return "Sonarr"

    @property
    def image(self): return "lscr.io/linuxserver/sonarr:latest"

    @property
    def ports(self):
        return {"8989": "8989"}

    @property
    def volumes(self):
        return {
            f"{self.data_dir}/config": "/config",
            "/opt/media": "/data" # Unified media path
        }

@registry.register
class RadarrService(BaseService):
    @property
    def name(self): return "radarr"

    @property
    def pretty_name(self): return "Radarr"

    @property
    def image(self): return "lscr.io/linuxserver/radarr:latest"

    @property
    def ports(self):
        return {"7878": "7878"}

    @property
    def volumes(self):
        return {
            f"{self.data_dir}/config": "/config",
            "/opt/media": "/data"
        }
