import pytest
import subprocess
import os
import tempfile
import shutil
import re

# Helper class to manage the test environment
class BashTestEnvironment:
    def __init__(self):
        self.tmp_dir = tempfile.mkdtemp()
        self.bin_dir = os.path.join(self.tmp_dir, "bin")
        os.makedirs(self.bin_dir)
        self.env = os.environ.copy()
        self.env["PATH"] = f"{self.bin_dir}:{self.env['PATH']}"
        self.env["AUTH_FILE"] = os.path.join(self.tmp_dir, "auth_details")
        self.env["CONFIG_FILE"] = os.path.join(self.tmp_dir, "cyl_manager.conf")
        self.env["PROFILE_FILE"] = os.path.join(self.tmp_dir, "cyl_profile")
        self.env["LOG_DIR"] = os.path.join(self.tmp_dir, "logs")
        os.makedirs(self.env["LOG_DIR"])

        # Ensure we are in the root of the repo for sourcing
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
            source_cmd += f"source {f}\n"

        cmd = [
            "bash",
            "-c",
            f"cd {self.cwd}\n"
            f"source src/lib/core.sh\n" # Always source core first for logging
            f"{source_cmd}\n"
            f"{function_name} {' '.join(args)}"
        ]

        # Capture FD 3 logic by redirecting it to stdout within the shell
        cmd[-1] = f"exec 3>&1\n{cmd[-1]}"

        proc = subprocess.run(
            cmd,
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

# ------------------------------------------------------------------------------
# TESTS
# ------------------------------------------------------------------------------

# Happy Path: Config
def test_validate_password_ok(bash_env):
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "validate_password",
        ["securepassword123"]
    )
    assert code == 0

# Edge Case: Short Password
def test_validate_password_short(bash_env):
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "validate_password",
        ["short"]
    )
    assert code == 1
    assert "Password is too short" in stdout

# Mocking: detect_profile low memory
def test_detect_profile_low(bash_env):
    # Mock `free` command
    bash_env.create_mock_command("free", "echo 'Mem: 2000'")
    bash_env.create_mock_command("apt-get", "echo 'installed'") # Mock apt-get for zram

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/utils.sh"],
        "detect_profile"
    )
    assert code == 0
    assert "Profile selected: LOW RESOURCE" in stdout
    with open(bash_env.env["PROFILE_FILE"], "r") as f:
        assert f.read().strip() == "LOW"

# Mocking: detect_profile high memory
def test_detect_profile_high(bash_env):
    bash_env.create_mock_command("free", "echo 'Mem: 8000'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/utils.sh"],
        "detect_profile"
    )
    assert code == 0
    assert "Profile selected: HIGH PERFORMANCE" in stdout
    with open(bash_env.env["PROFILE_FILE"], "r") as f:
        assert f.read().strip() == "HIGH"

# Logic: calculate_swap_size
def test_calculate_swap_size_logic(bash_env):
    # Case 1: 1GB RAM, Plenty of Disk -> 2GB Swap
    bash_env.create_mock_command("free", "echo 'Mem: 1024'")
    # df output: Header line, then data line. 4th col is available blocks (using -BM means M suffix)
    bash_env.create_mock_command("df", "echo 'Filesystem 1M-blocks Used Available Use% Mounted on'; echo '/dev/sda1 50000 1000 49000M 2% /'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/install_system.sh"],
        "calculate_swap_size"
    )
    assert code == 0
    assert stdout.strip() == "2048"

# Security: check_root
def test_check_root_failure(bash_env):
    # We are not root in the test runner
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/core.sh"],
        "check_root"
    )
    assert code == 1
    assert "This script must be run as root" in stdout

# Docker: check_port_conflict
def test_check_port_conflict_free(bash_env):
    bash_env.create_mock_command("ss", "echo ''") # No ports used
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/docker.sh"],
        "check_port_conflict",
        ["8080", "TestService"]
    )
    assert code == 0

def test_check_port_conflict_taken(bash_env):
    bash_env.create_mock_command("ss", "echo 'tcp LISTEN 0 128 0.0.0.0:8080'") # Port used

    # We override `ask` by defining it before calling the function
    override = """
    ask() {
        echo "Mock Ask: $1"
        eval "$2='n'"
    }
    """

    cmd = [
        "bash",
        "-c",
        f"exec 3>&1\n"
        f"source src/lib/core.sh\n"
        f"{override}\n"
        f"source src/lib/docker.sh\n"
        f"check_port_conflict 8080 TestService"
    ]

    proc = subprocess.run(
        cmd,
        env=bash_env.env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    assert proc.returncode == 1 # Should fail fatal
    assert "Aborting installation" in proc.stdout

# Integration: deploy_docker_service structure
def test_deploy_docker_service_flow(bash_env):
    bash_env.create_mock_command("docker", "echo 'docker command mock'")
    bash_env.create_mock_command("ss", "echo ''") # Port free

    # We cannot easily patch the script to use a temp dir instead of /opt in the sandbox
    # So we simply mock `mkdir` to succeed silently
    # And we need to ensure `echo ... > /opt/...` doesn't fail.
    # The script uses: `echo "$docker_compose_content" > "/opt/$name/docker-compose.yml"`
    # This redirection happens in the shell. We can't mock the redirection destination easily.

    # STRATEGY: We read `src/lib/docker.sh`, replace `/opt` with `$OPT_DIR`, and then source it.

    with open("src/lib/docker.sh", "r") as f:
        docker_sh_content = f.read()

    # Replace /opt with our temp dir
    opt_dir = os.path.join(bash_env.tmp_dir, "opt")
    os.makedirs(opt_dir)
    modified_docker_sh = docker_sh_content.replace("/opt", opt_dir)

    # Override update_nginx
    override = """
    update_nginx() {
        echo "Mock Nginx Update for $1"
    }
    """

    cmd = [
        "bash",
        "-c",
        f"exec 3>&1\n"
        f"source src/lib/core.sh\n"
        f"{override}\n"
        f"{modified_docker_sh}\n" # Source the modified content directly
        f"deploy_docker_service myservice 'My Service' sub.dom 9090 'services: ...'"
    ]

    proc = subprocess.run(
        cmd,
        env=bash_env.env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    assert proc.returncode == 0
    assert "My Service Installed" in proc.stdout
    # Verify file creation in our temp opt dir
    assert os.path.exists(os.path.join(opt_dir, "myservice/docker-compose.yml"))

# System Update Happy Path
def test_system_update(bash_env):
    bash_env.create_mock_command("apt-get", "echo 'apt-get mock'")
    bash_env.create_mock_command("docker", "echo 'docker mock'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/utils.sh"],
        "system_update"
    )
    assert code == 0
    assert "Updating System..." in stdout
    assert "System Updated" in stdout

# Swap Calculation Edge Cases
def test_calculate_swap_size_edge_cases(bash_env):
    # Edge Case: Small RAM, Small Disk -> Cap Swap
    bash_env.create_mock_command("free", "echo 'Mem: 1024'")
    # 5GB Free Disk (5120M)
    bash_env.create_mock_command("df", "echo 'Filesystem 1M-blocks Used Available Use% Mounted on'; echo '/dev/sda1 10000 5000 5120M 50% /'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/install_system.sh"],
        "calculate_swap_size"
    )
    assert code == 0
    # Ideal swap for 1024MB RAM is 2048MB.
    # Max safe swap is 5120 / 2 = 2560.
    # Small disk logic: Free < 10GB (10240). If ideal > 1024, cap at 1024.
    # So expected is 1024.
    assert stdout.strip() == "1024"

    # Edge Case: Very Large RAM (32GB)
    bash_env.create_mock_command("free", "echo 'Mem: 32768'")
    bash_env.create_mock_command("df", "echo 'Filesystem 1M-blocks Used Available Use% Mounted on'; echo '/dev/sda1 100000 1000 90000M 2% /'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/install_system.sh"],
        "calculate_swap_size"
    )
    assert code == 0
    # > 8GB RAM -> Cap at 4096
    assert stdout.strip() == "4096"
