use sysinfo::{System, SystemExt, DiskExt};
use std::process::Command;
use std::path::Path;
use log::{info};

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
    pub user_id: u32,
    pub group_id: u32,
}

impl HardwareInfo {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu();
        sys.refresh_disks();

        let total_memory = sys.total_memory(); // Bytes
        let ram_gb = total_memory / 1024 / 1024 / 1024;

        let total_swap = sys.total_swap();
        let swap_gb = total_swap / 1024 / 1024 / 1024;

        let cpu_cores = sys.cpus().len();

        let mut disk_gb = 0;
        for disk in sys.disks() {
            disk_gb += disk.total_space() / 1024 / 1024 / 1024;
        }

        let profile = Self::evaluate_profile(ram_gb, cpu_cores, swap_gb);

        let has_nvidia = Self::check_nvidia();
        let has_intel_quicksync = Path::new("/dev/dri").exists();

        info!("Hardware Detected: RAM={}GB, Swap={}GB, Disk={}GB, Cores={}, Profile={:?}", ram_gb, swap_gb, disk_gb, cpu_cores, profile);
        if has_nvidia { info!("Nvidia GPU Detected"); }
        if has_intel_quicksync { info!("Intel QuickSync Detected"); }

        Self {
            profile,
            ram_gb,
            cpu_cores,
            has_nvidia,
            has_intel_quicksync,
            disk_gb,
            swap_gb,
            user_id: 1000,
            group_id: 1000,
        }
    }

    fn check_nvidia() -> bool {
        // Check for nvidia-smi AND (nvidia-container-cli OR nvidia-container-runtime)
        let has_smi = Command::new("which").arg("nvidia-smi").output().map(|o| o.status.success()).unwrap_or(false);
        let has_cli = Command::new("which").arg("nvidia-container-cli").output().map(|o| o.status.success()).unwrap_or(false);
        let has_runtime = Command::new("which").arg("nvidia-container-runtime").output().map(|o| o.status.success()).unwrap_or(false);

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
mod tests {
    use super::*;

    #[test]
    fn test_hardware_profile_evaluation() {
        assert_eq!(HardwareInfo::evaluate_profile(2, 4, 2), HardwareProfile::Low); // Low RAM
        assert_eq!(HardwareInfo::evaluate_profile(8, 1, 2), HardwareProfile::Low); // Low Cores
        assert_eq!(HardwareInfo::evaluate_profile(32, 8, 0), HardwareProfile::High); // High RAM ignores Swap
        assert_eq!(HardwareInfo::evaluate_profile(8, 4, 0), HardwareProfile::Standard); // 8GB RAM No Swap -> Standard
        assert_eq!(HardwareInfo::evaluate_profile(6, 4, 0), HardwareProfile::Low); // 6GB RAM No Swap -> Low
        assert_eq!(HardwareInfo::evaluate_profile(6, 4, 2), HardwareProfile::Standard); // 6GB RAM + Swap -> Standard
    }
}
