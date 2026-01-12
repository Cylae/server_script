from conftest import BashTestEnvironment
import subprocess
import os

# ==============================================================================
# TESTS FOR src/lib/docker.sh and src/lib/proxy.sh
# ==============================================================================

def test_deploy_docker_service_validation_success(bash_env):
    """Happy Path: deploy service with valid inputs"""

    # Mock Deps
    bash_env.create_mock_command("docker", "echo docker $@")
    bash_env.create_mock_command("ss", "echo") # No port conflict

    # Let's modify the path in memory for the test execution.
    # We replace /opt with the tmp_dir/opt which we create
    opt_dir = os.path.join(bash_env.tmp_dir, "opt")
    os.makedirs(opt_dir)

    with open("src/lib/docker.sh", "r") as f:
        docker_code = f.read().replace("/opt", opt_dir)

    # We also need to override update_nginx to avoid writing to /etc/nginx
    override_nginx = "override_nginx.sh"
    with open(override_nginx, "w") as f:
        f.write("""
update_nginx() {
    echo "Mock Nginx Update: $1 $2"
}
""")

    docker_test_file = "docker_test_mod.sh"
    with open(docker_test_file, "w") as f:
        f.write(docker_code)

    # Use double quotes for arguments.
    # We escape double quotes in the python f-string.
    cmd_script = (
        f"exec 3>&1\n"
        f"deploy_docker_service testservice \"Test Service\" sub.domain 8080 \"version: 3\""
    )

    # Run via sourcing
    # Note: run_bash_function wraps the command in single quotes!
    # f"'{cmd_script}'"
    # cmd_script contains double quotes. This is fine in bash: '... "..." ...'

    stdout, stderr, code = bash_env.run_bash_function(
        [override_nginx, docker_test_file],
        "eval",
        [f"'{cmd_script}'"]
    )

    # Cleanup
    if os.path.exists(override_nginx): os.remove(override_nginx)
    if os.path.exists(docker_test_file): os.remove(docker_test_file)

    if code != 0:
        print("STDOUT:", stdout)
        print("STDERR:", stderr)

    assert code == 0
    assert "Test Service Installed" in stdout

def test_deploy_docker_service_invalid_subdomain(bash_env):
    """Edge Case: Injection attempt in subdomain"""

    bash_env.create_mock_command("docker", "echo docker")

    # Use double quotes for arguments
    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/docker.sh"],
        "deploy_docker_service",
        ['"bad"', '"BadService"', '"sub;rm -rf /"', '"8080"', '"content"']
    )

    assert code == 1
    assert "Invalid subdomain" in stdout

def test_update_nginx_validation(bash_env):
    """Test Nginx input validation directly"""

    # Use single quotes inside double quotes?
    # run_bash_function splits args and joins with space.
    # args=['"sub$(reboot)"', "8080", "proxy"]
    # cmd = update_nginx "sub$(reboot)" 8080 proxy
    # bash executes this. "sub$(reboot)" expands $(reboot).

    # We want to pass the literal string containing $ to test regex failure.
    # In bash: 'sub$(reboot)' or "sub\$(reboot)" or similar.

    # In python: '"sub\$(reboot)"'

    stdout, stderr, code = bash_env.run_bash_function(
        ["src/lib/proxy.sh"],
        "update_nginx",
        ["'sub$(reboot)'", "8080", "proxy"]
    )

    assert code == 1
    assert "Invalid subdomain" in stdout

def test_check_port_conflict_found(bash_env):
    """Test port conflict logic"""
    bash_env.create_mock_command("ss", "echo 'tcp LISTEN 0 128 0.0.0.0:9090'")

    # Mock ask to say 'n' (abort)
    override_file = "override_ask_port.sh"
    with open(override_file, "w") as f:
        f.write("""
ask() {
    echo "Conflict Found"
    eval "$2='n'"
}
""")

    cmd = (
        f"source src/lib/docker.sh\n"
        f"exec 3>&1\n"
        f"check_port_conflict 9090 TestApp"
    )

    stdout, stderr, code = bash_env.run_bash_function(
        [override_file], "eval", [f"'{cmd}'"]
    )

    if os.path.exists(override_file): os.remove(override_file)

    assert code == 1
    assert "Aborting installation" in stdout
