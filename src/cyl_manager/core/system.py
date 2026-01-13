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
    def get_detailed_specs() -> dict:
        """Returns a dictionary with detailed system specifications."""
        mem = psutil.virtual_memory()
        swap = psutil.swap_memory()
        disk = psutil.disk_usage("/")

        return {
            "os": distro.name(pretty=True),
            "cpu_cores": psutil.cpu_count(logical=True),
            "ram_total_gb": round(mem.total / (1024**3), 2),
            "ram_available_gb": round(mem.available / (1024**3), 2),
            "swap_total_gb": round(swap.total / (1024**3), 2),
            "disk_total_gb": round(disk.total / (1024**3), 2),
            "disk_free_gb": round(disk.free / (1024**3), 2),
        }

    @staticmethod
    def get_hardware_profile() -> str:
        """
        Determines the hardware profile (LOW or HIGH).

        Logic:
        - LOW: < 4GB RAM OR <= 2 CPU Cores.
        - HIGH: >= 4GB RAM AND > 2 CPU Cores.

        Swap is analyzed but currently does not downgrade the profile,
        however it is logged for decision making in services.
        """
        specs = SystemManager.get_detailed_specs()

        profile = "HIGH"
        if specs["ram_total_gb"] < 4 or specs["cpu_cores"] <= 2:
            profile = "LOW"

        logger.debug(f"Hardware Profile Analysis: {profile}")
        logger.debug(f"Stats: RAM={specs['ram_total_gb']}GB, Cores={specs['cpu_cores']}, Swap={specs['swap_total_gb']}GB")

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
        except (KeyError, ImportError, AttributeError):
             # Fallback if docker group doesn't exist or other error
            gid = "1000"
        return uid, gid

    @staticmethod
    def get_timezone():
        if os.path.exists("/etc/timezone"):
            return Path("/etc/timezone").read_text().strip()
        return "UTC"
