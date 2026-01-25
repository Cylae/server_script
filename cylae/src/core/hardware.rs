use sysinfo::{System, SystemExt, DiskExt};
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use log::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareProfile {
    Low,
    High,
}

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub ram_gb: u64,
    pub cpu_cores: usize,
    pub swap_gb: u64,
    pub disk_gb: u64,
    pub has_nvidia_gpu: bool,
    pub has_intel_gpu: bool,
    pub profile: HardwareProfile,
}

pub struct HardwareManager;

impl HardwareManager {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_hardware(&self) -> Result<HardwareInfo> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let ram_gb = sys.total_memory() / 1024 / 1024 / 1024;
        let swap_gb = sys.total_swap() / 1024 / 1024 / 1024;
        let cpu_cores = sys.cpus().len();

        // Simple disk check (root)
        let disk_gb = sys.disks().iter()
            .find(|d| d.mount_point() == Path::new("/"))
            .map(|d| d.total_space() / 1024 / 1024 / 1024)
            .unwrap_or(0);

        let has_nvidia_gpu = self.check_nvidia();
        let has_intel_gpu = self.check_intel();

        let profile = self.evaluate_profile(ram_gb, cpu_cores, swap_gb);

        info!("Detected Hardware: RAM={}GB, Cores={}, Swap={}GB, Disk={}GB", ram_gb, cpu_cores, swap_gb, disk_gb);
        info!("GPU Status: Nvidia={}, Intel={}", has_nvidia_gpu, has_intel_gpu);
        info!("Profile Selected: {:?}", profile);

        Ok(HardwareInfo {
            ram_gb,
            cpu_cores,
            swap_gb,
            disk_gb,
            has_nvidia_gpu,
            has_intel_gpu,
            profile,
        })
    }

    fn check_nvidia(&self) -> bool {
        let has_smi = Command::new("nvidia-smi").output().is_ok();
        let has_toolkit = Command::new("nvidia-container-cli").output().is_ok()
                       || Command::new("nvidia-container-runtime").output().is_ok();

        if has_smi && !has_toolkit {
            warn!("Nvidia driver detected but Container Toolkit is missing. GPU will not be usable in Docker.");
        }

        has_smi && has_toolkit
    }

    fn check_intel(&self) -> bool {
        // Check for /dev/dri/renderD*
        if let Ok(entries) = std::fs::read_dir("/dev/dri") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("renderD") {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn evaluate_profile(&self, ram: u64, cores: usize, swap: u64) -> HardwareProfile {
        if ram < 4 || cores <= 2 || swap < 1 {
            HardwareProfile::Low
        } else {
            HardwareProfile::High
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_profile_low() {
        let hw = HardwareManager::new();
        // RAM < 4
        assert_eq!(hw.evaluate_profile(3, 4, 2), HardwareProfile::Low);
        // Cores <= 2
        assert_eq!(hw.evaluate_profile(8, 2, 2), HardwareProfile::Low);
        // Swap < 1
        assert_eq!(hw.evaluate_profile(8, 4, 0), HardwareProfile::Low);
    }

    #[test]
    fn test_evaluate_profile_high() {
        let hw = HardwareManager::new();
        // RAM >= 4, Cores > 2, Swap >= 1
        assert_eq!(hw.evaluate_profile(4, 3, 1), HardwareProfile::High);
        assert_eq!(hw.evaluate_profile(16, 8, 4), HardwareProfile::High);
    }
}
