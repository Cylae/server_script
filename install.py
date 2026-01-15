import os
import sys
import subprocess
import shutil
import platform
from pathlib import Path
from typing import List

def check_docker():
    """Checks if Docker is installed. If not, installs it."""
    if shutil.which("docker") is None:
        print("Docker not found. Installing Docker...")
        try:
            # Install Docker using the convenience script
            subprocess.run("curl -fsSL https://get.docker.com -o get-docker.sh", shell=True, check=True)
            subprocess.run("sh get-docker.sh", shell=True, check=True)
            print("Docker installed successfully.")
        except subprocess.CalledProcessError:
            print("Failed to install Docker. Please install Docker manually.")
            sys.exit(1)
        finally:
            if os.path.exists("get-docker.sh"):
                os.remove("get-docker.sh")
    else:
        print("Docker is already installed.")

def main():
    print("Checking root...")
    if os.geteuid() != 0:
        print_error("This script must be run as root.")
        sys.exit(1)

def install_system_deps() -> None:
    """Installs basic system dependencies via apt."""
    print_header("Installing System Dependencies")
    deps = ["python3", "python3-pip", "python3-venv", "git", "curl"]
    try:
        run_cmd(["apt-get", "update", "-q"])
        run_cmd(["apt-get", "install", "-y"] + deps)
    except subprocess.CalledProcessError as e:
        print_error(f"Failed to install system dependencies: {e}")
        sys.exit(1)

def install_docker() -> None:
    """Checks for Docker and installs if missing."""
    print_header("Checking Docker Installation")
    if shutil.which("docker"):
        print_success("Docker is already installed.")
        return

    print_info("Docker not found. Installing via convenience script...")
    try:
        # Download script
        subprocess.run(["curl", "-fsSL", "https://get.docker.com", "-o", "get-docker.sh"], check=True)
        # Run script
        subprocess.run(["sh", "get-docker.sh"], check=True)
        # Cleanup
        if os.path.exists("get-docker.sh"):
            os.remove("get-docker.sh")
        print_success("Docker installed successfully.")
    except subprocess.CalledProcessError as e:
        print_error(f"Failed to install Docker: {e}")
        sys.exit(1)

def setup_venv() -> None:
    """Sets up the Python virtual environment."""
    print_header("Setting up Virtual Environment")
    venv_path = Path(".venv")

    if not venv_path.exists():
        run_cmd([sys.executable, "-m", "venv", ".venv"])

    pip_cmd = str(venv_path / "bin" / "pip")

    try:
        # Upgrade pip tools
        run_cmd([pip_cmd, "install", "-U", "pip", "setuptools", "wheel"])
        # Install project in editable mode
        run_cmd([pip_cmd, "install", "-e", "."])
        print_success("Package installed successfully.")
    except subprocess.CalledProcessError as e:
        print_error(f"Failed to install Python packages: {e}")
        sys.exit(1)

def create_symlink() -> None:
    """Creates the global symlink for the CLI."""
    print_header("Finalizing Installation")
    target = Path("/usr/local/bin/cyl-manager")
    source = Path.cwd() / ".venv" / "bin" / "cyl-manager"

    if target.exists() or target.is_symlink():
        target.unlink()

    try:
        target.symlink_to(source)
        print_success(f"Symlink created at {target}")
    except OSError as e:
        print_error(f"Failed to create symlink: {e}")

    print("Creating symlink...")
    link_path = "/usr/local/bin/cyl-manager"
    if os.path.exists(link_path) or os.path.islink(link_path):
        os.remove(link_path)
    os.symlink(os.path.abspath(".venv/bin/cyl-manager"), link_path)

    print_header("Installation Complete! 🚀")
    print_info("Run 'cyl-manager menu' to start managing your server.")

if __name__ == "__main__":
    main()
