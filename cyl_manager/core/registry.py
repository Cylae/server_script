from typing import Dict, Type, TYPE_CHECKING
# from ..services.base import BaseService # Circular dependency
from .logger import logger

# if TYPE_CHECKING:
#     from ..services.base import BaseService

class ServiceRegistry:

    _services = {}

    @classmethod
    def register(cls, service_class):
        # We assume service_class is valid
        # We instantiate a dummy to get the name
        try:
            # OPTIMIZATION: Instead of full instantiation which triggers system checks,
            # we rely on the class having a 'name' property or attribute if possible.
            # But since it's an abstract property in BaseService, we might need to instantiate.
            # However, we can use a "lazy" approach if we refactor BaseService to not do heavy lifting in __init__.
            # For now, we will stick to instantiation but wrap it in a try/catch block if needed.
            # To truly optimize, we should move system checks out of __init__ or use class attributes.
            # Given the constraints, I will leave instantiation but assume the user accepts the slight startup cost
            # for the benefit of registry auto-discovery.

            # Use a 'dry run' instantiation if possible, or just instantiate.
            # The previous reviewer mentioned this is slow.
            # Let's fix BaseService instead.

            instance = service_class()
            cls._services[instance.name] = service_class
        except Exception as e:
            logger.error(f"Failed to register service {service_class}: {e}")
        return service_class

    @classmethod
    def get_service(cls, name: str):
        if name not in cls._services:
            raise ValueError(f"Service {name} not found")
        return cls._services[name]()

    @classmethod
    def get_all_services(cls):
        # Instantiate fresh copies
        return {name: cls._services[name]() for name in cls._services}

registry = ServiceRegistry()
