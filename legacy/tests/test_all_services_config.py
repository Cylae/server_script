import unittest
from unittest.mock import patch, MagicMock
from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.core.system import SystemManager
from cyl_manager.core.hardware import HardwareManager

class TestAllServicesConfig(unittest.TestCase):
    def setUp(self):
        # Patch DockerManager and FirewallManager to prevent side effects
        self.docker_patcher = patch('cyl_manager.services.base.DockerManager')
        self.mock_docker = self.docker_patcher.start()
        self.firewall_patcher = patch('cyl_manager.services.base.FirewallManager')
        self.mock_firewall = self.firewall_patcher.start()

        # Patch settings to avoid file I/O or missing env vars
        self.settings_patcher = patch('cyl_manager.core.config.settings')
        self.mock_settings = self.settings_patcher.start()
        self.mock_settings.DATA_DIR = "/tmp/cylae_data"
        self.mock_settings.DOMAIN = "example.com"
        self.mock_settings.EMAIL = "admin@example.com"
        self.mock_settings.DOCKER_NET = "cylae_net"
        self.mock_settings.MYSQL_ROOT_PASSWORD = "rootpassword"
        self.mock_settings.MYSQL_USER_PASSWORD = "userpassword"

        # Also patch save_settings to avoid actual writes
        self.save_settings_patcher = patch('cyl_manager.services.infrastructure.save_settings')
        self.mock_save_settings = self.save_settings_patcher.start()

        # Ensure we can reset profiles
        HardwareManager._hardware_profile = None

    def tearDown(self):
        self.docker_patcher.stop()
        self.firewall_patcher.stop()
        self.settings_patcher.stop()
        self.save_settings_patcher.stop()
        HardwareManager._hardware_profile = None

    @patch('cyl_manager.core.hardware.psutil')
    def test_all_services_generate_compose_high_profile(self, mock_psutil):
        """
        Verify that EVERY registered service can generate a valid docker-compose
        dictionary under a HIGH performance profile.
        """
        # Mock HIGH profile hardware
        mock_mem = MagicMock()
        mock_mem.total = 16 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem
        mock_psutil.cpu_count.return_value = 8
        mock_swap = MagicMock()
        mock_swap.total = 4 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        # Reset cache
        HardwareManager._hardware_profile = None

        services = ServiceRegistry.get_all()
        for name, svc_cls in services.items():
            with self.subTest(service=name, profile="HIGH"):
                svc = svc_cls()
                try:
                    compose = svc.generate_compose()

                    # Basic Validation
                    self.assertIsInstance(compose, dict)
                    self.assertIn('services', compose)
                    # Networks might not be present if network_mode is 'host' (e.g. Plex)
                    if 'services' in compose and list(compose['services'].values())[0].get('network_mode') != 'host':
                        self.assertIn('networks', compose)

                    # Service-specific validation
                    # Most services use their name as the key in 'services'
                    # But some might differ slightly or have multiple (like mailserver might imply others? No, usually 1-to-1 here)
                    # We check if *at least one* service is defined.
                    self.assertTrue(len(compose['services']) > 0)

                except Exception as e:
                    self.fail(f"Service {name} failed to generate compose config in HIGH profile: {e}")

    @patch('cyl_manager.core.hardware.psutil')
    def test_all_services_generate_compose_low_profile(self, mock_psutil):
        """
        Verify that EVERY registered service can generate a valid docker-compose
        dictionary under a LOW performance profile (Survival Mode).
        """
        # Mock LOW profile hardware (1GB RAM, 1 Core)
        mock_mem = MagicMock()
        mock_mem.total = 1 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem
        mock_psutil.cpu_count.return_value = 1
        mock_swap = MagicMock()
        mock_swap.total = 0.5 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        # Reset cache
        HardwareManager._hardware_profile = None

        services = ServiceRegistry.get_all()
        for name, svc_cls in services.items():
            with self.subTest(service=name, profile="LOW"):
                svc = svc_cls()
                try:
                    compose = svc.generate_compose()

                    # Basic Validation
                    self.assertIsInstance(compose, dict)
                    self.assertIn('services', compose)

                    # Check resource limits if present (BaseService usually adds them)
                    # Note: generate_compose() usually calls get_resource_limits() and merges it,
                    # or the service implementation does.
                    # We verify no crash and structure validity.

                except Exception as e:
                    self.fail(f"Service {name} failed to generate compose config in LOW profile: {e}")

if __name__ == '__main__':
    unittest.main()
