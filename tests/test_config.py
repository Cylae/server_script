import pytest
from cyl_manager.core.config import Settings

def test_config_defaults():
    settings = Settings()
    assert settings.DOMAIN == "example.com"
    assert settings.DOCKER_NET == "server-net"
