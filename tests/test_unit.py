import pytest
from cyl_manager.core.config import config
from cyl_manager.core.registry import registry
from cyl_manager.services.base import BaseService
from cyl_manager.services import media_services # Manually import to trigger registration

def test_config_load():
    assert config.DOMAIN is not None
    assert config.DOCKER_NET is not None

def test_registry_registration():
    services = registry.get_all_services()
    assert "plex" in services
    assert "sonarr" in services
    assert isinstance(services["plex"], BaseService)

def test_plex_service_structure():
    plex = registry.get_service("plex")
    assert plex.name == "plex"
    assert plex.image == "lscr.io/linuxserver/plex:latest"
    assert "32400" in plex.ports

def test_template_generation():
    plex = registry.get_service("plex")
    compose = plex.generate_compose()
    assert "image: lscr.io/linuxserver/plex:latest" in compose
    assert "container_name: plex" in compose
