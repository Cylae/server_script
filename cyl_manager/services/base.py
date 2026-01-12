from abc import ABC, abstractmethod
from ..core.docker import deploy_service, remove_service, is_installed
from ..core.config import get

class BaseService(ABC):
    def __init__(self):
        self.domain = get("DOMAIN")
        self.docker_net = get("DOCKER_NET")

    @abstractmethod
    def install(self):
        pass

    @abstractmethod
    def remove(self):
        pass

    def is_installed(self):
        return is_installed(self.name)
