
import unittest
from unittest.mock import patch, MagicMock
import sys
import os

# Add src to sys.path
sys.path.append(os.path.abspath("src"))

from cyl_manager.services.media import PlexService, SonarrService
from cyl_manager.core.system import SystemManager

class TestUltimateOptimizations(unittest.TestCase):
    def setUp(self):
        # Patch DockerManager to avoid connection errors
        self.docker_patcher = patch('cyl_manager.services.base.DockerManager')
        self.mock_docker = self.docker_patcher.start()

        # Patch FirewallManager
        self.firewall_patcher = patch('cyl_manager.services.base.FirewallManager')
        self.mock_firewall = self.firewall_patcher.start()

    def tearDown(self):
        self.docker_patcher.stop()
        self.firewall_patcher.stop()

    @patch('cyl_manager.core.system.SystemManager.get_hardware_profile')
    def test_plex_optimization(self, mock_get_profile):
        # Test LOW Profile
        mock_get_profile.return_value = SystemManager.PROFILE_LOW
        service = PlexService()
        compose = service.generate_compose()
        env = compose['services']['plex']['environment']

        self.assertEqual(env.get("PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS"), "2",
                         "Plex Plugin Procs should be 2 on LOW profile")

        # Test HIGH Profile
        mock_get_profile.return_value = SystemManager.PROFILE_HIGH
        service = PlexService()
        compose = service.generate_compose()
        env = compose['services']['plex']['environment']

        self.assertEqual(env.get("PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS"), "6",
                         "Plex Plugin Procs should be 6 on HIGH profile")

    @patch('cyl_manager.core.system.SystemManager.get_hardware_profile')
    def test_arr_optimization(self, mock_get_profile):
        # Test LOW Profile
        mock_get_profile.return_value = SystemManager.PROFILE_LOW
        service = SonarrService()
        compose = service.generate_compose()
        env = compose['services']['sonarr']['environment']

        self.assertEqual(env.get("COMPlus_EnableDiagnostics"), "0",
                         "Diagnostics should be disabled on all profiles")
        self.assertEqual(env.get("COMPlus_GCServer"), "0",
                         "GCServer should be 0 (Workstation GC) on LOW profile")

        # Test HIGH Profile
        mock_get_profile.return_value = SystemManager.PROFILE_HIGH
        service = SonarrService()
        compose = service.generate_compose()
        env = compose['services']['sonarr']['environment']

        self.assertEqual(env.get("COMPlus_EnableDiagnostics"), "0",
                         "Diagnostics should be disabled on all profiles")
        self.assertIsNone(env.get("COMPlus_GCServer"),
                          "GCServer override should NOT be present on HIGH profile (defaulting to Server GC)")

if __name__ == '__main__':
    unittest.main()
