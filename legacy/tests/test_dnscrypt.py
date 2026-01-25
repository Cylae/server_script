import pytest
from unittest.mock import patch
from cyl_manager.services.infrastructure import DNSCryptService
from cyl_manager.core.system import SystemManager

@pytest.fixture
def mock_settings():
    with patch("cyl_manager.services.base.settings") as m:
        m.DOCKER_NET = "cylae_net"
        m.DATA_DIR = "/opt/cylae/data"
        m.DOMAIN = "example.com"
        yield m

@pytest.fixture
def mock_system():
    with patch("cyl_manager.services.base.SystemManager.get_hardware_profile") as mock_profile, \
         patch("cyl_manager.services.base.SystemManager.get_uid_gid") as mock_uid, \
         patch("cyl_manager.services.base.SystemManager.get_timezone") as mock_tz:

        mock_uid.return_value = ("1000", "1000")
        mock_tz.return_value = "UTC"
        yield mock_profile

@pytest.fixture
def mock_docker():
    with patch("cyl_manager.services.base.DockerManager") as m:
        yield m

def test_service_info(mock_docker, mock_system):
    mock_system.return_value = "HIGH"
    service = DNSCryptService()
    assert service.name == "dnscrypt-proxy"
    assert "DNSCrypt" in service.pretty_name
    assert service.get_ports() == ["5300/udp", "5300/tcp"]
    assert "5300" in service.get_url()

def test_generate_compose_high(mock_docker, mock_system, mock_settings):
    mock_system.return_value = "HIGH"
    service = DNSCryptService()
    compose = service.generate_compose()

    svc_def = compose["services"]["dnscrypt-proxy"]
    assert svc_def["image"] == "lscr.io/linuxserver/dnscrypt-proxy:latest"
    assert "5300:53/udp" in svc_def["ports"]
    assert "5300:53/tcp" in svc_def["ports"]

    # Check resource limits for HIGH profile
    # Expected: 256M High
    limits = svc_def["deploy"]["resources"]["limits"]
    assert limits["memory"] == "256M"

def test_generate_compose_low(mock_docker, mock_system, mock_settings):
    mock_system.return_value = "LOW"
    service = DNSCryptService()
    compose = service.generate_compose()

    svc_def = compose["services"]["dnscrypt-proxy"]

    # Check resource limits for LOW profile
    # Expected: 128M Low
    limits = svc_def["deploy"]["resources"]["limits"]
    assert limits["memory"] == "128M"
