import os
import logging
from .system import run_command, install_apt_packages

logger = logging.getLogger("DockerManager")

def ensure_installed():
    """
    Ensures Docker and Docker Compose are installed.
    """
    try:
        run_command("docker --version")
        run_command("docker compose version")
    except:
        logger.info("Installing Docker...")
        # Official Docker installation script is robust
        # But for this environment, let's stick to apt if possible or get.docker.com
        # Since we want to be optimal, the apt version in debian/ubuntu repos is usually fine,
        # but 'docker-ce' from upstream is better.

        # Check if curl exists
        install_apt_packages(['curl', 'gnupg', 'lsb-release', 'ca-certificates'])

        # Add Docker GPG key and repo
        # Note: This is simplified. In a real rewriting, we'd do the full keyring dance.
        # For robustness, we'll use the convenience script which handles OS detection perfectly.
        try:
            run_command("curl -fsSL https://get.docker.com -o get-docker.sh", shell=True)
            run_command("sh get-docker.sh", shell=True)
            run_command("rm get-docker.sh", shell=True)
        except Exception as e:
            logger.error(f"Failed to install docker via script: {e}")
            raise

def deploy_service(name, path, compose_content):
    """
    Deploys a docker service.
    """
    if path:
        service_dir = path
    else:
        service_dir = f"/opt/{name}"

    os.makedirs(service_dir, exist_ok=True)

    compose_path = os.path.join(service_dir, "docker-compose.yml")

    # Check if content changed to avoid unnecessary restarts?
    # Docker Compose handles that logic mostly.

    with open(compose_path, 'w') as f:
        f.write(compose_content)

    logger.info(f"Deploying {name}...")
    try:
        run_command(["docker", "compose", "up", "-d", "--remove-orphans"], cwd=service_dir)
        return True
    except Exception as e:
        logger.error(f"Failed to deploy {name}: {e}")
        return False

def remove_service(name):
    service_dir = f"/opt/{name}"
    if os.path.exists(service_dir):
        logger.info(f"Removing {name}...")
        try:
            # Check if compose file exists
            if os.path.exists(os.path.join(service_dir, "docker-compose.yml")):
                run_command(["docker", "compose", "down"], cwd=service_dir)
            import shutil
            # Don't delete data blindly? Memory says we need confirmation.
            # But "rewrite code" implies I control the UX. I will implement a safe delete in CLI.
            # Here in the core, we just provide the capability.
            # We won't delete the folder in remove_service, just stop it.
            # Cleanup should be a separate explicit action.
        except Exception as e:
            logger.error(f"Failed to remove {name}: {e}")

def is_running(container_name):
    try:
        res = run_command(f"docker ps -q -f name=^{container_name}$", shell=True)
        return bool(res.stdout.strip())
    except:
        return False

def get_network_gateway(network_name="server-net"):
    try:
        # Create network if not exists
        create_network(network_name)
        cmd = f"docker network inspect {network_name} -f '{{{{range .IPAM.Config}}}}{{{{.Gateway}}}}{{{{end}}}}'"
        res = run_command(cmd, shell=True)
        return res.stdout.strip()
    except:
        return None

def create_network(network_name="server-net"):
    try:
        run_command(f"docker network inspect {network_name}", shell=True, check=True)
    except:
        run_command(f"docker network create {network_name}", shell=True)
