import unittest
from unittest.mock import MagicMock, patch
import os
import sys
from pathlib import Path

# Ensure src is in path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from cyl_manager.core.system import SystemManager
from cyl_manager.services.more_services import MailService
from cyl_manager.services.media import PlexService
from cyl_manager.core.orchestrator import InstallationOrchestrator

class TestArchitect(unittest.TestCase):

    @patch("cyl_manager.core.system.psutil")
    def test_hardware_profile_low(self, mock_psutil):
        # Mock 2GB RAM, 2 Cores
        mock_mem = MagicMock()
        mock_mem.total = 2 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem

        mock_psutil.cpu_count.return_value = 2

        mock_swap = MagicMock()
        mock_swap.total = 0
        mock_psutil.swap_memory.return_value = mock_swap

        self.assertEqual(SystemManager.get_hardware_profile(), "LOW")
        self.assertEqual(SystemManager.get_concurrency_limit(), 1)

    @patch("cyl_manager.core.system.psutil")
    def test_hardware_profile_high(self, mock_psutil):
        # Mock 8GB RAM, 4 Cores
        mock_mem = MagicMock()
        mock_mem.total = 8 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem

        mock_psutil.cpu_count.return_value = 4

        mock_swap = MagicMock()
        mock_swap.total = 2 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        self.assertEqual(SystemManager.get_hardware_profile(), "HIGH")
        self.assertEqual(SystemManager.get_concurrency_limit(), 4)

    @patch("cyl_manager.services.base.SystemManager.get_hardware_profile", return_value="LOW")
    @patch("cyl_manager.services.base.SystemManager.get_uid_gid", return_value=("1000", "1000"))
    @patch("cyl_manager.services.base.SystemManager.get_timezone", return_value="UTC")
    @patch("cyl_manager.services.base.DockerManager")
    def test_mailserver_config_low(self, mock_docker, mock_tz, mock_uid, mock_profile):
        svc = MailService()
        config = svc.generate_compose()
        env = config["services"]["mailserver"]["environment"]
        deploy = config["services"]["mailserver"]["deploy"]

        self.assertEqual(env["ENABLE_CLAMAV"], "0")
        self.assertEqual(env["ENABLE_SPAMASSASSIN"], "0")
        self.assertEqual(deploy["resources"]["limits"]["memory"], "1G")

    @patch("cyl_manager.services.base.SystemManager.get_hardware_profile", return_value="HIGH")
    @patch("cyl_manager.services.base.SystemManager.get_uid_gid", return_value=("1000", "1000"))
    @patch("cyl_manager.services.base.SystemManager.get_timezone", return_value="UTC")
    @patch("cyl_manager.services.base.DockerManager")
    def test_mailserver_config_high(self, mock_docker, mock_tz, mock_uid, mock_profile):
        svc = MailService()
        config = svc.generate_compose()
        env = config["services"]["mailserver"]["environment"]
        deploy = config["services"]["mailserver"]["deploy"]

        self.assertEqual(env["ENABLE_CLAMAV"], "1")
        self.assertEqual(env["ENABLE_SPAMASSASSIN"], "1")
        self.assertEqual(deploy["resources"]["limits"]["memory"], "2G")

    @patch("cyl_manager.services.base.SystemManager.get_hardware_profile", return_value="LOW")
    @patch("cyl_manager.services.base.SystemManager.get_uid_gid", return_value=("1000", "1000"))
    @patch("cyl_manager.services.base.SystemManager.get_timezone", return_value="UTC")
    @patch("cyl_manager.services.base.DockerManager")
    def test_plex_config_low(self, mock_docker, mock_tz, mock_uid, mock_profile):
        svc = PlexService()
        config = svc.generate_compose()
        volumes = config["services"]["plex"]["volumes"]

        # Check if transcode is NOT /tmp:/transcode
        has_tmp_transcode = False
        for vol in volumes:
            if vol == "/tmp:/transcode":
                has_tmp_transcode = True

        self.assertFalse(has_tmp_transcode, "Plex should not use RAM transcoding on LOW profile")

    @patch("cyl_manager.services.base.SystemManager.get_hardware_profile", return_value="HIGH")
    @patch("cyl_manager.services.base.SystemManager.get_uid_gid", return_value=("1000", "1000"))
    @patch("cyl_manager.services.base.SystemManager.get_timezone", return_value="UTC")
    @patch("cyl_manager.services.base.DockerManager")
    def test_plex_config_high(self, mock_docker, mock_tz, mock_uid, mock_profile):
        svc = PlexService()
        config = svc.generate_compose()
        volumes = config["services"]["plex"]["volumes"]

        # Check if transcode IS /tmp:/transcode
        has_tmp_transcode = False
        for vol in volumes:
            if vol == "/tmp:/transcode":
                has_tmp_transcode = True

        self.assertTrue(has_tmp_transcode, "Plex SHOULD use RAM transcoding on HIGH profile")

if __name__ == '__main__':
    unittest.main()
