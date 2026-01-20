import pytest
from unittest.mock import MagicMock, patch
from cyl_manager.core.system import SystemManager

@pytest.fixture(autouse=True)
def reset_hardware_profile():
    from cyl_manager.core.hardware import HardwareManager
    HardwareManager._hardware_profile = None
    yield
    HardwareManager._hardware_profile = None

@patch("cyl_manager.core.hardware.psutil")
def test_hardware_profile_low(mock_psutil):
    # Mock < 4GB RAM
    mock_mem = MagicMock()
    mock_mem.total = 3 * 1024**3
    mock_psutil.virtual_memory.return_value = mock_mem
    mock_psutil.cpu_count.return_value = 4
    mock_swap = MagicMock()
    mock_swap.total = 2 * 1024**3
    mock_psutil.swap_memory.return_value = mock_swap

    assert SystemManager.get_hardware_profile() == "LOW"

@patch("cyl_manager.core.hardware.psutil")
def test_hardware_profile_high(mock_psutil):
    # Reset cache before calling
    from cyl_manager.core.hardware import HardwareManager
    HardwareManager._hardware_profile = None

    # Mock 8GB RAM, 4 Cores, 2GB Swap
    mock_mem = MagicMock()
    mock_mem.total = 8 * 1024**3
    mock_psutil.virtual_memory.return_value = mock_mem
    mock_psutil.cpu_count.return_value = 4
    mock_swap = MagicMock()
    mock_swap.total = 2 * 1024**3
    mock_psutil.swap_memory.return_value = mock_swap

    assert SystemManager.get_hardware_profile() == "HIGH"

@patch("cyl_manager.core.system.shutil.which")
def test_check_command(mock_which):
    mock_which.return_value = "/usr/bin/docker"
    assert SystemManager.check_command("docker") is True

    mock_which.return_value = None
    assert SystemManager.check_command("nonexistent") is False

@patch("cyl_manager.core.system.subprocess.run")
def test_run_command(mock_run):
    SystemManager.run_command(["ls", "-la"])
    mock_run.assert_called_with(["ls", "-la"], shell=False, check=True, text=True, capture_output=True)
