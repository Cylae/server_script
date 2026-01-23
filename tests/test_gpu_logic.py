import unittest
from unittest.mock import patch, MagicMock
from cyl_manager.core.hardware import HardwareManager
from cyl_manager.services.media import PlexService

class TestGPULogic(unittest.TestCase):

    def setUp(self):
        # Patch BaseService init stuff to avoid side effects
        self.patchers = [
             patch('cyl_manager.services.base.DockerManager'),
             patch('cyl_manager.services.base.FirewallManager'),
             patch('cyl_manager.services.base.SystemManager')
        ]
        self.mocks = [p.start() for p in self.patchers]

        # Configure SystemManager mock (index 2)
        self.mock_system = self.mocks[2]
        self.mock_system.get_uid_gid.return_value = (1000, 1000)
        self.mock_system.get_timezone.return_value = "UTC"

    def tearDown(self):
        for p in self.patchers:
            p.stop()

    @patch('cyl_manager.core.hardware.Path')
    @patch('cyl_manager.core.hardware.shutil.which')
    def test_detect_gpu_none(self, mock_which, mock_path):
        """Test detection when no GPU is present."""
        # Mock Path("/dev/dri").exists() -> False
        # When Path() is called, it returns a Mock object
        # We configure that Mock object's exists() method
        mock_path_instance = MagicMock()
        mock_path_instance.exists.return_value = False
        mock_path.return_value = mock_path_instance

        # Mock nvidia-smi -> None
        mock_which.return_value = None

        info = HardwareManager.detect_gpu()
        self.assertFalse(info['intel'])
        self.assertFalse(info['nvidia'])

    @patch('cyl_manager.core.hardware.Path')
    @patch('cyl_manager.core.hardware.shutil.which')
    def test_detect_gpu_intel(self, mock_which, mock_path):
        """Test detection of Intel/AMD iGPU."""
        # We need Path("/dev/dri") to return a mock with exists()=True
        # And any other Path() to default to False (optional but good)

        def side_effect(arg):
            m = MagicMock()
            if arg == "/dev/dri":
                m.exists.return_value = True
            else:
                m.exists.return_value = False
            return m

        mock_path.side_effect = side_effect
        mock_which.return_value = None

        info = HardwareManager.detect_gpu()
        self.assertTrue(info['intel'])
        self.assertFalse(info['nvidia'])

    @patch('cyl_manager.core.hardware.Path')
    @patch('cyl_manager.core.hardware.shutil.which')
    def test_detect_gpu_nvidia(self, mock_which, mock_path):
        """Test detection of Nvidia GPU."""
        # Ensure Intel is false
        mock_path.return_value.exists.return_value = False

        # Ensure Nvidia is true
        def which_side_effect(arg):
            if arg == "nvidia-smi":
                return "/usr/bin/nvidia-smi"
            if arg == "nvidia-container-cli":
                return "/usr/bin/nvidia-container-cli"
            return None

        mock_which.side_effect = which_side_effect

        info = HardwareManager.detect_gpu()
        self.assertFalse(info['intel'])
        self.assertTrue(info['nvidia'])

    @patch('cyl_manager.core.hardware.Path')
    @patch('cyl_manager.core.hardware.shutil.which')
    def test_detect_gpu_nvidia_missing_toolkit(self, mock_which, mock_path):
        """Test detection of Nvidia GPU without toolkit."""
        mock_path.return_value.exists.return_value = False

        def which_side_effect(arg):
            if arg == "nvidia-smi":
                return "/usr/bin/nvidia-smi"
            return None # Toolkit missing

        mock_which.side_effect = which_side_effect

        info = HardwareManager.detect_gpu()
        self.assertFalse(info['intel'])
        self.assertFalse(info['nvidia'])

    @patch('cyl_manager.core.hardware.HardwareManager.detect_gpu')
    def test_plex_config_intel(self, mock_detect):
        """Test Plex compose generation with Intel GPU."""
        mock_detect.return_value = {'intel': True, 'nvidia': False}

        svc = PlexService()
        # Mock profile to be HIGH
        svc.profile = "HIGH"

        compose = svc.generate_compose()
        service_conf = compose['services']['plex']

        self.assertIn("devices", service_conf)
        self.assertIn("/dev/dri:/dev/dri", service_conf["devices"])
        self.assertNotIn("runtime", service_conf)

    @patch('cyl_manager.core.hardware.HardwareManager.detect_gpu')
    def test_plex_config_nvidia(self, mock_detect):
        """Test Plex compose generation with Nvidia GPU."""
        mock_detect.return_value = {'intel': False, 'nvidia': True}

        svc = PlexService()
        svc.profile = "HIGH"

        compose = svc.generate_compose()
        service_conf = compose['services']['plex']

        self.assertEqual(service_conf.get("runtime"), "nvidia")
        self.assertIn("NVIDIA_VISIBLE_DEVICES", service_conf["environment"])
        self.assertEqual(service_conf["environment"]["NVIDIA_VISIBLE_DEVICES"], "all")
