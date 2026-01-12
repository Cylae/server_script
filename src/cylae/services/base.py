from abc import ABC, abstractmethod

class Service(ABC):
    def __init__(self, name):
        self.name = name

    @abstractmethod
    def install(self):
        """Installs the service."""
        pass

    @abstractmethod
    def remove(self, delete_data=False):
        """Removes the service."""
        pass

    @abstractmethod
    def status(self):
        """Returns the status of the service."""
        pass
