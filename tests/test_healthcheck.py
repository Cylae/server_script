import pytest
from unittest.mock import patch, MagicMock
from cyl_manager.services.base import BaseService
from cyl_manager.core.exceptions import ServiceError

class MockService(BaseService):
    name = "mock_service"
    pretty_name = "Mock Service"
    def __init__(self):
        self.docker = MagicMock()
        self.system = MagicMock()
        self.profile = "HIGH"
    def generate_compose(self):
        return {}

def test_wait_for_health_success():
    svc = MockService()
    # Mock subprocess.run to return healthy
    with patch("subprocess.run") as mock_run:
        mock_run.return_value.returncode = 0
        mock_run.return_value.stdout = '{"Status": "running", "Health": {"Status": "healthy"}}'

        assert svc.wait_for_health(timeout=1) == True

def test_wait_for_health_no_check():
    svc = MockService()
    with patch("subprocess.run") as mock_run:
        mock_run.return_value.returncode = 0
        # No Health key means no check, so it assumes healthy if running
        mock_run.return_value.stdout = '{"Status": "running"}'

        assert svc.wait_for_health(timeout=1) == True

def test_wait_for_health_timeout():
    svc = MockService()
    with patch("subprocess.run") as mock_run:
        mock_run.return_value.returncode = 0
        mock_run.return_value.stdout = '{"Status": "running", "Health": {"Status": "starting"}}'

        with patch("time.sleep", return_value=None): # Speed up test
            with pytest.raises(ServiceError, match="Timeout"):
                svc.wait_for_health(timeout=0.1)

def test_wait_for_health_failure():
    svc = MockService()
    with patch("subprocess.run") as mock_run:
        mock_run.return_value.returncode = 0
        mock_run.return_value.stdout = '{"Status": "dead"}'

        with patch("time.sleep", return_value=None):
             with pytest.raises(ServiceError, match="crashed"):
                svc.wait_for_health(timeout=1)
