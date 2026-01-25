import unittest
from unittest.mock import MagicMock, patch
import json
from cyl_manager.web.app import app
from cyl_manager.core.system import SystemManager
from cyl_manager.core.hardware import HardwareManager

class TestWebRoutes(unittest.TestCase):
    def setUp(self):
        self.app = app.test_client()
        self.app.testing = True
        HardwareManager._hardware_profile = None

    @patch("cyl_manager.web.app.psutil")
    @patch("cyl_manager.core.hardware.psutil")
    def test_scenario_d_web_status(self, mock_sys_psutil, mock_web_psutil):
        """Scenario D (Web): Mock a GET request to /status -> Assert it returns JSON with current hardware stats and selected profile."""

        # Configure Mocks for SystemManager (for profile detection)
        mock_sys_mem = MagicMock()
        mock_sys_mem.total = 8 * 1024**3 # High spec
        mock_sys_psutil.virtual_memory.return_value = mock_sys_mem
        mock_sys_psutil.cpu_count.return_value = 4
        mock_sys_swap = MagicMock()
        mock_sys_swap.total = 2 * 1024**3
        mock_sys_psutil.swap_memory.return_value = mock_sys_swap

        # Configure Mocks for Web App (for realtime stats)
        mock_web_mem = MagicMock()
        mock_web_mem.total = 8 * 1024**3
        mock_web_mem.used = 2 * 1024**3
        mock_web_mem.percent = 25.0
        mock_web_psutil.virtual_memory.return_value = mock_web_mem

        mock_web_swap = MagicMock()
        mock_web_swap.total = 2 * 1024**3
        mock_web_swap.used = 0
        mock_web_psutil.swap_memory.return_value = mock_web_swap

        mock_web_psutil.cpu_count.return_value = 4
        mock_web_psutil.cpu_percent.return_value = 10.5

        # Perform Request
        response = self.app.get('/status')

        self.assertEqual(response.status_code, 200)
        data = json.loads(response.data)

        # Assertions
        self.assertEqual(data['profile'], SystemManager.PROFILE_HIGH)
        self.assertEqual(data['hardware']['ram_total_gb'], 8.0)
        self.assertEqual(data['hardware']['cpu_cores'], 4)

if __name__ == "__main__":
    unittest.main()
