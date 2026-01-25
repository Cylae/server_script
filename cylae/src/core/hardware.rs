use sysinfo::System;
use std::path::Path;
use log::{info, debug, warn};
use which::which;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwareProfile {
    Low,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuType {
    Nvidia,
    Intel,
    None,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub gpu_type: GpuType,
    pub device_path: Option<String>,
}

pub struct HardwareManager;

impl HardwareManager {
    pub fn get_hardware_profile() -> HardwareProfile {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_memory = sys.total_memory(); // in bytes
        let total_swap = sys.total_swap(); // in bytes
        let cpu_cores = sys.cpus().len();

        let _mem_gb = total_memory as f64 / 1024.0 / 1024.0 / 1024.0;
        let _swap_gb = total_swap as f64 / 1024.0 / 1024.0 / 1024.0;

        Self::evaluate_profile(total_memory, total_swap, cpu_cores)
    }

    // Public for testing without mocking system calls
    pub fn evaluate_profile(total_memory_bytes: u64, total_swap_bytes: u64, cpu_cores: usize) -> HardwareProfile {
        let is_low_ram = total_memory_bytes < 4 * 1024 * 1024 * 1024;
        let is_low_swap = total_swap_bytes < 1 * 1024 * 1024 * 1024;
        let is_low_cpu = cpu_cores <= 2;

        if is_low_ram || is_low_cpu || is_low_swap {
            debug!("Hardware Profile: LOW (RAM: {} bytes, Swap: {} bytes, CPU: {})", total_memory_bytes, total_swap_bytes, cpu_cores);
            HardwareProfile::Low
        } else {
            debug!("Hardware Profile: HIGH (RAM: {} bytes, Swap: {} bytes, CPU: {})", total_memory_bytes, total_swap_bytes, cpu_cores);
            HardwareProfile::High
        }
    }

    pub fn detect_gpu() -> GpuInfo {
        // Check for Nvidia
        let has_nvidia_driver = which("nvidia-smi").is_ok();
        let has_nvidia_toolkit = which("nvidia-container-cli").is_ok() || which("nvidia-container-runtime").is_ok();

        if has_nvidia_driver && has_nvidia_toolkit {
            info!("GPU Detected: Nvidia");
            return GpuInfo {
                gpu_type: GpuType::Nvidia,
                device_path: None, // Nvidia runtime handles devices
            };
        } else if has_nvidia_driver {
            warn!("Nvidia GPU detected but container toolkit missing. GPU acceleration disabled.");
        }

        // Check for Intel
        let dri_path = Path::new("/dev/dri");
        if dri_path.exists() {
            // Check for renderD*
            if let Ok(entries) = std::fs::read_dir(dri_path) {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.starts_with("renderD") {
                            info!("GPU Detected: Intel ({})", name);
                            return GpuInfo {
                                gpu_type: GpuType::Intel,
                                device_path: Some("/dev/dri".to_string()),
                            };
                        }
                    }
                }
            }
        }

        GpuInfo {
            gpu_type: GpuType::None,
            device_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_profile_low_ram() {
        let profile = HardwareManager::evaluate_profile(3 * 1024 * 1024 * 1024, 2 * 1024 * 1024 * 1024, 4);
        assert_eq!(profile, HardwareProfile::Low);
    }

    #[test]
    fn test_evaluate_profile_low_cpu() {
        let profile = HardwareManager::evaluate_profile(8 * 1024 * 1024 * 1024, 2 * 1024 * 1024 * 1024, 2);
        assert_eq!(profile, HardwareProfile::Low);
    }

    #[test]
    fn test_evaluate_profile_low_swap() {
        let profile = HardwareManager::evaluate_profile(8 * 1024 * 1024 * 1024, 512 * 1024 * 1024, 4);
        assert_eq!(profile, HardwareProfile::Low);
    }

    #[test]
    fn test_evaluate_profile_high() {
        let profile = HardwareManager::evaluate_profile(8 * 1024 * 1024 * 1024, 2 * 1024 * 1024 * 1024, 4);
        assert_eq!(profile, HardwareProfile::High);
    }
}
