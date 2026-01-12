from conftest import BashTestEnvironment

# ==============================================================================
# TESTS FOR src/lib/core.sh and src/lib/config.sh
# ==============================================================================

def test_log_functions(bash_env):
    """Test logging functions write to LOG_FILE and FD 3"""
    script = """
    msg "Test Message"
    success "Test Success"
    warn "Test Warning"
    """

    # Ensure log file exists so logic writes to it
    log_path = bash_env.env["LOG_FILE"]
    with open(log_path, "w") as f:
        pass

    stdout, stderr, code = bash_env.run_bash_function(
        [], # Core is always sourced
        "eval", # Execute the script block
        [f"'{script}'"]
    )

    assert code == 0
    assert "Test Message" in stdout
    assert "Test Success" in stdout
    assert "Test Warning" in stdout

    with open(log_path, "r") as f:
        log_content = f.read()
        assert "INFO: Test Message" in log_content
        assert "SUCCESS: Test Success" in log_content
        assert "WARN: Test Warning" in log_content

def test_validate_password_happy_path(bash_env):
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "validate_password",
        ["securepassword123"]
    )
    assert code == 0

def test_validate_password_edge_case_short(bash_env):
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "validate_password",
        ["short"]
    )
    assert code == 1
    assert "Password is too short" in stdout

def test_save_and_get_credential(bash_env):
    """Test saving and retrieving credentials idempotently"""

    # 1. Save
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "save_credential",
        ["TEST_KEY", "TEST_VALUE"]
    )
    assert code == 0

    # 2. Retrieve
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "get_auth_value",
        ["TEST_KEY"]
    )
    assert code == 0
    assert "TEST_VALUE" in stdout.strip()

    # 3. Overwrite
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "save_credential",
        ["TEST_KEY", "NEW_VALUE"]
    )

    # 4. Retrieve New
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/config.sh"],
        "get_auth_value",
        ["TEST_KEY"]
    )
    assert "NEW_VALUE" in stdout.strip()
    # Ensure no duplicates
    with open(bash_env.env["AUTH_FILE"], "r") as f:
        content = f.read()
        assert content.count("TEST_KEY=") == 1

def test_check_root_fail_non_root(bash_env):
    """Expect fatal error if not root (mocked by not running as root)"""
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/core.sh"],
        "check_root"
    )
    # fatal calls exit 1
    assert code == 1
    assert "This script must be run as root" in stdout
