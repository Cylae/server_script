from abc import ABC, abstractmethod

class BaseService(ABC):
    def __init__(self):
        pass

    @property
    @abstractmethod
    def name(self):
        pass

    @property
    @abstractmethod
    def display_name(self):
        pass

    @abstractmethod
    def install(self):
        pass

    @abstractmethod
    def uninstall(self):
        pass

    @abstractmethod
    def is_installed(self):
        pass
