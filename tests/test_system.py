import pytest
from unittest.mock import patch, MagicMock
from cyl_manager.core.system import SystemManager
from cyl_manager.core.exceptions import SystemRequirementError

@pytest.fixture(autouse=True)
def reset_system_manager():
    SystemManager._hardware_profile = None
    yield
    SystemManager._hardware_profile = None

def test_check_root_failure():
    with patch("os.geteuid", return_value=1000):
        with pytest.raises(SystemRequirementError):
            SystemManager.check_root()

def test_check_root_success():
    with patch("os.geteuid", return_value=0):
        SystemManager.check_root() # Should not raise

def test_hardware_profile():
    # Patch the psutil used by HardwareManager
    with patch("cyl_manager.core.hardware.psutil.virtual_memory") as mock_mem:
        with patch("cyl_manager.core.hardware.psutil.cpu_count") as mock_cpu:
            with patch("cyl_manager.core.hardware.psutil.swap_memory") as mock_swap:
                mock_mem.return_value.total = 8 * (1024**3) # 8GB
                mock_cpu.return_value = 4
                mock_swap.return_value.total = 4 * (1024**3) # 4GB Swap

                # Reset cache before calling
                from cyl_manager.core.hardware import HardwareManager
                HardwareManager._hardware_profile = None

                assert SystemManager.get_hardware_profile() == "HIGH"

                mock_mem.return_value.total = 2 * (1024**3) # 2GB
                mock_cpu.return_value = 2
                # Reset cache before calling again, otherwise it returns cached "HIGH"
                HardwareManager._hardware_profile = None
                assert SystemManager.get_hardware_profile() == "LOW"
