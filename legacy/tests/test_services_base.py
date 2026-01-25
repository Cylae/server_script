import pytest
from unittest.mock import MagicMock, patch, mock_open
from cyl_manager.services.base import BaseService
from cyl_manager.core.system import SystemManager
from cyl_manager.core.exceptions import ServiceError

# Create a concrete implementation for testing
class MockServiceImplementation(BaseService):
    name = "test_service"
    pretty_name = "Test Service"
    def generate_compose(self):
        return {"services": {"test": {"image": "test"}}}

@pytest.fixture
def mock_docker():
    with patch("cyl_manager.services.base.DockerManager") as m:
        yield m

@pytest.fixture
def mock_system():
    # Only patch methods, not attributes/constants
    with patch("cyl_manager.services.base.SystemManager.get_hardware_profile") as mock_profile, \
         patch("cyl_manager.services.base.SystemManager.get_uid_gid") as mock_uid, \
         patch("cyl_manager.services.base.SystemManager.get_timezone") as mock_tz, \
         patch("cyl_manager.services.base.SystemManager.run_command") as mock_run:

        mock_uid.return_value = ("1000", "1000")
        mock_tz.return_value = "UTC"
        yield mock_profile

@pytest.fixture
def mock_settings():
    with patch("cyl_manager.services.base.settings") as m:
        m.DATA_DIR = "/tmp/data"
        yield m

def test_service_initialization(mock_docker, mock_system):
    mock_system.return_value = "HIGH"
    svc = MockServiceImplementation()
    assert svc.name == "test_service"
    assert svc.profile == "HIGH"
    assert mock_docker.called

def test_resource_limits_high(mock_docker, mock_system):
    mock_system.return_value = "HIGH"
    svc = MockServiceImplementation()

    # We rely on MockServiceImplementation calling SystemManager.get_hardware_profile() during init
    # which returns "HIGH".
    # And get_resource_limits checks self.profile == SystemManager.PROFILE_HIGH ("HIGH")

    limits = svc.get_resource_limits(high_mem="2G", low_mem="1G")
    assert limits["resources"]["limits"]["memory"] == "2G"

def test_resource_limits_low(mock_docker, mock_system):
    mock_system.return_value = "LOW"
    svc = MockServiceImplementation()

    limits = svc.get_resource_limits(high_mem="2G", low_mem="1G")
    assert limits["resources"]["limits"]["memory"] == "1G"
