import pytest
from unittest.mock import patch, MagicMock
from cyl_manager.core.system import SystemManager
from cyl_manager.core.exceptions import SystemRequirementError

def test_check_root_failure():
    with patch("os.geteuid", return_value=1000):
        with pytest.raises(SystemRequirementError):
            SystemManager.check_root()

def test_check_root_success():
    with patch("os.geteuid", return_value=0):
        SystemManager.check_root() # Should not raise

def test_hardware_profile():
    with patch("psutil.virtual_memory") as mock_mem:
        with patch("psutil.cpu_count") as mock_cpu:
            mock_mem.return_value.total = 8 * (1024**3) # 8GB
            mock_cpu.return_value = 4
            assert SystemManager.get_hardware_profile() == "HIGH"

            mock_mem.return_value.total = 2 * (1024**3) # 2GB
            mock_cpu.return_value = 2
            assert SystemManager.get_hardware_profile() == "LOW"
