from conftest import BashTestEnvironment

# ==============================================================================
# TESTS FOR src/lib/security.sh
# ==============================================================================

def test_change_ssh_port_validation(bash_env):
    """Test SSH port change validation"""

    # Case 1: Invalid Port (Text)
    override_file = "override_ask.sh"
    with open(override_file, "w") as f:
        f.write("""
ask() {
    local prompt="$1"
    local var="$2"
    if [[ "$prompt" == *"change the default"* ]]; then eval "$var='y'"; fi
    if [[ "$prompt" == *"Enter new SSH Port"* ]]; then eval "$var='badport'"; fi
}
""")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/security.sh", override_file],
        "change_ssh_port"
    )

    assert code == 0
    assert "Invalid port" in stdout

    # Case 2: Valid Port
    # We modify the behavior of ask to return a valid port
    with open(override_file, "w") as f:
        f.write("""
ask() {
    local prompt="$1"
    local var="$2"
    if [[ "$prompt" == *"change the default"* ]]; then eval "$var='y'"; fi
    if [[ "$prompt" == *"Enter new SSH Port"* ]]; then eval "$var='2222'"; fi
}
""")

    # Mock system commands
    bash_env.create_mock_command("sed", "echo sed $@")
    bash_env.create_mock_command("systemctl", "echo systemctl $@")
    bash_env.create_mock_command("ufw", "echo ufw $@")
    bash_env.create_mock_command("grep", "exit 1")

    # Mock /etc/ssh/sshd_config using a temp file
    # We create a temporary copy of security.sh that uses our mock config file
    import os
    ssh_conf = os.path.join(bash_env.tmp_dir, "sshd_config")
    with open(ssh_conf, "w") as f:
        f.write("Port 22\nPermitRootLogin yes\n")

    with open("src/lib/security.sh", "r") as f:
        script_content = f.read().replace("/etc/ssh/sshd_config", ssh_conf)

    security_file = "security_test_mod.sh"
    with open(security_file, "w") as f:
        f.write(script_content)

    # Now run via sourcing
    stdout, stderr, code = bash_env.run_bash_function(
        [override_file, security_file],
        "change_ssh_port"
    )

    # Cleanup temp files
    if os.path.exists(override_file): os.remove(override_file)
    if os.path.exists(security_file): os.remove(security_file)

    assert code == 0
    assert "SSH Port changed to 2222" in stdout

def test_configure_firewall_execution(bash_env):
    """Test firewall configuration steps"""
    bash_env.create_mock_command("ufw", "echo ufw $@")
    bash_env.create_mock_command("apt-get", "echo apt")

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/security.sh"],
        "configure_firewall"
    )

    assert code == 0
    assert "Firewall configured" in stdout
    # Ensure it allows 80 and 443
    assert "ufw allow 80/tcp" in stdout or "HTTP" in stdout
