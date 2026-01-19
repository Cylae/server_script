import unittest
from unittest.mock import patch, MagicMock
import subprocess
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

    @patch('install.subprocess.run')
    @patch('install.run_cmd')
    @patch('install.shutil.which')
    def test_install_system_deps_success(self, mock_which, mock_run_cmd, mock_subprocess_run):
        mock_which.return_value = '/usr/bin/apt-get'
        install.install_system_deps()
        # Expect run_cmd for 'apt-get update'
        mock_run_cmd.assert_called_once()
        # Expect subprocess.run for 'apt-get install' with specific env
        mock_subprocess_run.assert_called_once()

    @patch('install.sys.exit')
    @patch('install.shutil.which')
    def test_install_system_deps_no_apt(self, mock_which, mock_exit):
        mock_which.return_value = None
        install.install_system_deps()
        mock_exit.assert_called_with(1)

    @patch('install.subprocess.run')
    @patch('install.run_cmd')
    @patch('install.shutil.which')
    def test_configure_firewall_uses_no_shell(self, mock_which, mock_run_cmd, mock_subprocess_run):
        mock_which.return_value = '/usr/sbin/ufw'
        # Mock status check to return inactive so it proceeds
        # NOTE: run_cmd is mocked, so it won't actually call subprocess.run.
        # However, install.py now calls run_cmd for the enable step too.

        # We need to set up mock_run_cmd to return "Status: inactive" for the first call
        # and then just succeed for subsequent calls.
        mock_run_cmd.side_effect = [
            MagicMock(stdout="Status: inactive"), # ufw status
            MagicMock(), # ufw default deny
            MagicMock(), # ufw default allow
            MagicMock(), # ufw allow ssh
            MagicMock(), # ufw allow 22/tcp
            MagicMock()  # ufw enable
        ]

        install.configure_firewall()

        # Verify calls to run_cmd
        # We expect the last call to be ["ufw", "--force", "enable"]

        args, kwargs = mock_run_cmd.call_args
        self.assertEqual(args[0], ["ufw", "--force", "enable"])
        self.assertTrue(kwargs.get('check', True))

        # Also ensure subprocess.run was NOT called directly with shell=True for the enable step
        # (Though run_cmd calls subprocess.run, run_cmd itself uses shell=False by default)

        # We can check that NO call to subprocess.run had shell=True and the "echo 'y'..." command
        for call in mock_subprocess_run.call_args_list:
             args, kwargs = call
             if args[0] == "echo 'y' | ufw enable" and kwargs.get('shell') is True:
                 self.fail("Found use of shell=True with pipe, expected run_cmd with list args")

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

    @patch('install.Path.cwd')
    @patch('install.Path')
    def test_create_symlink_logic(self, MockPath, MockCwd):
        # Create separate mocks for target and source
        mock_target = MagicMock()
        mock_source = MagicMock()

        # MockCwd should return a path that, when joined, creates the source path
        MockCwd.return_value = MagicMock()
        # When we do Path.cwd() / VENV_DIR / ... it returns mock_source eventually
        # But install.py uses Path(CLI_LINK_PATH) and Path.cwd() / ...

        # Let's verify the source path logic
        # source = Path.cwd() / VENV_DIR / "bin" / "cyl-manager"
        # We need source.exists() to be True

        # Mocking Path instantiation
        def path_side_effect(*args, **kwargs):
            if args and str(args[0]) == "/usr/local/bin/cyl-manager":
                return mock_target
            return MagicMock() # For other Path() calls if any

        MockPath.side_effect = path_side_effect

        # Handle the chain: Path.cwd() -> / .venv -> / bin -> / cyl-manager
        # This is getting complex to mock the chain.
        # Instead, let's look at how install.py does it:
        # source = Path.cwd() / VENV_DIR / "bin" / "cyl-manager"

        # We can mock the end result of the chain
        mock_cwd_path = MagicMock()
        MockCwd.return_value = mock_cwd_path

        mock_venv = MagicMock()
        mock_cwd_path.__truediv__.return_value = mock_venv

        mock_bin = MagicMock()
        mock_venv.__truediv__.return_value = mock_bin

        mock_executable = MagicMock()
        mock_bin.__truediv__.return_value = mock_executable

        # Make source executable exist
        mock_executable.exists.return_value = True

        # Target exists and is symlink
        mock_target.exists.return_value = True
        mock_target.is_symlink.return_value = True

        install.create_symlink()

        mock_target.unlink.assert_called_once()
        mock_target.symlink_to.assert_called_once_with(mock_executable)

if __name__ == '__main__':
    unittest.main()
