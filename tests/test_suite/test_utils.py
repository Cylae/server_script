from conftest import BashTestEnvironment
import subprocess
import os

# ==============================================================================
# TESTS FOR src/lib/utils.sh
# ==============================================================================

def test_detect_profile_low(bash_env):
    """Test Low Profile detection logic"""
    bash_env.create_mock_command("free", "echo 'Mem: 2000'")
    bash_env.create_mock_command("apt-get", "echo 'apt-get install mock'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/utils.sh"],
        "detect_profile"
    )
    assert code == 0
    assert "Profile selected: LOW RESOURCE" in stdout

    with open(bash_env.env["PROFILE_FILE"], "r") as f:
        assert f.read().strip() == "LOW"

def test_detect_profile_high(bash_env):
    """Test High Profile detection logic"""
    bash_env.create_mock_command("free", "echo 'Mem: 8000'")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/utils.sh"],
        "detect_profile"
    )
    assert code == 0
    assert "Profile selected: HIGH PERFORMANCE" in stdout

    with open(bash_env.env["PROFILE_FILE"], "r") as f:
        assert f.read().strip() == "HIGH"

def test_manage_backup_security(bash_env):
    """Verify that mysqldump uses defaults-extra-file"""

    # Mock commands
    mock_dump_log = os.path.join(bash_env.tmp_dir, "dump.log")
    script = f"echo \"$@\" > {mock_dump_log}"
    bash_env.create_mock_command("mysqldump", script)
    bash_env.create_mock_command("pigz", "echo pigz")
    bash_env.create_mock_command("tar", "echo tar")
    bash_env.create_mock_command("find", "echo find")

    with open(bash_env.env["AUTH_FILE"], "w") as f:
        f.write("mysql_root_password=supersecret")

    # Override BACKUP_DIR to tmp dir using env var?
    # utils.sh: BACKUP_DIR="/var/backups/cyl_manager" is hardcoded in config.sh?
    # No, config.sh sets it: BACKUP_DIR="/var/backups/cyl_manager"
    # We need to override it.

    # Since config.sh is sourced, we can override it AFTER sourcing.
    # But run_bash_function sources files then runs function.
    # We can create a wrapper that overrides it.

    backup_dir = bash_env.env["BACKUP_DIR"]
    os.makedirs(backup_dir)

    cmd_script = (
        f"source src/lib/config.sh\n"
        f"source src/lib/utils.sh\n"
        f"BACKUP_DIR='{backup_dir}'\n"
        f"manage_backup"
    )

    stdout, stderr, code = bash_env.run_bash_function(
        [], "eval", [f"'{cmd_script}'"]
    )

    if code != 0:
        print("STDOUT:", stdout)
        print("STDERR:", stderr)

    assert code == 0
    assert "Backup Complete" in stdout

    with open(mock_dump_log, "r") as f:
        args = f.read()
        assert "--defaults-extra-file=" in args
        assert "password=supersecret" not in args

def test_restore_logic_cancellation(bash_env):
    """Test restore wizard logic - cancellation path"""

    backup_dir = bash_env.env["BACKUP_DIR"]
    if not os.path.exists(backup_dir):
         os.makedirs(backup_dir)
    with open(os.path.join(backup_dir, "files_20230101.tar.gz"), "w") as f:
        f.write("dummy")

    # Mock `ask`
    override = """
    ask() {
        if [[ "$1" == *"Select backup"* ]]; then
            eval "$2='1'"
        elif [[ "$1" == *"Are you sure"* ]]; then
            eval "$2='n'"
        else
            eval "$2=''"
        fi
    }
    """

    cmd_script = (
        f"cd {bash_env.cwd}\n"
        f"source src/lib/core.sh\n"
        f"source src/lib/config.sh\n"
        f"source src/lib/utils.sh\n"
        f"{override}\n"
        f"BACKUP_DIR='{backup_dir}'\n" # Override
        f"exec 3>&1\n"
        f"manage_restore"
    )

    proc = subprocess.run(
        ["bash", "-c", cmd_script],
        env=bash_env.env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    if proc.returncode != 0:
        print("STDOUT:", proc.stdout)
        print("STDERR:", proc.stderr)

    assert proc.returncode == 0
    assert "Restore Cancelled" in proc.stdout
