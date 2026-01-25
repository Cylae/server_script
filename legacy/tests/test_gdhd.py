import unittest
from unittest.mock import MagicMock, patch
from cyl_manager.core.system import SystemManager
from cyl_manager.core.hardware import HardwareManager

class TestGDHDLogic(unittest.TestCase):
    def setUp(self):
        # Reset the cached profile before each test
        HardwareManager._hardware_profile = None

    @patch("cyl_manager.core.hardware.psutil")
    def test_scenario_a_potato(self, mock_psutil):
        """Scenario A (Potato): Mock 1GB RAM -> Assert 'Low-Spec' config."""
        # Mock RAM < 4GB (e.g., 1GB)
        mock_mem = MagicMock()
        mock_mem.total = 1 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem

        # Mock CPU <= 2 (e.g., 1)
        mock_psutil.cpu_count.return_value = 1

        # Mock Swap < 1GB (e.g., 0.5GB)
        mock_swap = MagicMock()
        mock_swap.total = 0.5 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        profile = SystemManager.get_hardware_profile()
        self.assertEqual(profile, SystemManager.PROFILE_LOW, "Should be LOW profile for 1GB RAM")

    @patch("cyl_manager.core.hardware.psutil")
    def test_scenario_b_beast(self, mock_psutil):
        """Scenario B (Beast): Mock 64GB RAM -> Assert 'Performance' config."""
        # Mock RAM > 4GB (e.g., 64GB)
        mock_mem = MagicMock()
        mock_mem.total = 64 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem

        # Mock CPU > 2 (e.g., 8)
        mock_psutil.cpu_count.return_value = 8

        # Mock Swap >= 1GB (e.g., 4GB)
        mock_swap = MagicMock()
        mock_swap.total = 4 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        profile = SystemManager.get_hardware_profile()
        self.assertEqual(profile, SystemManager.PROFILE_HIGH, "Should be HIGH profile for 64GB RAM")

    @patch("cyl_manager.core.hardware.psutil")
    def test_scenario_c_disaster(self, mock_psutil):
        """Scenario C (Disaster): Mock IOError -> Assert Safe Mode defaults."""
        # Simulate an exception during hardware detection
        mock_psutil.virtual_memory.side_effect = IOError("Hardware failure")

        profile = SystemManager.get_hardware_profile()
        self.assertEqual(profile, SystemManager.PROFILE_LOW, "Should default to LOW profile on error")

if __name__ == "__main__":
    unittest.main()
