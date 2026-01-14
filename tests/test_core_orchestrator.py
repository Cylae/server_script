import pytest
from unittest.mock import MagicMock, patch
from cyl_manager.core.orchestrator import InstallationOrchestrator

# Mocking DockerManager to prevent real connection attempts and ServiceError
@patch("cyl_manager.core.orchestrator.DockerManager")
@patch("cyl_manager.core.orchestrator.SystemManager")
@patch("cyl_manager.core.orchestrator.ThreadPoolExecutor")
def test_orchestrator_concurrency(mock_executor, mock_system, mock_docker_cls):
    # Setup SystemManager mocks
    mock_system.get_concurrency_limit.return_value = 2
    mock_system.get_hardware_profile.return_value = "HIGH"

    # Setup DockerManager mock instance
    mock_docker_instance = MagicMock()
    mock_docker_cls.return_value = mock_docker_instance

    # Create fake service objects
    s1 = MagicMock()
    s1.name = "s1"
    s1.pretty_name = "Service 1"

    s2 = MagicMock()
    s2.name = "s2"
    s2.pretty_name = "Service 2"

    services = [s1, s2]

    InstallationOrchestrator.install_services(services)

    # Verify DockerManager.ensure_network was called
    mock_docker_instance.ensure_network.assert_called_once()

    # Verify ThreadPoolExecutor was initialized with correct workers
    mock_executor.assert_called_with(max_workers=2)

@patch("cyl_manager.core.orchestrator.DockerManager")
@patch("cyl_manager.core.orchestrator.SystemManager")
def test_orchestrator_execution(mock_system, mock_docker_cls):
    # Setup SystemManager mocks
    mock_system.get_concurrency_limit.return_value = 1
    mock_system.get_hardware_profile.return_value = "LOW"

    # Setup DockerManager mock instance
    mock_docker_instance = MagicMock()
    mock_docker_cls.return_value = mock_docker_instance

    s1 = MagicMock()
    s1.name = "s1"
    s1.pretty_name = "Service 1"

    s2 = MagicMock()
    s2.name = "s2"
    s2.pretty_name = "Service 2"

    services = [s1, s2]

    InstallationOrchestrator.install_services(services)

    # Verify installs were called
    assert s1.install.called
    assert s2.install.called
