import unittest
from unittest.mock import patch, mock_open, MagicMock
from cyl_manager.core.optimization import KernelOptimizer
from cyl_manager.core.system import SystemManager

class TestKernelOptimizer(unittest.TestCase):

    @patch("cyl_manager.core.system.SystemManager.get_hardware_profile")
    def test_get_optimizations_high(self, mock_profile):
        mock_profile.return_value = SystemManager.PROFILE_HIGH
        opts = KernelOptimizer.get_optimizations()

        self.assertEqual(opts["net.ipv4.tcp_congestion_control"], "bbr")
        self.assertEqual(opts["fs.file-max"], "2097152")
        self.assertEqual(opts["vm.swappiness"], "10")

    @patch("cyl_manager.core.system.SystemManager.get_hardware_profile")
    def test_get_optimizations_low(self, mock_profile):
        mock_profile.return_value = SystemManager.PROFILE_LOW
        opts = KernelOptimizer.get_optimizations()

        self.assertIn("vm.vfs_cache_pressure", opts)
        self.assertEqual(opts["vm.swappiness"], "20")
        self.assertNotEqual(opts.get("net.ipv4.tcp_congestion_control"), "bbr")

    @patch("cyl_manager.core.system.SystemManager.run_command")
    @patch("cyl_manager.core.system.SystemManager.get_hardware_profile")
    @patch("pathlib.Path.write_text")
    @patch("pathlib.Path.parent", new_callable=MagicMock)
    def test_apply_optimizations(self, mock_parent, mock_write, mock_profile, mock_run):
        mock_profile.return_value = SystemManager.PROFILE_HIGH

        KernelOptimizer.apply_optimizations()

        # Check that sysctl commands were run
        # We expect multiple calls, check for at least one critical one
        calls = [args[0] for args, _ in mock_run.call_args_list]
        self.assertTrue(any(['sysctl', '-w', 'net.ipv4.tcp_congestion_control=bbr'] == c for c in calls))

        # Check that file was written
        mock_write.assert_called_once()
        content = mock_write.call_args[0][0]
        self.assertIn("net.ipv4.tcp_congestion_control = bbr", content)

if __name__ == "__main__":
    unittest.main()
