from typing import Optional, Dict
import shutil
from pathlib import Path
import psutil
from cyl_manager.core.logging import logger

class HardwareManager:
    """
    Manager for hardware detection and profiling.
    Implements the 'Global Dynamic Hardware Detection' (GDHD) algorithm.
    """

    PROFILE_LOW: str = "LOW"
    PROFILE_HIGH: str = "HIGH"

    GPU_NVIDIA: str = "NVIDIA"
    GPU_INTEL: str = "INTEL"
    GPU_NONE: str = "NONE"

    _hardware_profile: Optional[str] = None
    _gpu_info: Optional[Dict[str, str]] = None

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
        if HardwareManager._hardware_profile is not None:
            return HardwareManager._hardware_profile

        try:
            mem = psutil.virtual_memory()
            swap = psutil.swap_memory()
            cpu_count = psutil.cpu_count() or 1

            # Detect GPU for logging
            gpu_info = HardwareManager.detect_gpu()
            gpu_type = gpu_info.get("type", HardwareManager.GPU_NONE)

            # Criteria for LOW profile (The "Survival Mode" Thresholds)
            is_low_ram = mem.total < (4 * 1024**3)
            is_low_cpu = cpu_count <= 2
            is_low_swap = swap.total < (1 * 1024**3)

            if is_low_ram or is_low_cpu or is_low_swap:
                profile = HardwareManager.PROFILE_LOW
                logger.info("Hardware Detection: [bold yellow]LOW SPEC DETECTED[/bold yellow]")
                if is_low_ram: logger.debug(" - RAM: %.2fGB (< 4GB)", mem.total/1024**3)
                if is_low_cpu: logger.debug(" - CPU: %d Cores (<= 2)", cpu_count)
                if is_low_swap: logger.debug(" - SWAP: %.2fGB (< 1GB)", swap.total/1024**3)
            else:
                profile = HardwareManager.PROFILE_HIGH
                logger.info("Hardware Detection: [bold green]HIGH PERFORMANCE[/bold green]")
                logger.debug("Stats: %.2fGB RAM, %d Cores", mem.total/1024**3, cpu_count)

            if gpu_type != HardwareManager.GPU_NONE:
                logger.info(f"GPU Detected: [bold cyan]{gpu_type}[/bold cyan]")

            HardwareManager._hardware_profile = profile
            return profile

        except Exception as e: # pylint: disable=broad-exception-caught
            logger.warning("Failed to detect hardware profile, defaulting to LOW (Safe Mode): %s", e)
            HardwareManager._hardware_profile = HardwareManager.PROFILE_LOW
            return HardwareManager.PROFILE_LOW

    @staticmethod
    def detect_gpu() -> Dict[str, str]:
        """
        Detects the presence of GPU hardware (Nvidia or Intel).

        Returns:
            Dict[str, str]: A dictionary containing 'type' and potentially 'device' paths.
                            Type is one of GPU_NVIDIA, GPU_INTEL, GPU_NONE.
        """
        if HardwareManager._gpu_info is not None:
            return HardwareManager._gpu_info

        info = {"type": HardwareManager.GPU_NONE}

        # 1. Check for Nvidia
        # We check for nvidia-smi (driver) AND nvidia-container-cli or nvidia-container-runtime (container toolkit)
        has_nvidia_driver = shutil.which("nvidia-smi") is not None
        has_nvidia_toolkit = shutil.which("nvidia-container-cli") is not None or \
                             shutil.which("nvidia-container-runtime") is not None

        if has_nvidia_driver and has_nvidia_toolkit:
             info["type"] = HardwareManager.GPU_NVIDIA
             HardwareManager._gpu_info = info
             return info

        # 2. Check for Intel QSV (/dev/dri)
        if Path("/dev/dri").exists():
             # Check for renderD* devices
             render_nodes = list(Path("/dev/dri").glob("renderD*"))
             if render_nodes:
                 info["type"] = HardwareManager.GPU_INTEL
                 info["device"] = "/dev/dri"
                 HardwareManager._gpu_info = info
                 return info

        HardwareManager._gpu_info = info
        return info

    @staticmethod
    def get_concurrency_limit() -> int:
        """
        Returns the recommended concurrency limit for operations.
        LOW: Serial (1) to prevent IO/CPU saturation.
        HIGH: Parallel (4) for speed.
        """
        limit = 1 if HardwareManager.get_hardware_profile() == HardwareManager.PROFILE_LOW else 4
        logger.debug("Orchestrator Concurrency Limit: %d worker(s)", limit)
        return limit

    @staticmethod
    def get_ram_gb() -> float:
        """Returns the total RAM in GB."""
        try:
            return psutil.virtual_memory().total / (1024**3)
        except Exception:
            return 0.0

    @staticmethod
    def detect_gpu() -> Dict[str, bool]:
        """
        Detects presence of Hardware Acceleration devices.
        Returns:
            Dict[str, bool]: {'intel': bool, 'nvidia': bool}
        """
        gpu_info = {'intel': False, 'nvidia': False}

        # Check for Intel/AMD (VAAPI/QSV)
        if Path("/dev/dri").exists():
            gpu_info['intel'] = True
            logger.debug("GPU Detection: Intel/AMD iGPU detected (/dev/dri).")

        # Check for Nvidia
        # We need both the driver (nvidia-smi) and the container toolkit (nvidia-container-cli or runtime)
        if shutil.which("nvidia-smi"):
            if shutil.which("nvidia-container-cli") or shutil.which("nvidia-container-runtime"):
                gpu_info['nvidia'] = True
                logger.debug("GPU Detection: Nvidia GPU detected and Container Toolkit present.")
            else:
                logger.warning("GPU Detection: Nvidia GPU detected but Container Toolkit missing. GPU acceleration will be disabled.")

        return gpu_info
