from typing import Dict, Type
from .base import BaseService
from ..core.logging import logger

class ServiceRegistry:
    _services: Dict[str, Type[BaseService]] = {}

    @classmethod
    def register(cls, service_cls: Type[BaseService]):
        cls._services[service_cls.name] = service_cls
        return service_cls

    @classmethod
    def get_all(cls) -> Dict[str, Type[BaseService]]:
        return cls._services

    @classmethod
    def get(cls, name: str) -> Type[BaseService]:
        return cls._services.get(name)

    @classmethod
    def get_instance(cls, name: str) -> BaseService:
        service_cls = cls.get(name)
        if service_cls:
            return service_cls()
        return None
