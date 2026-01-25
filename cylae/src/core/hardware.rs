use sysinfo::{System, SystemExt};
use std::path::Path;
use log::{info, debug};
use which::which;

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareProfile {
    Low,
    High,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuType {
    Nvidia,
    Intel,
    None,
}

pub struct HardwareManager;

impl HardwareManager {
    pub fn get_hardware_profile() -> HardwareProfile {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_ram_bytes = sys.total_memory();
        let total_swap_bytes = sys.total_swap();
        let cpu_count = sys.cpus().len();

        let ram_gb = total_ram_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let swap_gb = total_swap_bytes as f64 / 1024.0 / 1024.0 / 1024.0;

        let is_low_ram = ram_gb < 4.0;
        let is_low_cpu = cpu_count <= 2;
        let is_low_swap = swap_gb < 1.0;

        if is_low_ram || is_low_cpu || is_low_swap {
            info!("Hardware Detection: LOW SPEC DETECTED");
            if is_low_ram { debug!(" - RAM: {:.2}GB (< 4GB)", ram_gb); }
            if is_low_cpu { debug!(" - CPU: {} Cores (<= 2)", cpu_count); }
            if is_low_swap { debug!(" - SWAP: {:.2}GB (< 1GB)", swap_gb); }
        } else {
            info!("Hardware Detection: HIGH PERFORMANCE");
            debug!("Stats: {:.2}GB RAM, {} Cores", ram_gb, cpu_count);
        }

        Self::evaluate_profile(ram_gb, cpu_count, swap_gb)
    }

    pub fn evaluate_profile(ram_gb: f64, cpu_count: usize, swap_gb: f64) -> HardwareProfile {
        let is_low_ram = ram_gb < 4.0;
        let is_low_cpu = cpu_count <= 2;
        let is_low_swap = swap_gb < 1.0;

        if is_low_ram || is_low_cpu || is_low_swap {
            HardwareProfile::Low
        } else {
            HardwareProfile::High
        }
    }

    pub fn detect_gpu() -> GpuType {
        // Check for Nvidia
        // Need nvidia-smi AND (nvidia-container-cli OR nvidia-container-runtime)
        let has_nvidia_driver = which("nvidia-smi").is_ok();
        let has_nvidia_toolkit = which("nvidia-container-cli").is_ok() || which("nvidia-container-runtime").is_ok();

        if has_nvidia_driver && has_nvidia_toolkit {
            info!("GPU Detected: NVIDIA");
            return GpuType::Nvidia;
        }

        // Check for Intel QSV
        if Path::new("/dev/dri").exists() {
             // Basic check for renderD* devices
             if let Ok(entries) = std::fs::read_dir("/dev/dri") {
                 for entry in entries.flatten() {
                     if let Ok(name) = entry.file_name().into_string() {
                         if name.starts_with("renderD") {
                             info!("GPU Detected: INTEL");
                             return GpuType::Intel;
                         }
                     }
                 }
             }
        }

        GpuType::None
    }
}
