import pytest
from unittest.mock import patch, MagicMock
from cyl_manager.core.orchestrator import InstallationOrchestrator
from cyl_manager.services.base import BaseService

class MockService(BaseService):
    name = "mock_service"
    pretty_name = "Mock Service"

    def __init__(self):
        # Bypass real DockerManager init
        self.docker = MagicMock()
        self.system = MagicMock()
        self.profile = "HIGH"

    def generate_compose(self):
        return {}

    def install(self):
        # We need to override install because we are patching it on the class,
        # but the orchestrator instantiates a NEW instance.
        # Patching on the instance in test is hard because orchestrator creates it.
        pass

def test_orchestrator_parallel_high_profile():
    with patch("cyl_manager.core.system.SystemManager.get_hardware_profile", return_value="HIGH"):
        orch = InstallationOrchestrator()
        assert orch.max_workers == 4

        # We mock the ServiceRegistry.get to return a class that we can spy on?
        # Better: mock the 'install' method of the MockService class.

        with patch("cyl_manager.services.registry.ServiceRegistry.get", side_effect=lambda x: MockService if x == "mock_service" else None):
             with patch.object(MockService, "install", autospec=True) as mock_install:
                 # Need to mock property is_installed on the instance or class
                 with patch.object(MockService, "is_installed", new_callable=MagicMock) as mock_is_installed:
                    # Make is_installed return False (property mock is tricky)
                    # Let's just mock the property on the class using PropertyMock if needed,
                    # but new_callable=PropertyMock is correct usage.
                    # Actually, simple patch.object with return_value might not work for property.
                    pass

                    # Simpler approach: Mock the instance returned by registry
                    mock_instance = MagicMock(spec=MockService)
                    mock_instance.name = "mock_service"
                    mock_instance.pretty_name = "Mock Service"
                    mock_instance.is_installed = False

                    # We need ServiceRegistry.get to return a CLASS that returns our mock_instance
                    mock_class = MagicMock(return_value=mock_instance)

                    with patch("cyl_manager.services.registry.ServiceRegistry.get", return_value=mock_class):
                         orch.install_services(["mock_service"])
                         mock_instance.install.assert_called()

def test_orchestrator_serial_low_profile():
    with patch("cyl_manager.core.system.SystemManager.get_hardware_profile", return_value="LOW"):
        orch = InstallationOrchestrator()
        assert orch.max_workers == 1

        mock_instance = MagicMock(spec=MockService)
        mock_instance.name = "mock_service"
        mock_instance.pretty_name = "Mock Service"
        mock_instance.is_installed = False

        mock_class = MagicMock(return_value=mock_instance)

        with patch("cyl_manager.services.registry.ServiceRegistry.get", return_value=mock_class):
             orch.install_services(["mock_service"])
             mock_instance.install.assert_called()
