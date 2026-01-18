
import unittest
from unittest.mock import patch, MagicMock
import sys
import os

# Add src to sys.path
sys.path.append(os.path.abspath("src"))

from cyl_manager.core.system import SystemManager
from cyl_manager.services.media import PlexService, SonarrService
from cyl_manager.services.more_services import MailService

class TestArchitecturalRequirements(unittest.TestCase):
    def setUp(self):
        # Patch DockerManager and FirewallManager to prevent side effects
        self.docker_patcher = patch('cyl_manager.services.base.DockerManager')
        self.mock_docker = self.docker_patcher.start()
        self.firewall_patcher = patch('cyl_manager.services.base.FirewallManager')
        self.mock_firewall = self.firewall_patcher.start()

        # Reset hardware profile cache
        SystemManager._hardware_profile = None

    def tearDown(self):
        self.docker_patcher.stop()
        self.firewall_patcher.stop()
        SystemManager._hardware_profile = None

    @patch('cyl_manager.core.system.psutil')
    def test_2vcpu_triggers_low_profile(self, mock_psutil):
        """
        The "2 vCPU" Benchmark: The script must pass a deployment test on a limited 2 vCPU environment.
        Verify that 2 vCPUs triggers LOW profile.
        """
        # Mock sufficient RAM but low CPU
        mock_mem = MagicMock()
        mock_mem.total = 8 * 1024**3 # 8GB (High)
        mock_psutil.virtual_memory.return_value = mock_mem

        mock_swap = MagicMock()
        mock_swap.total = 2 * 1024**3 # 2GB (High)
        mock_psutil.swap_memory.return_value = mock_swap

        # Mock 2 CPUs
        mock_psutil.cpu_count.return_value = 2

        profile = SystemManager.get_hardware_profile()
        self.assertEqual(profile, SystemManager.PROFILE_LOW, "2 vCPUs should trigger LOW profile")

    @patch('cyl_manager.core.system.SystemManager.get_hardware_profile')
    def test_plex_optimization(self, mock_get_profile):
        """
        Plex: Adjust transcoding buffers based on available resources.
        """
        # Case 1: LOW Profile
        mock_get_profile.return_value = SystemManager.PROFILE_LOW
        plex_low = PlexService()
        compose_low = plex_low.generate_compose()
        volumes_low = compose_low['services']['plex']['volumes']

        # Check for disk transcoding
        has_disk_transcode = any("plex/transcode:/transcode" in v for v in volumes_low)
        self.assertTrue(has_disk_transcode, "LOW profile should use disk for transcoding")
        self.assertFalse(any("/tmp:/transcode" in v for v in volumes_low), "LOW profile should NOT use /tmp for transcoding")

        # Case 2: HIGH Profile
        mock_get_profile.return_value = SystemManager.PROFILE_HIGH
        plex_high = PlexService()
        compose_high = plex_high.generate_compose()
        volumes_high = compose_high['services']['plex']['volumes']

        # Check for RAM transcoding
        has_ram_transcode = any("/tmp:/transcode" in v for v in volumes_high)
        self.assertTrue(has_ram_transcode, "HIGH profile should use /tmp (RAM) for transcoding")

    @patch('cyl_manager.core.system.SystemManager.get_hardware_profile')
    def test_starr_optimization(self, mock_get_profile):
        """
        Starr Apps: Optimize startup parameters.
        """
        # Profile doesn't matter for this specific check as it applies to all,
        # but resource limits do change.
        mock_get_profile.return_value = SystemManager.PROFILE_HIGH

        sonarr = SonarrService()
        compose = sonarr.generate_compose()
        env = compose['services']['sonarr']['environment']

        self.assertIn('COMPlus_EnableDiagnostics', env)
        self.assertEqual(env['COMPlus_EnableDiagnostics'], "0", "Starr apps should disable .NET diagnostics")

    @patch('cyl_manager.core.system.SystemManager.get_hardware_profile')
    def test_concurrency_serialization(self, mock_get_profile):
        """
        Concurrency: Serialize installations on low-end hardware.
        """
        # Case 1: LOW Profile
        mock_get_profile.return_value = SystemManager.PROFILE_LOW
        limit_low = SystemManager.get_concurrency_limit()
        self.assertEqual(limit_low, 1, "LOW profile should have concurrency limit of 1")

        # Case 2: HIGH Profile
        mock_get_profile.return_value = SystemManager.PROFILE_HIGH
        limit_high = SystemManager.get_concurrency_limit()
        self.assertEqual(limit_high, 4, "HIGH profile should have concurrency limit of 4")

if __name__ == '__main__':
    unittest.main()
