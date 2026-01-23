#!/usr/bin/env python3
"""
Cylae Server Manager - Bootstrap Installer
"""

import os
import sys
import subprocess
import shutil
# import platform # Unused
from pathlib import Path
from typing import List

# --- Constants ---
REQUIRED_PACKAGES = [
    "python3", "python3-venv", "python3-pip", "python3-dev",
    "git", "curl", "ufw", "build-essential", "libffi-dev", "libssl-dev"
]
DOCKER_INSTALL_URL = "https://get.docker.com"
VENV_DIR = ".venv"
CLI_LINK_PATH = "/usr/local/bin/cyl-manager"

# --- Utils ---
def print_header(msg: str) -> None:
    print(f"\n\033[1;36m=== {msg} ===\033[0m")

def print_info(msg: str) -> None:
    print(f"\033[34mℹ\033[0m {msg}")

def print_success(msg: str) -> None:
    print(f"\033[32m✔\033[0m {msg}")

def print_error(msg: str) -> None:
    print(f"\033[31m✖ Error:\033[0m {msg}", file=sys.stderr)

def print_warning(msg: str) -> None:
    print(f"\033[33m⚠ Warning:\033[0m {msg}")

def run_cmd(cmd: List[str], check: bool = True, shell: bool = False) -> "subprocess.CompletedProcess[str]":
    """Runs a subprocess command safely."""
    try:
        return subprocess.run(cmd, check=check, shell=shell, text=True, capture_output=True)
    except subprocess.CalledProcessError as e:
        error_msg = e.stderr if e.stderr else str(e)
        raise RuntimeError(f"Command failed: {' '.join(cmd)}\nOutput: {error_msg}") from e

# --- Steps ---

def check_root() -> None:
    """Ensures script is run as root."""
    if os.geteuid() != 0:
        print_error("This script must be run as root (sudo).")
        sys.exit(1)

def install_system_deps() -> None:
    """Installs required system packages."""
    print_header("Installing System Dependencies")

    # Check if apt is available (Debian/Ubuntu)
    if not shutil.which("apt-get"):
        print_error("This installer currently supports Debian/Ubuntu based systems only.")
        sys.exit(1)

    try:
        print_info("Updating package list...")
        run_cmd(["apt-get", "update", "-qq"])

        print_info(f"Installing packages: {', '.join(REQUIRED_PACKAGES)}")
        run_cmd(["apt-get", "install", "-y", "-qq"] + REQUIRED_PACKAGES)
        print_success("System dependencies installed.")
    except RuntimeError as e:
        print_error(str(e))
        sys.exit(1)

def configure_firewall() -> None:
    """Ensures basic firewall security."""
    print_header("Configuring Firewall")

    if not shutil.which("ufw"):
        print_warning("ufw not found. Skipping firewall configuration.")
        return

    try:
        # Only configure if inactive or first run to avoid locking out
        res = run_cmd(["ufw", "status"], check=False)
        if "Status: active" in res.stdout:
            print_info("Firewall is already active.")
            return

        print_info("Setting up basic firewall rules...")
        run_cmd(["ufw", "default", "deny", "incoming"])
        run_cmd(["ufw", "default", "allow", "outgoing"])
        run_cmd(["ufw", "allow", "ssh"])
        run_cmd(["ufw", "allow", "22/tcp"])

        # Enable without prompt
        print_info("Enabling firewall...")
        run_cmd(["ufw", "--force", "enable"])

        print_success("Firewall configured and enabled.")
    except Exception as e:
        print_error(f"Failed to configure firewall: {e}")

def check_and_install_docker() -> None:
    """Checks for Docker and installs if missing."""
    print_header("Checking Docker Installation")

    if shutil.which("docker"):
        print_success("Docker is already installed.")
        return

    print_info("Docker not found. Installing via official script...")
    try:
        run_cmd(["curl", "-fsSL", DOCKER_INSTALL_URL, "-o", "get-docker.sh"])
        run_cmd(["sh", "get-docker.sh"])

        if os.path.exists("get-docker.sh"):
            os.remove("get-docker.sh")

        print_success("Docker installed successfully.")
    except RuntimeError as e:
        print_error(f"Failed to install Docker: {e}")
        sys.exit(1)
    except Exception as e: # pylint: disable=broad-exception-caught
        print_error(f"Unexpected error installing Docker: {e}")
        sys.exit(1)

def setup_virtual_environment() -> None:
    """Creates and configures the Python virtual environment."""
    print_header("Setting up Virtual Environment")

    venv_path = Path(os.getcwd()) / VENV_DIR
    python_exec = sys.executable

    # 1. Create venv
    if not venv_path.exists():
        print_info(f"Creating virtual environment at {venv_path}...")
        try:
            run_cmd([python_exec, "-m", "venv", str(venv_path)])
        except RuntimeError as e:
             print_error(f"Failed to create venv: {e}")
             sys.exit(1)
    else:
        print_info("Virtual environment already exists.")

    # 2. Install Package
    pip_path = venv_path / "bin" / "pip"
    if not pip_path.exists():
        print_error("pip not found in virtual environment.")
        sys.exit(1)

    print_info("Installing/Updating application...")
    try:
        # Upgrade pip deps
        run_cmd([str(pip_path), "install", "--no-warn-script-location", "-U", "pip", "setuptools", "wheel"])
        # Install in editable mode
        run_cmd([str(pip_path), "install", "-e", "."])
        print_success("Application installed.")
    except RuntimeError as e:
        print_error(f"Failed to install application: {e}")
        sys.exit(1)

def create_symlink() -> None:
    """Creates the global symlink for CLI access."""
    print_header("Finalizing Installation")

    target = Path(CLI_LINK_PATH)
    source = Path(os.getcwd()) / VENV_DIR / "bin" / "cyl-manager"

    if target.exists() or target.is_symlink():
        try:
            target.unlink()
        except OSError as e:
            print_error(f"Failed to remove existing symlink: {e}")
            # Warning only, try to overwrite

    try:
        target.symlink_to(source)
        print_success(f"Global command '{target.name}' created at {target}")
    except OSError as e:
        print_error(f"Failed to create symlink: {e}")
        sys.exit(1)

def optimize_system() -> None:
    """Applies persistent system optimizations via sysctl."""
    print_header("Applying System Optimizations")

    sysctl_conf = Path("/etc/sysctl.d/99-cylae-optimization.conf")

    # Optimizations:
    # 1. fs.inotify.max_user_watches: Essential for Plex/Arr monitoring of large libraries.
    # 2. vm.swappiness: Reduce to 10 to prefer RAM over Swap (better performance).
    # 3. net.core.default_qdisc & net.ipv4.tcp_congestion_control: BBR for better throughput.

    config_content = (
        "# Cylae Media Server Optimizations\n"
        "fs.inotify.max_user_watches=524288\n"
        "vm.swappiness=10\n"
        "net.core.default_qdisc=fq\n"
        "net.ipv4.tcp_congestion_control=bbr\n"
    )

    try:
        print_info("Writing sysctl optimizations...")
        with sysctl_conf.open("w", encoding="utf-8") as f:
            f.write(config_content)

        print_info("Applying sysctl changes...")
        run_cmd(["sysctl", "--system"])
        print_success("System optimizations applied.")

    except Exception as e:
        print_error(f"Failed to apply system optimizations: {e}")

def cloud_provider_warning() -> None:
    """Displays a warning about Cloud Firewalls."""
    print("\n" + "="*60)
    print_warning("Cloud Provider Detected (or assumed)")
    print("If you are running this on Google Cloud (GCP), AWS, or Azure:")
    print("You MUST configure your VPC Firewall Rules to allow traffic")
    print("on the ports used by your services (e.g., 80, 443, 81, etc.).")
    print("The script has configured the local OS firewall (ufw),")
    print("but it cannot modify your cloud provider's network firewall.")
    print("="*60 + "\n")

def main() -> None:
    """Main entry point."""
    try:
        check_root()
        install_system_deps()
        configure_firewall()
        check_and_install_docker()
        optimize_system()
        setup_virtual_environment()
        create_symlink()
        cloud_provider_warning()

        print("\n" + "="*50)
        print_success("Installation Complete! 🚀")
        print_info("You can now run 'cyl-manager menu' to start.")
        print("="*50 + "\n")

    except KeyboardInterrupt:
        print("\n\nInstallation cancelled by user.")
        sys.exit(130)
    except Exception as e: # pylint: disable=broad-exception-caught
        print_error(f"Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
