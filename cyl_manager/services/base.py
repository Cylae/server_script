from abc import ABC, abstractmethod
from ..core.docker import deploy_service, remove_service, is_installed
from ..core.config import get
from ..core.system import determine_profile

class BaseService(ABC):
    def __init__(self):
        self.domain = get("DOMAIN")
        self.docker_net = get("DOCKER_NET")
        self.profile = determine_profile()

    @abstractmethod
    def install(self):
        pass

    @abstractmethod
    def remove(self):
        pass

    def is_installed(self):
        return is_installed(self.name)

    def get_resource_limit(self, default_high="2048M", default_low="512M"):
        """Returns the memory limit based on the system profile."""
        if self.profile == "HIGH":
            return default_high
        return default_low
