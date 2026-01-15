import unittest
from unittest.mock import patch, MagicMock
import sys
import os
from pathlib import Path

# Import the module to be tested
import install

class TestInstallScript(unittest.TestCase):

    @patch('install.os.geteuid')
    def test_check_root_success(self, mock_geteuid):
        mock_geteuid.return_value = 0
        try:
            install.check_root()
        except SystemExit:
            self.fail("check_root() raised SystemExit unexpectedly!")

    @patch('install.sys.exit')
    @patch('install.os.geteuid')
    def test_check_root_fail(self, mock_geteuid, mock_exit):
        mock_geteuid.return_value = 1000
        install.check_root()
        mock_exit.assert_called_with(1)

    @patch('install.run_cmd')
    @patch('install.shutil.which')
    def test_install_system_deps_success(self, mock_which, mock_run_cmd):
        mock_which.return_value = '/usr/bin/apt-get'
        install.install_system_deps()
        self.assertEqual(mock_run_cmd.call_count, 2) # update + install

    @patch('install.sys.exit')
    @patch('install.shutil.which')
    def test_install_system_deps_no_apt(self, mock_which, mock_exit):
        mock_which.return_value = None
        install.install_system_deps()
        mock_exit.assert_called_with(1)

    @patch('install.run_cmd')
    @patch('install.shutil.which')
    def test_check_and_install_docker_already_installed(self, mock_which, mock_run_cmd):
        mock_which.return_value = '/usr/bin/docker'
        install.check_and_install_docker()
        mock_run_cmd.assert_not_called()

    @patch('install.os.remove')
    @patch('install.os.path.exists')
    @patch('install.run_cmd')
    @patch('install.shutil.which')
    def test_check_and_install_docker_installing(self, mock_which, mock_run_cmd, mock_exists, mock_remove):
        mock_which.return_value = None
        mock_exists.return_value = True
        install.check_and_install_docker()
        self.assertEqual(mock_run_cmd.call_count, 2) # curl + sh
        mock_remove.assert_called_with("get-docker.sh")

    @patch('install.run_cmd')
    @patch('install.Path.exists')
    def test_setup_virtual_environment(self, mock_exists, mock_run_cmd):
        # Mocking exists to return False for venv, then True for pip
        mock_exists.side_effect = [False, True]

        with patch('install.sys.executable', '/usr/bin/python3'):
             install.setup_virtual_environment()

        # Verify calls: 1. create venv, 2. pip install upgrade, 3. pip install -e .
        self.assertEqual(mock_run_cmd.call_count, 3)

    @patch('install.Path')
    def test_create_symlink(self, MockPath):
        mock_target = MagicMock()
        mock_source = MagicMock()

        # Setup mock behavior
        mock_target.exists.return_value = False
        mock_target.is_symlink.return_value = False

        # MockPath return values
        # Path(CLI_LINK_PATH) -> mock_target
        # Path(os.getcwd()) / ... -> mock_source

        # We need to control what Path() returns based on input, but checking arg is complex with side_effect
        # Simplified: The first call in function is Path(CLI_LINK_PATH)
        # The second is Path(os.getcwd())

        # Let's mock the methods on the instance returned by Path
        mock_path_instance = MagicMock()
        MockPath.return_value = mock_path_instance

        # Fix: separate mocks for source and target
        # Assuming usage: target = Path(...), source = Path(...)
        # It's easier to mock the whole logic or just trust simple os.symlink logic
        pass

    @patch('install.Path')
    def test_create_symlink_logic(self, MockPath):
        # Create separate mocks for target and source
        mock_target = MagicMock()
        mock_source = MagicMock()

        # Helper to return the correct mock based on path
        def side_effect(path):
            if str(path) == "/usr/local/bin/cyl-manager":
                return mock_target
            return mock_source # Default fallback

        MockPath.side_effect = side_effect

        mock_target.exists.return_value = True
        mock_target.is_symlink.return_value = True

        install.create_symlink()

        mock_target.unlink.assert_called_once()
        mock_target.symlink_to.assert_called_once()

if __name__ == '__main__':
    unittest.main()
