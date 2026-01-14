import os
import sys
import subprocess
import shutil
import platform
from pathlib import Path
from typing import List

# Attempt to use rich if installed (unlikely on fresh bootstrap, but good practice)
try:
    from rich.console import Console
    from rich.panel import Panel
    console = Console()
    def print_info(msg: str): console.print(f"[bold cyan]INFO:[/bold cyan] {msg}")
    def print_success(msg: str): console.print(f"[bold green]SUCCESS:[/bold green] {msg}")
    def print_error(msg: str): console.print(f"[bold red]ERROR:[/bold red] {msg}")
    def print_header(msg: str): console.print(Panel(f"[bold magenta]{msg}[/bold magenta]", expand=False))
except ImportError:
    def print_info(msg: str): print(f"INFO: {msg}")
    def print_success(msg: str): print(f"SUCCESS: {msg}")
    def print_error(msg: str): print(f"ERROR: {msg}")
    def print_header(msg: str): print(f"\n=== {msg} ===\n")

def run_cmd(cmd: List[str], check: bool = True) -> subprocess.CompletedProcess:
    """Runs a command safely without shell=True."""
    print_info(f"Running: {' '.join(cmd)}")
    return subprocess.run(cmd, check=check, text=True, capture_output=False)

def check_root() -> None:
    """Ensures script runs as root."""
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

def main() -> None:
    """Main installation routine."""
    check_root()
    install_system_deps()
    install_docker()
    setup_venv()
    create_symlink()

    print_header("Installation Complete! 🚀")
    print_info("Run 'cyl-manager menu' to start managing your server.")

if __name__ == "__main__":
    main()
