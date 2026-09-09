use log::{info, warn};
use nix::unistd::User;
use std::path::Path;
use std::sync::OnceLock;
use sysinfo::{DiskExt, System, SystemExt};
use which::which;

static HARDWARE_CACHE: OnceLock<HardwareInfo> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HardwareProfile {
    Low,      // < 4GB RAM, <= 2 cores
    Standard, // 4-16GB RAM
    High,     // > 16GB RAM
}

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub profile: HardwareProfile,
    pub ram_gb: u64,
    pub cpu_cores: usize,
    pub has_nvidia: bool,
    pub has_intel_quicksync: bool,
    pub disk_gb: u64,
    pub swap_gb: u64,
    pub user_id: String,
    pub group_id: String,
}

impl HardwareInfo {
    pub fn detect() -> Self {
        HARDWARE_CACHE.get_or_init(Self::detect_uncached).clone()
    }

    pub fn detect_uncached() -> Self {
        let (user_id, group_id) = Self::detect_user();
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu();
        sys.refresh_disks_list();
        sys.refresh_disks();

        let total_memory = sys.total_memory(); // Bytes
        let ram_gb = total_memory / 1024 / 1024 / 1024;

        let total_swap = sys.total_swap();
        let swap_gb = total_swap / 1024 / 1024 / 1024;

        let cpu_cores = sys.cpus().len();

        let disk_gb = sys
            .disks()
            .iter()
            .filter(|disk| {
                !matches!(
                    disk.file_system(),
                    b"overlay" | b"tmpfs" | b"devtmpfs" | b"squashfs" | b"sysfs" | b"proc"
                )
            })
            .map(|disk| disk.total_space() / 1024 / 1024 / 1024)
            .sum();

        let profile = Self::evaluate_profile(ram_gb, cpu_cores, swap_gb);

        let has_nvidia = Self::check_nvidia();
        let has_intel_quicksync = Path::new("/dev/dri").exists();

        info!(
            "Hardware Detected: RAM={}GB, Swap={}GB, Disk={}GB, Cores={}, Profile={:?}",
            ram_gb, swap_gb, disk_gb, cpu_cores, profile
        );
        if has_nvidia {
            info!("Nvidia GPU Detected");
        }
        if has_intel_quicksync {
            info!("Intel QuickSync Detected");
        }
        info!("User Context: UID={}, GID={}", user_id, group_id);

        Self {
            profile,
            ram_gb,
            cpu_cores,
            has_nvidia,
            has_intel_quicksync,
            disk_gb,
            swap_gb,
            user_id,
            group_id,
        }
    }

    fn detect_user() -> (String, String) {
        // Optimization: Try to use SUDO_UID and SUDO_GID directly to avoid subprocesses
        if let (Ok(uid), Ok(gid)) = (std::env::var("SUDO_UID"), std::env::var("SUDO_GID")) {
            return (uid, gid);
        }

        if let Ok(username) = std::env::var("SUDO_USER") {
            if let Ok(Some(user)) = User::from_name(&username) {
                return (user.uid.to_string(), user.gid.to_string());
            }
        }

        warn!("SUDO_USER not found or lookup failed. Defaulting to UID/GID 1000.");
        ("1000".to_string(), "1000".to_string())
    }

    fn check_nvidia() -> bool {
        // Check for nvidia-smi AND (nvidia-container-cli OR nvidia-container-runtime)
        let has_smi = which("nvidia-smi").is_ok();
        let has_cli = which("nvidia-container-cli").is_ok();
        let has_runtime = which("nvidia-container-runtime").is_ok();

        has_smi && (has_cli || has_runtime)
    }

    // For testing logic without system calls
    pub fn evaluate_profile(ram_gb: u64, cpu_cores: usize, swap_gb: u64) -> HardwareProfile {
        if ram_gb > 16 {
            HardwareProfile::High
        } else if ram_gb < 4 || cpu_cores <= 2 {
            HardwareProfile::Low
        } else {
            // Standard range (4-16GB RAM, >2 Cores)
            // If RAM is on the lower end (4-8GB) and no swap, downgrade to Low for safety
            // (This is a defensive measure to prevent OOM on machines with just enough RAM but no swap buffer)
            if ram_gb < 8 && swap_gb < 1 {
                HardwareProfile::Low
            } else {
                HardwareProfile::Standard
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_profile_evaluation() {
        assert_eq!(
            HardwareInfo::evaluate_profile(2, 4, 2),
            HardwareProfile::Low
        ); // Low RAM
        assert_eq!(
            HardwareInfo::evaluate_profile(8, 1, 2),
            HardwareProfile::Low
        ); // Low Cores
        assert_eq!(
            HardwareInfo::evaluate_profile(32, 8, 0),
            HardwareProfile::High
        ); // High RAM ignores Swap
        assert_eq!(
            HardwareInfo::evaluate_profile(8, 4, 0),
            HardwareProfile::Standard
        ); // 8GB RAM No Swap -> Standard
        assert_eq!(
            HardwareInfo::evaluate_profile(6, 4, 0),
            HardwareProfile::Low
        ); // 6GB RAM No Swap -> Low
        assert_eq!(
            HardwareInfo::evaluate_profile(6, 4, 2),
            HardwareProfile::Standard
        ); // 6GB RAM + Swap -> Standard
    }

    #[test]
    fn test_hardware_info_cached_and_uncached_detection() {
        let hw_uncached = HardwareInfo::detect_uncached();
        let hw_cached = HardwareInfo::detect();
        assert_eq!(hw_uncached.cpu_cores, hw_cached.cpu_cores);
        assert_eq!(hw_uncached.ram_gb, hw_cached.ram_gb);
        assert_eq!(hw_uncached.profile, hw_cached.profile);
    }
}
