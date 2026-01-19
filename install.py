#!/usr/bin/env python3
"""
Cylae Server Manager - Bootstrap Installer

This script bootstraps the environment required to run the Cylae Server Manager.
It handles system dependency installation, firewall configuration, Docker setup,
and Python virtual environment creation.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path
from typing import List, NoReturn

# --- Constants ---
REQUIRED_PACKAGES: list[str] = [
    "python3", "python3-venv", "python3-pip", "python3-dev",
    "git", "curl", "ufw", "build-essential", "libffi-dev", "libssl-dev"
]
DOCKER_INSTALL_URL: str = "https://get.docker.com"
VENV_DIR: str = ".venv"
CLI_LINK_PATH: str = "/usr/local/bin/cyl-manager"


# --- Utils ---
def print_header(msg: str) -> None:
    """Prints a styled header."""
    print(f"\n\033[1;36m=== {msg} ===\033[0m")


def print_info(msg: str) -> None:
    """Prints an informational message."""
    print(f"\033[34mℹ\033[0m {msg}")


def print_success(msg: str) -> None:
    """Prints a success message."""
    print(f"\033[32m✔\033[0m {msg}")


def print_error(msg: str) -> None:
    """Prints an error message to stderr."""
    print(f"\033[31m✖ Error:\033[0m {msg}", file=sys.stderr)


def print_warning(msg: str) -> None:
    """Prints a warning message."""
    print(f"\033[33m⚠ Warning:\033[0m {msg}")


def fail_exit(msg: str, code: int = 1) -> NoReturn:
    """Prints an error message and exits the script."""
    print_error(msg)
    sys.exit(code)


def run_cmd(cmd: List[str], check: bool = True, shell: bool = False) -> subprocess.CompletedProcess:
    """
    Runs a subprocess command safely.

    Args:
        cmd: The command to run as a list of strings.
        check: Whether to raise an exception on non-zero exit code.
        shell: Whether to run via shell (avoid if possible).

    Returns:
        The completed process object.

    Raises:
        RuntimeError: If the command fails and check is True.
    """
    try:
        return subprocess.run(
            cmd,
            check=check,
            shell=shell,
            text=True,
            capture_output=True
        )
    except subprocess.CalledProcessError as e:
        error_msg = e.stderr.strip() if e.stderr else str(e)
        raise RuntimeError(f"Command failed: {' '.join(cmd)}\nOutput: {error_msg}") from e


# --- Steps ---

def check_root() -> None:
    """Ensures script is run as root."""
    if os.geteuid() != 0:
        fail_exit("This script must be run as root (sudo).")


def install_system_deps() -> None:
    """Installs required system packages via apt-get."""
    print_header("Installing System Dependencies")

    if not shutil.which("apt-get"):
        fail_exit("This installer currently supports Debian/Ubuntu based systems only.")

    try:
        print_info("Updating package list...")
        run_cmd(["apt-get", "update", "-qq"])

        print_info(f"Installing packages: {', '.join(REQUIRED_PACKAGES)}")
        # DEBIAN_FRONTEND=noninteractive prevents prompts
        env = os.environ.copy()
        env["DEBIAN_FRONTEND"] = "noninteractive"

        # We use Popen or run with environment variables
        try:
            subprocess.run(
                ["apt-get", "install", "-y", "-qq"] + REQUIRED_PACKAGES,
                check=True,
                env=env,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True
            )
        except subprocess.CalledProcessError as e:
             raise RuntimeError(f"Apt install failed: {e.stderr}")

        print_success("System dependencies installed.")
    except RuntimeError as e:
        fail_exit(str(e))


def configure_firewall() -> None:
    """Ensures basic firewall security using ufw."""
    print_header("Configuring Firewall")

    if not shutil.which("ufw"):
        print_warning("ufw not found. Skipping firewall configuration.")
        return

    try:
        # Check status without raising error
        res = run_cmd(["ufw", "status"], check=False)
        if "Status: active" in res.stdout:
            print_info("Firewall is already active.")
            return

        print_info("Setting up basic firewall rules...")
        # Reset to known state? No, safer to just append rules.
        run_cmd(["ufw", "default", "deny", "incoming"])
        run_cmd(["ufw", "default", "allow", "outgoing"])
        run_cmd(["ufw", "allow", "ssh"])
        # Redundant but safe
        run_cmd(["ufw", "allow", "22/tcp"])

        print_info("Enabling firewall...")
        run_cmd(["ufw", "--force", "enable"])

        print_success("Firewall configured and enabled.")
    except Exception as e:
        print_error(f"Failed to configure firewall: {e}")
        # We don't exit here, as it's not strictly fatal for installation logic


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
        print_success("Docker installed successfully.")
    except RuntimeError as e:
        fail_exit(f"Failed to install Docker: {e}")
    finally:
        if os.path.exists("get-docker.sh"):
            try:
                os.remove("get-docker.sh")
            except OSError:
                pass


def setup_virtual_environment() -> None:
    """Creates and configures the Python virtual environment."""
    print_header("Setting up Virtual Environment")

    cwd = Path.cwd()
    venv_path = cwd / VENV_DIR
    python_exec = sys.executable

    # 1. Create venv
    if not venv_path.exists():
        print_info(f"Creating virtual environment at {venv_path}...")
        try:
            run_cmd([python_exec, "-m", "venv", str(venv_path)])
        except RuntimeError as e:
             fail_exit(f"Failed to create venv: {e}")
    else:
        print_info("Virtual environment already exists.")

    # 2. Install Package
    pip_path = venv_path / "bin" / "pip"
    if not pip_path.exists():
         fail_exit("pip not found in virtual environment.")

    print_info("Installing/Updating application...")
    try:
        # Upgrade pip deps
        run_cmd([str(pip_path), "install", "--no-warn-script-location", "-U", "pip", "setuptools", "wheel"])
        # Install in editable mode
        run_cmd([str(pip_path), "install", "-e", "."])
        print_success("Application installed.")
    except RuntimeError as e:
        fail_exit(f"Failed to install application: {e}")


def create_symlink() -> None:
    """Creates the global symlink for CLI access."""
    print_header("Finalizing Installation")

    target = Path(CLI_LINK_PATH)
    source = Path.cwd() / VENV_DIR / "bin" / "cyl-manager"

    if not source.exists():
        fail_exit(f"Source executable not found: {source}")

    if target.exists() or target.is_symlink():
        try:
            target.unlink()
        except OSError as e:
             print_warning(f"Failed to remove existing symlink: {e}")

    try:
        target.symlink_to(source)
        print_success(f"Global command '{target.name}' created at {target}")
    except OSError as e:
        fail_exit(f"Failed to create symlink: {e}")


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
    except Exception as e:
        fail_exit(f"Unexpected error: {e}")


if __name__ == "__main__":
    main()
