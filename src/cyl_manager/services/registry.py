from typing import Dict, Type, Optional
from cyl_manager.services.base import BaseService
from cyl_manager.core.logging import logger

class ServiceRegistry:
    """
    Registry for tracking all available services.
    """
    _services: Dict[str, Type[BaseService]] = {}

    @classmethod
    def register(cls, service_cls: Type[BaseService]) -> Type[BaseService]:
        """
        Decorator to register a service class.
        """
        if service_cls.name in cls._services:
            logger.warning(f"Service '{service_cls.name}' is being registered twice.")

        cls._services[service_cls.name] = service_cls
        return service_cls

    @classmethod
    def get_all(cls) -> Dict[str, Type[BaseService]]:
        """
        Returns a dictionary of all registered services.
        """
        return cls._services.copy()

    @classmethod
    def get(cls, name: str) -> Optional[Type[BaseService]]:
        """
        Retrieves a service class by name.
        """
        return cls._services.get(name)

    @classmethod
    def get_instance(cls, name: str) -> Optional[BaseService]:
        """
        Instantiates and returns a service by name.
        """
        service_cls = cls.get(name)
        if service_cls:
            return service_cls()
        return None
