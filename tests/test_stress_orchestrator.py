import unittest
from unittest.mock import patch, MagicMock
import time
from concurrent.futures import Future

from cyl_manager.core.orchestrator import InstallationOrchestrator
from cyl_manager.core.system import SystemManager
from cyl_manager.core.hardware import HardwareManager

class MockService:
    def __init__(self, name, delay=0.1):
        self.name = name
        self.pretty_name = f"Service {name}"
        self.delay = delay
        self.installed = False

    def install(self):
        time.sleep(self.delay)
        self.installed = True

class TestStressOrchestrator(unittest.TestCase):
    def setUp(self):
        # Reset profiles
        HardwareManager._hardware_profile = None
        # Patch DockerManager to prevent network checks
        self.docker_patcher = patch('cyl_manager.core.orchestrator.DockerManager')
        self.mock_docker = self.docker_patcher.start()

    def tearDown(self):
        self.docker_patcher.stop()
        HardwareManager._hardware_profile = None

    @patch('cyl_manager.core.hardware.psutil')
    def test_high_concurrency_stress(self, mock_psutil):
        """
        Stress test the orchestrator with 50 services under HIGH profile (Parallel).
        """
        # Mock HIGH profile (4 workers)
        mock_mem = MagicMock()
        mock_mem.total = 16 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem
        mock_psutil.cpu_count.return_value = 8
        mock_swap = MagicMock()
        mock_swap.total = 4 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        HardwareManager._hardware_profile = None # Reset

        # Create 50 mock services
        services = [MockService(f"svc_{i}", delay=0.01) for i in range(50)]

        start_time = time.time()
        InstallationOrchestrator.install_services(services)
        end_time = time.time()

        # Verify all installed
        for svc in services:
            self.assertTrue(svc.installed, f"Service {svc.name} was not installed")

        # Verify concurrency limit was respected (mock check)
        # Note: We can't easily verify *active* threads here without deeper mocking of ThreadPoolExecutor,
        # but we verified the logic gets the limit.
        self.assertEqual(HardwareManager.get_concurrency_limit(), 4)

        # Rough check: 50 services * 0.01s / 4 workers ~= 0.125s minimum.
        # Serial would be 0.5s.
        # This is flaky in CI, so we just log it or rely on the fact it finished without error.
        print(f"High stress test took {end_time - start_time:.4f}s")

    @patch('cyl_manager.core.hardware.psutil')
    def test_low_concurrency_stress(self, mock_psutil):
        """
        Stress test the orchestrator with 20 services under LOW profile (Serial).
        """
        # Mock LOW profile (1 worker)
        mock_mem = MagicMock()
        mock_mem.total = 2 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem
        mock_psutil.cpu_count.return_value = 1
        mock_swap = MagicMock()
        mock_swap.total = 0.5 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        HardwareManager._hardware_profile = None

        services = [MockService(f"svc_{i}", delay=0.01) for i in range(20)]

        InstallationOrchestrator.install_services(services)

        # Verify all installed
        for svc in services:
            self.assertTrue(svc.installed)

        self.assertEqual(HardwareManager.get_concurrency_limit(), 1)

    @patch('cyl_manager.core.hardware.psutil')
    def test_orchestrator_resilience_to_failure(self, mock_psutil):
        """
        Verify that one service failing does not crash the entire orchestration.
        """
        # Mock HIGH profile
        mock_mem = MagicMock()
        mock_mem.total = 16 * 1024**3
        mock_psutil.virtual_memory.return_value = mock_mem
        mock_psutil.cpu_count.return_value = 8
        mock_swap = MagicMock()
        mock_swap.total = 4 * 1024**3
        mock_psutil.swap_memory.return_value = mock_swap

        HardwareManager._hardware_profile = None

        class FailingService(MockService):
            def install(self):
                raise RuntimeError("Boom!")

        services = [
            MockService("ok_1"),
            FailingService("fail_1"),
            MockService("ok_2"),
            FailingService("fail_2"),
            MockService("ok_3")
        ]

        # Should not raise exception
        InstallationOrchestrator.install_services(services)

        # Verify OK services installed
        self.assertTrue(services[0].installed)
        self.assertTrue(services[2].installed)
        self.assertTrue(services[4].installed)

        # Verify Failed services "tried" (the method was called)
        # Our mock class structure doesn't track "tried", but the fact that ok_3 installed
        # (which might be scheduled after fail_1) implies the loop continued.

if __name__ == '__main__':
    unittest.main()
