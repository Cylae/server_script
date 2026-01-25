import unittest
from unittest.mock import patch, MagicMock
import sys
import os

# Add src to sys.path
sys.path.append(os.path.abspath("src"))

from cyl_manager.core.hardware import HardwareManager
from cyl_manager.services.media import PlexService

class TestGPULogic(unittest.TestCase):
    def setUp(self):
        # Reset cached GPU info before each test
        HardwareManager._gpu_info = None

        # Patch DockerManager and FirewallManager for Service init
        self.docker_patcher = patch('cyl_manager.services.base.DockerManager')
        self.mock_docker = self.docker_patcher.start()
        self.firewall_patcher = patch('cyl_manager.services.base.FirewallManager')
        self.mock_firewall = self.firewall_patcher.start()

    def tearDown(self):
        self.docker_patcher.stop()
        self.firewall_patcher.stop()
        HardwareManager._gpu_info = None

    @patch('cyl_manager.core.hardware.shutil.which')
    def test_detect_gpu_nvidia(self, mock_which):
        # Mock nvidia-smi and nvidia-container-cli being present
        def side_effect(cmd):
            if cmd in ['nvidia-smi', 'nvidia-container-cli']:
                return f"/usr/bin/{cmd}"
            return None
        mock_which.side_effect = side_effect

        info = HardwareManager.detect_gpu()
        self.assertEqual(info['type'], HardwareManager.GPU_NVIDIA)

    @patch('cyl_manager.core.hardware.Path')
    @patch('cyl_manager.core.hardware.shutil.which')
    def test_detect_gpu_intel(self, mock_which, mock_path):
        # Mock no nvidia
        mock_which.return_value = None

        # Mock /dev/dri exists and has renderD*
        mock_dev_dri = MagicMock()
        mock_dev_dri.exists.return_value = True
        mock_dev_dri.glob.return_value = [MagicMock()] # return a list with one item

        # When Path("/dev/dri") is called, return mock_dev_dri
        def path_side_effect(arg):
            if str(arg) == "/dev/dri":
                return mock_dev_dri
            return MagicMock()

        mock_path.side_effect = path_side_effect

        info = HardwareManager.detect_gpu()
        self.assertEqual(info['type'], HardwareManager.GPU_INTEL)
        self.assertEqual(info['device'], "/dev/dri")

    @patch('cyl_manager.core.hardware.Path')
    @patch('cyl_manager.core.hardware.shutil.which')
    def test_detect_gpu_none(self, mock_which, mock_path):
        mock_which.return_value = None

        # Mock /dev/dri does NOT exist
        mock_dev_dri = MagicMock()
        mock_dev_dri.exists.return_value = False

        def path_side_effect(arg):
            if str(arg) == "/dev/dri":
                return mock_dev_dri
            return MagicMock()

        mock_path.side_effect = path_side_effect

        info = HardwareManager.detect_gpu()
        self.assertEqual(info['type'], HardwareManager.GPU_NONE)

    @patch('cyl_manager.core.hardware.HardwareManager.detect_gpu')
    def test_plex_config_nvidia(self, mock_detect):
        mock_detect.return_value = {"type": HardwareManager.GPU_NVIDIA}

        service = PlexService()
        compose = service.generate_compose()
        svc_conf = compose['services']['plex']

        self.assertEqual(svc_conf.get('runtime'), 'nvidia')
        self.assertIn("NVIDIA_VISIBLE_DEVICES", svc_conf['environment'])
        self.assertNotIn('devices', svc_conf)

    @patch('cyl_manager.core.hardware.HardwareManager.detect_gpu')
    def test_plex_config_intel(self, mock_detect):
        mock_detect.return_value = {"type": HardwareManager.GPU_INTEL}

        service = PlexService()
        compose = service.generate_compose()
        svc_conf = compose['services']['plex']

        self.assertIn("/dev/dri:/dev/dri", svc_conf.get('devices', []))
        self.assertNotIn('runtime', svc_conf)

if __name__ == '__main__':
    unittest.main()
