import pytest
import subprocess
import os
import tempfile
import shutil
import re

# ==============================================================================
# TEST HELPER CLASS
# ==============================================================================

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

        # Create mocks for common commands to prevent accidental system calls
        self.create_mock_command("sudo", "$@") # Pass through
        self.create_mock_command("systemctl", "echo 'systemctl mock $@'")

        self.cwd = os.getcwd()

    def teardown(self):
        shutil.rmtree(self.tmp_dir)

    def create_mock_command(self, name, script, return_code=0):
        path = os.path.join(self.bin_dir, name)
        with open(path, "w") as f:
            f.write(f"#!/bin/bash\n{script}\nexit {return_code}")
        os.chmod(path, 0o755)

    def write_file(self, path, content):
        full_path = os.path.join(self.tmp_dir, path)
        os.makedirs(os.path.dirname(full_path), exist_ok=True)
        with open(full_path, "w") as f:
            f.write(content)
        return full_path

    def run_bash_function(self, source_files, function_name, args=[], env_vars={}):
        """
        Sources the given files and runs a function with arguments.
        """
        source_cmd = ""
        for f in source_files:
            source_cmd += f"source {f}\n"

        # Escape args
        args_str = " ".join([f"'{a}'" for a in args])

        # Add extra env vars
        env_export = ""
        for k, v in env_vars.items():
            env_export += f"export {k}='{v}'\n"

        cmd = [
            "bash",
            "-c",
            f"cd {self.cwd}\n"
            f"{env_export}"
            f"source src/lib/core.sh\n"
            f"{source_cmd}\n"
            f"{function_name} {args_str}"
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


# ==============================================================================
# TEST SUITE
# ==============================================================================

# 1. Security: Password Validation
# Expectation: Should fail for weak passwords (no complexity)
def test_validate_password_complexity(bash_env):
    # Weak password (length ok, but no complexity)
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "validate_password",
        ["password123"]
    )
    # CURRENT CODE BUG: It only checks length, so this passes (0).
    # We want to Assert it FAILS (1) once we fix it.
    # For now, to demonstrate the bug, we can assert code == 0,
    # OR we write the test for the DESIRED behavior and let it fail.
    # I will write for DESIRED behavior.

    # Update: "password123" has digits and letters.
    # Let's try "passwordpassword" (no digits, no upper)
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "validate_password",
        ["passwordpassword"]
    )
    assert code == 1, "Should fail for password with no digits/uppercase"

def test_validate_password_strong(bash_env):
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "validate_password",
        ["StrongPass1!"]
    )
    assert code == 0

# 2. Database: Secure Execution
# Expectation: The password should NOT appear in the executed command line arguments
def test_ensure_db_security(bash_env):
    # Mock mysql to print its arguments
    bash_env.create_mock_command("mysql", "echo \"ARGS: $@\"")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/database.sh"],
        "ensure_db",
        ["testdb", "testuser", "SecretPass123"],
        env_vars={"DB_ROOT_PASS": "rootpass"}
    )

    assert code == 0
    # The password should NOT be in the arguments
    assert "SecretPass123" not in stdout, "Password exposed in mysql arguments"

# 3. Nginx: Dynamic Body Size
# Expectation: 10G for 'cloud', 512M for others
def test_update_nginx_cloud(bash_env):
    # Mock /etc/nginx directories in temp
    nginx_avail = os.path.join(bash_env.tmp_dir, "etc/nginx/sites-available")
    nginx_enabled = os.path.join(bash_env.tmp_dir, "etc/nginx/sites-enabled")
    os.makedirs(nginx_avail)
    os.makedirs(nginx_enabled)

    # Create dummy defaults to prevent cat errors if referenced? No, the script creates them.

    # We need to hack the script path in run_bash_function because the script hardcodes /etc/nginx
    # Since we can't easily change the script path in the source without refactoring,
    # we might need to rely on the refactoring to allow config, OR sed the script in memory.
    # But `run_bash_function` sources the file.
    #
    # Let's use `sed` to replace /etc/nginx with our temp path in a copied file.

    with open("src/lib/proxy.sh", "r") as f:
        content = f.read()

    patched_content = content.replace("/etc/nginx", os.path.join(bash_env.tmp_dir, "etc/nginx"))
    patched_file = bash_env.write_file("src_patched/proxy.sh", patched_content)

    # Test Cloud
    bash_env.run_bash_function(
        [patched_file], # Use the patched file
        "update_nginx",
        ["cloud.example.com", "8080", "proxy"]
    )

    conf_file = os.path.join(nginx_avail, "cloud.example.com")
    with open(conf_file, "r") as f:
        conf = f.read()

    assert "client_max_body_size 10G;" in conf

def test_update_nginx_standard(bash_env):
    # Setup paths again (or reuse)
    nginx_avail = os.path.join(bash_env.tmp_dir, "etc/nginx/sites-available")
    nginx_enabled = os.path.join(bash_env.tmp_dir, "etc/nginx/sites-enabled")
    os.makedirs(nginx_avail, exist_ok=True)
    os.makedirs(nginx_enabled, exist_ok=True)

    with open("src/lib/proxy.sh", "r") as f:
        content = f.read()
    patched_content = content.replace("/etc/nginx", os.path.join(bash_env.tmp_dir, "etc/nginx"))
    patched_file = bash_env.write_file("src_patched/proxy.sh", patched_content)

    # Test Standard Service
    bash_env.run_bash_function(
        [patched_file],
        "update_nginx",
        ["blog.example.com", "9090", "proxy"]
    )

    conf_file = os.path.join(nginx_avail, "blog.example.com")
    with open(conf_file, "r") as f:
        conf = f.read()

    # Should use default 512M (or whatever we set, but definitely NOT 10G unconditionally if we fixed it)
    # Current code sets 10G always. We assert 512M to confirm the fix later.
    assert "client_max_body_size 512M;" in conf, "Should be 512M for non-cloud services"


# 4. Docker: Cleanup Sites Available
def test_remove_docker_service_cleanup(bash_env):
    # Setup mocks
    nginx_avail = os.path.join(bash_env.tmp_dir, "etc/nginx/sites-available")
    nginx_enabled = os.path.join(bash_env.tmp_dir, "etc/nginx/sites-enabled")
    os.makedirs(nginx_avail, exist_ok=True)
    os.makedirs(nginx_enabled, exist_ok=True)

    # Create dummy config
    open(os.path.join(nginx_avail, "test.com"), 'w').close()
    open(os.path.join(nginx_enabled, "test.com"), 'w').close()

    # Patch docker.sh to use our temp nginx paths AND temp /opt
    with open("src/lib/docker.sh", "r") as f:
        content = f.read()
    content = content.replace("/etc/nginx", os.path.join(bash_env.tmp_dir, "etc/nginx"))
    content = content.replace("/opt", os.path.join(bash_env.tmp_dir, "opt"))

    patched_file = bash_env.write_file("src_patched/docker.sh", content)

    # Create dummy service dir
    os.makedirs(os.path.join(bash_env.tmp_dir, "opt/testservice"), exist_ok=True)

    # Mock ask to return 'y'
    bash_env.create_mock_command("docker", "echo docker mock")

    # Override ask
    override = """
    ask() {
        eval "$2='y'"
    }
    """

    cmd = [
        "bash",
        "-c",
        f"exec 3>&1\n"
        f"source src/lib/core.sh\n"
        f"{override}\n"
        f"source {patched_file}\n"
        f"remove_docker_service testservice 'Test Service' test.com"
    ]

    proc = subprocess.run(
        cmd,
        env=bash_env.env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    assert proc.returncode == 0
    assert not os.path.exists(os.path.join(nginx_enabled, "test.com"))
    assert not os.path.exists(os.path.join(nginx_avail, "test.com")), "sites-available config should be removed"

# 5. Swap Calculation Logic (Existing, but good to keep)
def test_calculate_swap_size(bash_env):
    bash_env.create_mock_command("free", "echo 'Mem: 4096'")
    bash_env.create_mock_command("df", "echo 'Filesystem 1M-blocks Used Available Use% Mounted on'; echo '/dev/sda1 50000 1000 49000M 2% /'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/install_system.sh"],
        "calculate_swap_size"
    )
    assert code == 0
    assert stdout.strip() == "4096"
