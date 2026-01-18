import os
import subprocess
import shutil
import platform
from pathlib import Path
from typing import Tuple, Union, List, Optional
import psutil
import distro

from cyl_manager.core.logging import logger
from cyl_manager.core.exceptions import SystemRequirementError

class SystemManager:
    """
    Manager for system-level operations and checks.
    Implements the 'Global Dynamic Hardware Detection' (GDHD) algorithm.
    """

    PROFILE_LOW: str = "LOW"
    PROFILE_HIGH: str = "HIGH"
    _hardware_profile: Optional[str] = None

    @staticmethod
    def check_root() -> None:
        """
        Enforces execution with elevated privileges (root).

        Raises:
            SystemRequirementError: If not running as root.
        """
        if os.geteuid() != 0:
            raise SystemRequirementError("Insufficient privileges. This operation requires root access.")

    @staticmethod
    def get_hardware_profile() -> str:
        """
        Determines the hardware profile using the GDHD heuristics.

        The system calculates a hardware profile based on strict thresholds:
        - CPU: <= 2 vCPUs (Critical for VPS context switching)
        - RAM: < 4 GB (Minimum baseline for full stack)
        - Swap: < 1 GB (OOM protection)

        If ANY of these conditions are met, the 'LOW' (Survival Mode) profile is enforced.

        Returns:
            str: 'LOW' if resources are constrained, else 'HIGH'.
        """
        if SystemManager._hardware_profile is not None:
            return SystemManager._hardware_profile

        try:
            mem = psutil.virtual_memory()
            swap = psutil.swap_memory()
            cpu_count = psutil.cpu_count() or 1

            # Criteria for LOW profile (The "Survival Mode" Thresholds)
            is_low_ram = mem.total < (4 * 1024**3)
            is_low_cpu = cpu_count <= 2
            is_low_swap = swap.total < (1 * 1024**3)

            if is_low_ram or is_low_cpu or is_low_swap:
                profile = SystemManager.PROFILE_LOW
                logger.info("Hardware Detection: [bold yellow]LOW SPEC DETECTED[/bold yellow]")
                if is_low_ram: logger.debug(f" - RAM: {mem.total/1024**3:.2f}GB (< 4GB)")
                if is_low_cpu: logger.debug(f" - CPU: {cpu_count} Cores (<= 2)")
                if is_low_swap: logger.debug(f" - SWAP: {swap.total/1024**3:.2f}GB (< 1GB)")
            else:
                profile = SystemManager.PROFILE_HIGH
                logger.info("Hardware Detection: [bold green]HIGH PERFORMANCE[/bold green]")
                logger.debug(f"Stats: {mem.total/1024**3:.2f}GB RAM, {cpu_count} Cores")

            SystemManager._hardware_profile = profile
            return profile

        except Exception as e:
            logger.warning(f"Failed to detect hardware profile, defaulting to LOW (Safe Mode): {e}")
            SystemManager._hardware_profile = SystemManager.PROFILE_LOW
            return SystemManager.PROFILE_LOW

    @staticmethod
    def get_concurrency_limit() -> int:
        """
        Returns the recommended concurrency limit for operations.
        LOW: Serial (1) to prevent IO/CPU saturation.
        HIGH: Parallel (4) for speed.
        """
        limit = 1 if SystemManager.get_hardware_profile() == SystemManager.PROFILE_LOW else 4
        logger.debug(f"Orchestrator Concurrency Limit: {limit} worker(s)")
        return limit

    @staticmethod
    def get_uid_gid() -> Tuple[str, str]:
        """
        Returns the UID and GID of the real user (even if running as root via sudo).

        Returns:
            Tuple[str, str]: (UID, GID)
        """
        try:
            # If running under sudo, use the original user's ID
            sudo_uid = os.getenv("SUDO_UID")
            sudo_gid = os.getenv("SUDO_GID")

            if sudo_uid and sudo_gid:
                return sudo_uid, sudo_gid

            # Fallback to current user
            return str(os.getuid()), str(os.getgid())
        except Exception as e:
            logger.warning(f"Failed to determine UID/GID, defaulting to 0/0: {e}")
            return "0", "0"

    @staticmethod
    def get_timezone() -> str:
        """
        Retrieves the system timezone.

        Returns:
            str: Timezone string (e.g., 'Europe/Paris') or 'UTC' on failure.
        """
        try:
            # Check /etc/timezone first
            tz_file = Path("/etc/timezone")
            if tz_file.exists():
                return tz_file.read_text(encoding="utf-8").strip()

            # Check symlink at /etc/localtime
            localtime = Path("/etc/localtime")
            if localtime.exists() and localtime.is_symlink():
                return str(localtime.resolve()).replace("/usr/share/zoneinfo/", "")

            return "UTC"
        except Exception as e:
            logger.warning(f"Could not determine timezone: {e}. Defaulting to UTC.")
            return "UTC"

    @staticmethod
    def check_os() -> None:
        """
        Checks if the operating system is supported.
        Currently supports Debian/Ubuntu based systems.
        """
        try:
            os_id = distro.id()
            if os_id not in ["debian", "ubuntu", "raspbian", "linuxmint"]:
                logger.warning(f"Untested OS detected: {os_id}. Proceeding with caution.")
        except Exception:
            logger.warning("Could not detect OS distribution.")

    @staticmethod
    def check_command(command: str) -> bool:
        """
        Checks if a system command is available.

        Args:
            command: The command to check (e.g., 'docker').

        Returns:
            bool: True if available, False otherwise.
        """
        return shutil.which(command) is not None

    @staticmethod
    def run_command(command: Union[List[str], str], check: bool = True, shell: bool = False) -> subprocess.CompletedProcess:
        """
        Runs a shell command safely.

        Args:
            command: The command to execute (list of args preferred).
            check: Whether to raise an exception on non-zero exit code.
            shell: Whether to run via shell (default: False for security).

        Returns:
            subprocess.CompletedProcess: The result of the command.
        """
        logger.debug(f"Executing: {command}")
        return subprocess.run(
            command,
            shell=shell,
            check=check,
            text=True,
            capture_output=True
        )
