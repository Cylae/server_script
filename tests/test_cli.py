import pytest
from typer.testing import CliRunner
from unittest.mock import patch, MagicMock
from cyl_manager.cli import app

runner = CliRunner()

@pytest.fixture
def mock_registry():
    with patch("cyl_manager.cli.ServiceRegistry") as mock:
        yield mock

@pytest.fixture
def mock_system_manager():
    with patch("cyl_manager.cli.SystemManager") as mock:
        mock.check_root.return_value = True
        yield mock

@pytest.fixture
def mock_docker_manager():
    with patch("cyl_manager.cli.DockerManager") as mock:
        yield mock

def test_install_service_not_found(mock_registry, mock_system_manager, mock_docker_manager):
    mock_registry.get.return_value = None
    result = runner.invoke(app, ["install", "unknown_service"])
    assert result.exit_code == 1

def test_install_service_success(mock_registry, mock_system_manager, mock_docker_manager):
    mock_service_cls = MagicMock()
    mock_service_instance = mock_service_cls.return_value
    mock_service_instance.is_installed = False
    mock_service_instance.pretty_name = "My Service"
    mock_service_instance.get_install_summary.return_value = "Installation successful"
    mock_registry.get.return_value = mock_service_cls

    result = runner.invoke(app, ["install", "myservice"])
    assert result.exit_code == 0
    mock_service_instance.install.assert_called_once()

def test_install_service_already_installed(mock_registry, mock_system_manager, mock_docker_manager):
    mock_service_cls = MagicMock()
    mock_service_instance = mock_service_cls.return_value
    mock_service_instance.is_installed = True
    mock_service_instance.pretty_name = "My Service"
    mock_registry.get.return_value = mock_service_cls

    result = runner.invoke(app, ["install", "myservice"])
    assert result.exit_code == 0
    # Should log info but not install
    mock_service_instance.install.assert_not_called()

def test_remove_service_success(mock_registry, mock_system_manager, mock_docker_manager):
    mock_service_cls = MagicMock()
    mock_service_instance = mock_service_cls.return_value
    mock_service_instance.is_installed = True
    mock_service_instance.pretty_name = "My Service"
    mock_registry.get.return_value = mock_service_cls

    result = runner.invoke(app, ["remove", "myservice"])
    assert result.exit_code == 0
    mock_service_instance.remove.assert_called_once()

def test_status_command(mock_registry, mock_system_manager, mock_docker_manager):
    mock_service_cls = MagicMock()
    mock_service_instance = mock_service_cls.return_value
    # mock_service_instance.is_installed = True # This is now determined by get_all_container_names
    mock_service_instance.name = "myservice"
    mock_service_instance.pretty_name = "My Service"
    mock_service_instance.get_url.return_value = "http://localhost"

    mock_registry.get_all.return_value = {"myservice": mock_service_cls}

    # Mock DockerManager.get_all_container_names
    mock_docker_instance = mock_docker_manager.return_value
    mock_docker_instance.get_all_container_names.return_value = {"myservice"}

    result = runner.invoke(app, ["status"])
    assert result.exit_code == 0
    assert "Service Status" in result.stdout
    assert "myservice" in result.stdout
    assert "Installed" in result.stdout

def test_root_check_fail():
    with patch("cyl_manager.cli.SystemManager") as mock_sys:
        mock_sys.check_root.side_effect = Exception("Root required")
        result = runner.invoke(app, ["status"])
        assert result.exit_code == 1
