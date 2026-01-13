import os
import psutil
import distro
import shutil
from pathlib import Path
from .exceptions import SystemRequirementError
from .logging import logger

class SystemManager:
    @staticmethod
    def check_root():
        if os.geteuid() != 0:
            raise SystemRequirementError("This script must be run as root.")

    @staticmethod
    def check_os():
        os_name = distro.id()
        if os_name not in ["debian", "ubuntu"]:
            raise SystemRequirementError(f"Unsupported OS: {os_name}. Only Debian and Ubuntu are supported.")
        logger.info(f"OS Detected: {distro.name(pretty=True)}")

    @staticmethod
    def get_hardware_profile() -> str:
        """Determines the hardware profile (LOW or HIGH)."""
        mem = psutil.virtual_memory()
        cpu_cores = psutil.cpu_count(logical=True)
        ram_gb = mem.total / (1024**3)

        profile = "HIGH"
        if ram_gb < 4 or cpu_cores <= 2:
            profile = "LOW"

        logger.debug(f"Hardware Profile: {profile} (RAM: {ram_gb:.2f}GB, Cores: {cpu_cores})")
        return profile

    @staticmethod
    def check_disk_space(min_gb=10):
        """Checks if there is enough disk space."""
        disk = psutil.disk_usage("/")
        free_gb = disk.free / (1024**3)

        logger.info(f"Disk Free: {free_gb:.2f} GB")
        if free_gb < 5:
            raise SystemRequirementError(f"CRITICAL: Less than 5GB free disk space ({free_gb:.2f}GB). Cannot proceed safely.")
        elif free_gb < min_gb:
            logger.warning(f"LOW DISK SPACE: Only {free_gb:.2f}GB free.")

    @staticmethod
    def get_uid_gid():
        """Returns the sudo UID and GID or defaults."""
        uid = os.environ.get("SUDO_UID", str(os.getuid()))
        try:
            import grp
            gid = os.environ.get("SUDO_GID", str(grp.getgrnam("docker").gr_gid))
        except (KeyError, ImportError):
            gid = "1000" # Fallback
        return uid, gid

    @staticmethod
    def get_timezone():
        if os.path.exists("/etc/timezone"):
            return Path("/etc/timezone").read_text().strip()
        return "UTC"
