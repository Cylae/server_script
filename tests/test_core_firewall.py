import unittest
from unittest.mock import patch, MagicMock
from cyl_manager.core.firewall import FirewallManager

class TestFirewallManager(unittest.TestCase):
    def setUp(self):
        FirewallManager._instance = None

    @patch("cyl_manager.core.firewall.shutil.which")
    def test_is_available_true(self, mock_which):
        mock_which.return_value = "/usr/sbin/ufw"
        fw = FirewallManager()
        self.assertTrue(fw.is_available)

    @patch("cyl_manager.core.firewall.shutil.which")
    def test_is_available_false(self, mock_which):
        mock_which.return_value = None
        fw = FirewallManager()
        self.assertFalse(fw.is_available)

    @patch("cyl_manager.core.firewall.SystemManager.run_command")
    @patch("cyl_manager.core.firewall.shutil.which")
    def test_allow_port(self, mock_which, mock_run):
        mock_which.return_value = "/usr/sbin/ufw"
        fw = FirewallManager()
        fw.allow_port("80/tcp", "HTTP")
        mock_run.assert_called_with(["ufw", "allow", "80/tcp", "comment", "Cylae: HTTP"])

    @patch("cyl_manager.core.firewall.SystemManager.run_command")
    @patch("cyl_manager.core.firewall.shutil.which")
    def test_enable(self, mock_which, mock_run):
        mock_which.return_value = "/usr/sbin/ufw"
        # Mock status returning inactive
        mock_status = MagicMock()
        mock_status.stdout = "Status: inactive"

        # Mock enable return
        mock_enable = MagicMock()

        mock_run.side_effect = [mock_status, mock_enable]

        fw = FirewallManager()
        fw.enable()

        # Check calls: 1. status, 2. enable
        self.assertEqual(mock_run.call_count, 2)
        mock_run.assert_any_call(["ufw", "--force", "enable"])

if __name__ == "__main__":
    unittest.main()
