import os
import shutil
import platform
import logging
from .shell import run_command

logger = logging.getLogger(__name__)

def check_root():
    """Checks if the script is running as root."""
    if os.geteuid() != 0:
        raise PermissionError("This script must be run as root.")

def check_os():
    """Checks if the OS is Debian or Ubuntu."""
    try:
        with open("/etc/os-release", "r") as f:
            content = f.read()
            if "ID=debian" in content or "ID=ubuntu" in content:
                return True
    except FileNotFoundError:
        pass

    logger.warning("OS detection failed or OS not supported. Proceeding with caution.")
    return False

def check_disk_space(min_mb=512):
    """Checks if there is enough disk space."""
    total, used, free = shutil.disk_usage("/")
    free_mb = free // (1024 * 1024)
    if free_mb < min_mb:
        logger.warning(f"Low disk space: {free_mb}MB available. Recommended: {min_mb}MB.")
    else:
        logger.debug(f"Disk space OK: {free_mb}MB available.")

def install_packages(packages):
    """Installs system packages using apt."""
    if not packages:
        return

    logger.info(f"Installing packages: {', '.join(packages)}")
    env = os.environ.copy()
    env["DEBIAN_FRONTEND"] = "noninteractive"

    cmd = ["apt-get", "install", "-y"] + packages
    run_command(cmd, env=env)

def install_core_dependencies():
    """Installs minimal dependencies required for the script to run fully."""
    deps = [
        "curl",
        "wget",
        "gnupg",
        "lsb-release",
        "git",
        "nginx",
        "ufw",
        "certbot",
        "python3-certbot-nginx",
        "python3-venv" # Useful if we ever need it
    ]

    # Check what is missing to save time
    missing = []
    for dep in deps:
        if not shutil.which(dep):
             # Some packages don't map 1:1 to binary names (e.g. gnupg -> gpg)
             if dep == "gnupg" and shutil.which("gpg"):
                 continue
             if dep == "lsb-release" and shutil.which("lsb_release"):
                 continue
             missing.append(dep)

    if missing:
        logger.info(f"Installing missing dependencies: {missing}")
        run_command(["apt-get", "update", "-q"])
        install_packages(missing)
