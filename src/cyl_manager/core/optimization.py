from typing import Dict
from pathlib import Path
from cyl_manager.core.system import SystemManager
from cyl_manager.core.logging import logger

class KernelOptimizer:
    """
    Applies kernel-level optimizations (sysctl) based on the GDHD profile.
    """

    SYSCTL_CONF_PATH = Path("/etc/sysctl.d/99-cylae-optimization.conf")

    @staticmethod
    def get_optimizations() -> Dict[str, str]:
        """
        Returns a dictionary of sysctl key-value pairs based on the hardware profile.
        """
        profile = SystemManager.get_hardware_profile()
        optimizations = {}

        # Universal Hardening & Optimization
        optimizations.update({
            # Network hardening
            "net.ipv4.conf.all.accept_redirects": "0",
            "net.ipv6.conf.all.accept_redirects": "0",
            "net.ipv4.conf.all.send_redirects": "0",
            # TCP Optimization
            "net.ipv4.tcp_fastopen": "3",
            "net.ipv4.tcp_window_scaling": "1",
            "net.ipv4.tcp_timestamps": "1",
        })

        if profile == SystemManager.PROFILE_HIGH:
            # High Performance: Throughput focus
            optimizations.update({
                "fs.file-max": "2097152",
                "net.core.somaxconn": "65535",
                "net.ipv4.tcp_max_syn_backlog": "8192",
                "net.core.netdev_max_backlog": "16384",
                "net.ipv4.tcp_slow_start_after_idle": "0",
                "net.core.default_qdisc": "fq",
                "net.ipv4.tcp_congestion_control": "bbr",
                "vm.swappiness": "10", # Prefer RAM over Swap
            })
        else:
            # Low Spec: Stability & Memory Conservation
            optimizations.update({
                "vm.swappiness": "20",  # Use swap moderately to prevent OOM
                "vm.vfs_cache_pressure": "50", # Keep inode/dentry cache longer
                "vm.overcommit_memory": "0", # Heuristic overcommit (don't force always-overcommit "1")
                "net.ipv4.tcp_mem": "65536 131072 262144", # Cap TCP memory usage
            })

        return optimizations

    @staticmethod
    def apply_optimizations() -> None:
        """
        Applies sysctl settings immediately and writes them to a config file for persistence.
        """
        logger.info(f"Applying Kernel Optimizations for profile: {SystemManager.get_hardware_profile()}")

        settings_dict = KernelOptimizer.get_optimizations()
        conf_content = [f"# Cylae Server Manager - Auto Generated for {SystemManager.get_hardware_profile()} Profile"]

        for key, value in settings_dict.items():
            # 1. Apply immediately
            try:
                SystemManager.run_command(["sysctl", "-w", f"{key}={value}"])
                logger.debug(f"Sysctl set: {key} = {value}")
            except Exception as e:
                logger.warning(f"Failed to set sysctl {key}: {e}")

            # 2. Prepare for persistence
            conf_content.append(f"{key} = {value}")

        # 3. Write to file
        try:
            KernelOptimizer.SYSCTL_CONF_PATH.parent.mkdir(parents=True, exist_ok=True)
            KernelOptimizer.SYSCTL_CONF_PATH.write_text("\n".join(conf_content) + "\n", encoding="utf-8")
            logger.info(f"Kernel optimizations persisted to {KernelOptimizer.SYSCTL_CONF_PATH}")
        except Exception as e:
            logger.error(f"Failed to write sysctl config file: {e}")
