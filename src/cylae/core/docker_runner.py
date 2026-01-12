import os
import logging
import shutil
import subprocess
from .shell import run_command

logger = logging.getLogger(__name__)

DOCKER_NET = "server-net"

def is_docker_installed():
    return shutil.which("docker") is not None

def install_docker():
    """Installs Docker if not present."""
    if is_docker_installed():
        logger.info("Docker is already installed.")
        return

    logger.info("Installing Docker...")

    # 1. Add GPG key
    run_command("mkdir -p /etc/apt/keyrings")
    # Using shell=True for the pipe
    cmd_key = "curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg"
    # We need to handle overwrite if exists
    if os.path.exists("/etc/apt/keyrings/docker.gpg"):
        os.remove("/etc/apt/keyrings/docker.gpg")

    try:
        run_command(cmd_key, shell=True, check=True)
    except subprocess.CalledProcessError as e:
        logger.error("Failed to download Docker GPG key.")
        raise e

    # 2. Add Repository
    # We need to determine arch and codename
    try:
        # Simple detection
        out = subprocess.check_output(["dpkg", "--print-architecture"], text=True).strip()
        arch = out
        out = subprocess.check_output(["lsb_release", "-cs"], text=True).strip()
        codename = out
    except Exception:
        # Fallback
        arch = "amd64"
        codename = "bullseye" # Safe default? Or maybe read /etc/os-release

    repo_line = f"deb [arch={arch} signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian {codename} stable"

    with open("/etc/apt/sources.list.d/docker.list", "w") as f:
        f.write(repo_line + "\n")

    # 3. Install
    run_command(["apt-get", "update", "-q"])
    run_command(["apt-get", "install", "-y", "docker-ce", "docker-ce-cli", "containerd.io", "docker-compose-plugin"])

    logger.info("Docker installed successfully.")

def ensure_network():
    """Ensures the docker network exists."""
    try:
        run_command(["docker", "network", "inspect", DOCKER_NET], check=True, capture_output=True)
    except subprocess.CalledProcessError:
        logger.info(f"Creating network {DOCKER_NET}")
        run_command(["docker", "network", "create", DOCKER_NET])

def deploy_compose(service_name, compose_content, path):
    """Deploys a docker-compose service."""
    os.makedirs(path, exist_ok=True)
    file_path = os.path.join(path, "docker-compose.yml")

    with open(file_path, "w") as f:
        f.write(compose_content)

    logger.info(f"Deploying {service_name}...")
    run_command(["docker", "compose", "pull"], cwd=path)
    run_command(["docker", "compose", "up", "-d"], cwd=path)

def remove_compose(service_name, path, delete_data=False):
    """Removes a service."""
    if os.path.exists(os.path.join(path, "docker-compose.yml")):
        logger.info(f"Stopping {service_name}...")
        try:
            run_command(["docker", "compose", "down"], cwd=path)
        except Exception as e:
            logger.warning(f"Failed to stop {service_name}: {e}")

    if delete_data:
        shutil.rmtree(path)
        logger.info(f"Removed data for {service_name}")
