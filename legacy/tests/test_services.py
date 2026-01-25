import unittest
from unittest.mock import MagicMock, patch
import os
import sys
from pathlib import Path

# Ensure src is in path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from cyl_manager.services.registry import ServiceRegistry
# Import all service modules to ensure they register
import cyl_manager.services.media
import cyl_manager.services.misc
import cyl_manager.services.infrastructure
import cyl_manager.services.gitea
import cyl_manager.services.portainer
import cyl_manager.services.more_services

class TestAllServices(unittest.TestCase):

    @patch("cyl_manager.services.base.SystemManager.get_hardware_profile", return_value="HIGH")
    @patch("cyl_manager.services.base.SystemManager.get_uid_gid", return_value=("1000", "1000"))
    @patch("cyl_manager.services.base.SystemManager.get_timezone", return_value="UTC")
    @patch("cyl_manager.services.base.DockerManager")
    @patch("cyl_manager.services.infrastructure.save_settings")
    def test_all_services_generate_compose_high(self, mock_save, mock_docker, mock_tz, mock_uid, mock_profile):
        """Test that all registered services can generate a valid compose config."""
        services = ServiceRegistry.get_all().values()
        self.assertTrue(len(services) > 0, "No services registered")

        for service_cls in services:
            with self.subTest(service=service_cls.name):
                svc = service_cls()
                config = svc.generate_compose()
                self.assertIsInstance(config, dict)
                self.assertIn("services", config)
                self.assertIn(service_cls.name, config["services"])
                # Check for networks if defined
                if "networks" in config:
                    self.assertIsInstance(config["networks"], dict)

    @patch("cyl_manager.services.base.SystemManager.get_hardware_profile", return_value="HIGH")
    @patch("cyl_manager.services.base.SystemManager.get_uid_gid", return_value=("1000", "1000"))
    @patch("cyl_manager.services.base.SystemManager.get_timezone", return_value="UTC")
    @patch("cyl_manager.services.base.DockerManager")
    @patch("cyl_manager.services.infrastructure.save_settings")
    def test_port_conflicts(self, mock_save, mock_docker, mock_tz, mock_uid, mock_profile):
        """Test for port conflicts between services."""
        services = ServiceRegistry.get_all().values()
        used_ports = {}

        for service_cls in services:
            svc = service_cls()
            config = svc.generate_compose()

            # Extract ports
            service_config = config["services"].get(service_cls.name, {})
            ports = service_config.get("ports", [])

            for port_mapping in ports:
                # Handle "8080:80" or "8080:80/udp"
                host_port = port_mapping.split(":")[0]

                # Check for conflicts, but ignore if the conflict is with itself (e.g. UDP/TCP on same port)
                if host_port in used_ports:
                    conflict_service = used_ports[host_port]
                    if conflict_service != service_cls.name:
                        self.fail(f"Port conflict detected: Port {host_port} is used by {service_cls.name} and {conflict_service}")

                used_ports[host_port] = service_cls.name

    @patch("cyl_manager.services.base.SystemManager.get_hardware_profile", return_value="LOW")
    @patch("cyl_manager.services.base.SystemManager.get_uid_gid", return_value=("1000", "1000"))
    @patch("cyl_manager.services.base.SystemManager.get_timezone", return_value="UTC")
    @patch("cyl_manager.services.base.DockerManager")
    @patch("cyl_manager.services.infrastructure.save_settings")
    def test_all_services_generate_compose_low(self, mock_save, mock_docker, mock_tz, mock_uid, mock_profile):
        """Test that all registered services can generate a valid compose config on LOW profile."""
        services = ServiceRegistry.get_all().values()
        self.assertTrue(len(services) > 0, "No services registered")

        for service_cls in services:
            with self.subTest(service=service_cls.name):
                svc = service_cls()
                config = svc.generate_compose()
                self.assertIsInstance(config, dict)
                self.assertIn("services", config)
                self.assertIn(service_cls.name, config["services"])

if __name__ == '__main__':
    unittest.main()
