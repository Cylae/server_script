
import unittest
from unittest.mock import patch, MagicMock
import sys
import os

# Add src to sys.path
sys.path.append(os.path.abspath("src"))

from cyl_manager.services.more_services import MailService
from cyl_manager.core.system import SystemManager

class TestMailServiceConfig(unittest.TestCase):
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
    def test_low_profile_config(self, mock_get_profile):
        mock_get_profile.return_value = SystemManager.PROFILE_LOW

        service = MailService()

        compose = service.generate_compose()
        env = compose['services']['mailserver']['environment']
        deploy = compose['services']['mailserver']['deploy']

        print(f"LOW Profile Env: {env}")
        print(f"LOW Profile Limits: {deploy['resources']['limits']}")

        self.assertEqual(env['ENABLE_CLAMAV'], '0', "ClamAV should be disabled on LOW profile")
        self.assertEqual(env['ENABLE_SPAMASSASSIN'], '0', "SpamAssassin should be disabled on LOW profile")

        # Check resource limits
        self.assertEqual(deploy['resources']['limits']['cpus'], '1.0', "CPU limit should be 1.0 on LOW profile")

    @patch('cyl_manager.core.system.SystemManager.get_hardware_profile')
    def test_high_profile_config(self, mock_get_profile):
        mock_get_profile.return_value = SystemManager.PROFILE_HIGH

        service = MailService()
        compose = service.generate_compose()
        env = compose['services']['mailserver']['environment']
        deploy = compose['services']['mailserver']['deploy']

        print(f"HIGH Profile Env: {env}")
        print(f"HIGH Profile Limits: {deploy['resources']['limits']}")

        self.assertEqual(env['ENABLE_CLAMAV'], '1', "ClamAV should be enabled on HIGH profile")
        self.assertEqual(env['ENABLE_SPAMASSASSIN'], '1', "SpamAssassin should be enabled on HIGH profile")

if __name__ == '__main__':
    unittest.main()
