import pytest
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.services.base import BaseService
from cyl_manager.services.portainer import PortainerService
from unittest.mock import patch

def test_registry_registration():
    assert "portainer" in ServiceRegistry.get_all()
    assert ServiceRegistry.get("portainer") == PortainerService

def test_registry_instantiation():
    with patch("docker.from_env"):
        instance = ServiceRegistry.get_instance("portainer")
        assert isinstance(instance, PortainerService)
