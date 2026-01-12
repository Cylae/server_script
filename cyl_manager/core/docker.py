import os
import shutil
import subprocess
import json
from .utils import msg, warn, fatal, ask, check_command
from .config import get

def init_docker():
    """Installs Docker if not present and configures it."""
    if not check_command("docker"):
        msg("Installing Docker...")
        # Add Docker official GPG key and repo
        subprocess.run(["mkdir", "-p", "/etc/apt/keyrings"], check=True)

        # Download GPG key
        subprocess.run(
            "curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg",
            shell=True, check=True
        )

        # Set up repository
        # Determining arch and codename
        # Using lsb_release if available or parsing /etc/os-release is safer but relying on shell commands for now
        # distro module is available so let's use it properly in a real scenario, but for now replicate shell logic
        # strictly speaking we should use python distro here.
        import distro
        codename = distro.codename()

        # This is hardcoded to debian in original script, but should probably be dynamic
        # Original: echo "deb [arch=$(dpkg --print-architecture) signed-by=...] https://download.docker.com/linux/debian $(lsb_release -cs) stable"

        # We will execute the shell commands to be safe and match original logic which seems to force debian repo even for ubuntu?
        # Actually standard practice is separate repos. But let's assume Debian as per memory "primary distribution".

        cmd = "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable\" | tee /etc/apt/sources.list.d/docker.list > /dev/null"
        subprocess.run(cmd, shell=True, check=True)

        subprocess.run(["apt-get", "update", "-q"], check=True)
        subprocess.run(["apt-get", "install", "-y", "docker-ce", "docker-ce-cli", "containerd.io", "docker-compose-plugin"], check=True)

    network_name = get("DOCKER_NET")
    # Check network
    try:
        subprocess.run(["docker", "network", "inspect", network_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    except subprocess.CalledProcessError:
        msg(f"Creating Docker network: {network_name}")
        subprocess.run(["docker", "network", "create", network_name], check=True)

    # Log Rotation
    if not os.path.exists("/etc/docker/daemon.json"):
        msg("Configuring Docker Log Rotation...")
        os.makedirs("/etc/docker", exist_ok=True)
        config = {
            "log-driver": "json-file",
            "log-opts": {
                "max-size": "10m",
                "max-file": "3"
            }
        }
        with open("/etc/docker/daemon.json", "w") as f:
            json.dump(config, f, indent=2)
        subprocess.run(["systemctl", "restart", "docker"], check=True)

def check_port_conflict(port, name):
    """Checks if a port is in use."""
    # Using ss command
    try:
        result = subprocess.run(f"ss -tuln | awk '{{print $5}}' | grep -E ':{port}$'", shell=True, stdout=subprocess.PIPE)
        if result.returncode == 0 and result.stdout:
            warn(f"Port {port} is currently in use.")
            confirm = ask(f"Is this expected (e.g., updating existing service)? (y/n)")
            if confirm.lower() != "y":
                fatal(f"Aborting installation of {name} due to port conflict on {port}.")
    except Exception as e:
        warn(f"Could not check port conflict: {e}")

def deploy_service(name, pretty_name, docker_compose_content, subdomain=None, port=None):
    """Deploys a Docker service."""
    from .proxy import update_nginx

    msg(f"Installing {pretty_name}...")

    if port:
        check_port_conflict(port, pretty_name)

    service_dir = f"/opt/{name}"
    os.makedirs(service_dir, exist_ok=True)

    compose_path = os.path.join(service_dir, "docker-compose.yml")
    with open(compose_path, "w") as f:
        f.write(docker_compose_content)

    # Run docker compose
    try:
        subprocess.run(["docker", "compose", "pull", "--quiet"], cwd=service_dir)
    except subprocess.CalledProcessError:
        warn(f"Failed to pull images for {name}, trying to build/run anyway...")

    subprocess.run(["docker", "compose", "up", "-d"], cwd=service_dir, check=True)

    if subdomain and port:
        update_nginx(subdomain, port, "proxy")

    msg(f"{pretty_name} Installed.")
    if subdomain:
        msg(f"Access at https://{subdomain}")

def remove_service(name, pretty_name, subdomain, port=None):
    """Removes a Docker service."""
    msg(f"Removing {pretty_name}...")

    confirm_delete = ask(f"Do you want to PERMANENTLY DELETE all data for {pretty_name}? (y/n)")

    service_dir = f"/opt/{name}"
    if os.path.exists(service_dir):
        compose_path = os.path.join(service_dir, "docker-compose.yml")
        if os.path.exists(compose_path):
            try:
                subprocess.run(["docker", "compose", "down"], cwd=service_dir)
            except Exception:
                pass

        if confirm_delete.lower() == "y":
            shutil.rmtree(service_dir)
            warn(f"Data directory {service_dir} deleted.")
        else:
            warn(f"Data directory {service_dir} PRESERVED.")

    # Remove Nginx config
    if os.path.exists(f"/etc/nginx/sites-enabled/{subdomain}"):
        os.remove(f"/etc/nginx/sites-enabled/{subdomain}")
    if os.path.exists(f"/etc/nginx/sites-available/{subdomain}"):
        os.remove(f"/etc/nginx/sites-available/{subdomain}")

    # Close UFW port if needed
    if port:
        subprocess.run(f"ufw delete allow {port}", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    msg(f"{pretty_name} Removed")

def is_installed(name):
    """Checks if a service is installed."""
    # Check running container
    res = subprocess.run(f"docker ps --format '{{{{.Names}}}}' | grep -q '^{name}'", shell=True)
    if res.returncode == 0:
        return True

    # Check directory
    if os.path.exists(f"/opt/{name}/docker-compose.yml"):
        return True

    return False
