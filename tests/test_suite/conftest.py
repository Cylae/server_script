import pytest
import os
import shutil
import tempfile
import subprocess
import re

# ==============================================================================
# BASE TEST FIXTURES
# ==============================================================================

class BashTestEnvironment:
    def __init__(self):
        self.tmp_dir = tempfile.mkdtemp()
        self.bin_dir = os.path.join(self.tmp_dir, "bin")
        os.makedirs(self.bin_dir)

        # Setup Environment Variables
        self.env = os.environ.copy()
        self.env["PATH"] = f"{self.bin_dir}:{self.env.get('PATH', '')}"
        self.env["AUTH_FILE"] = os.path.join(self.tmp_dir, "auth_details")
        self.env["CONFIG_FILE"] = os.path.join(self.tmp_dir, "cyl_manager.conf")
        self.env["PROFILE_FILE"] = os.path.join(self.tmp_dir, "cyl_profile")
        self.env["LOG_DIR"] = os.path.join(self.tmp_dir, "logs")
        self.env["LOG_FILE"] = os.path.join(self.env["LOG_DIR"], "server_manager.log")
        self.env["BACKUP_DIR"] = os.path.join(self.tmp_dir, "backups")

        os.makedirs(self.env["LOG_DIR"])

        self.cwd = os.getcwd()

    def teardown(self):
        shutil.rmtree(self.tmp_dir)

    def create_mock_command(self, name, script, return_code=0):
        path = os.path.join(self.bin_dir, name)
        with open(path, "w") as f:
            f.write(f"#!/bin/bash\n{script}\nexit {return_code}")
        os.chmod(path, 0o755)

    def run_bash_function(self, source_files, function_name, args=[]):
        """
        Sources the given files and runs a function with arguments.
        Returns (stdout, stderr, return_code)
        """
        source_cmd = ""
        for f in source_files:
            # Handle absolute vs relative paths
            if not f.startswith("/"):
                 f = os.path.join(self.cwd, f)
            source_cmd += f"source {f}\n"

        cmd_script = (
            f"cd {self.cwd}\n"
            f"source src/lib/core.sh\n"
            f"{source_cmd}\n"
            f"exec 3>&1\n" # Redirect FD 3 to stdout for capturing
            f"{function_name} {' '.join(args)}"
        )

        proc = subprocess.run(
            ["bash", "-c", cmd_script],
            env=self.env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        return proc.stdout, proc.stderr, proc.returncode

@pytest.fixture
def bash_env():
    env = BashTestEnvironment()
    yield env
    env.teardown()
