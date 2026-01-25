use sysinfo::{System, SystemExt};
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
}

impl HardwareInfo {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu();

        let total_memory = sys.total_memory(); // Bytes
        let ram_gb = total_memory / 1024 / 1024 / 1024;
        let cpu_cores = sys.cpus().len();

        let profile = Self::evaluate_profile(ram_gb, cpu_cores);

        let has_nvidia = Self::check_nvidia();
        let has_intel_quicksync = Path::new("/dev/dri").exists();

        info!("Hardware Detected: RAM={}GB, Cores={}, Profile={:?}", ram_gb, cpu_cores, profile);
        if has_nvidia { info!("Nvidia GPU Detected"); }
        if has_intel_quicksync { info!("Intel QuickSync Detected"); }

        Self {
            profile,
            ram_gb,
            cpu_cores,
            has_nvidia,
            has_intel_quicksync,
        }
    }

    fn check_nvidia() -> bool {
        // Simple check for nvidia-smi
        match Command::new("which").arg("nvidia-smi").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    // For testing logic without system calls
    pub fn evaluate_profile(ram_gb: u64, cpu_cores: usize) -> HardwareProfile {
         if ram_gb < 4 || cpu_cores <= 2 {
            HardwareProfile::Low
        } else if ram_gb > 16 {
            HardwareProfile::High
        } else {
            HardwareProfile::Standard
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_profile_evaluation() {
        assert_eq!(HardwareInfo::evaluate_profile(2, 4), HardwareProfile::Low); // Low RAM
        assert_eq!(HardwareInfo::evaluate_profile(8, 1), HardwareProfile::Low); // Low Cores
        assert_eq!(HardwareInfo::evaluate_profile(8, 4), HardwareProfile::Standard);
        assert_eq!(HardwareInfo::evaluate_profile(32, 8), HardwareProfile::High);
    }
}
