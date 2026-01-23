use sysinfo::System;
use log::{info, debug, warn};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareProfile {
    Low,
    High,
}

impl HardwareProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            HardwareProfile::Low => "LOW",
            HardwareProfile::High => "HIGH",
        }
    }
}

pub struct HardwareManager;

static HARDWARE_PROFILE: OnceLock<HardwareProfile> = OnceLock::new();

impl HardwareManager {
    /// Determines the hardware profile using the GDHD heuristics.
    ///
    /// The system calculates a hardware profile based on strict thresholds:
    /// - CPU: <= 2 vCPUs (Critical for VPS context switching)
    /// - RAM: < 4 GB (Minimum baseline for full stack)
    /// - Swap: < 1 GB (OOM protection)
    ///
    /// If ANY of these conditions are met, the 'LOW' (Survival Mode) profile is enforced.
    pub fn get_hardware_profile() -> &'static HardwareProfile {
        HARDWARE_PROFILE.get_or_init(|| {
            match Self::detect_hardware_profile() {
                Ok(profile) => profile,
                Err(e) => {
                    warn!("Failed to detect hardware profile, defaulting to LOW (Safe Mode): {}", e);
                    HardwareProfile::Low
                }
            }
        })
    }

    fn detect_hardware_profile() -> anyhow::Result<HardwareProfile> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let mem_total_bytes = sys.total_memory();
        let swap_total_bytes = sys.total_swap();
        let cpu_count = sys.cpus().len();

        // Convert to GB for logging/comparison (approx)
        let mem_gb = mem_total_bytes as f64 / 1024.0_f64.powi(3);
        let swap_gb = swap_total_bytes as f64 / 1024.0_f64.powi(3);

        // Criteria for LOW profile (The "Survival Mode" Thresholds)
        let is_low_ram = mem_total_bytes < (4 * 1024u64.pow(3));
        let is_low_cpu = cpu_count <= 2;
        let is_low_swap = swap_total_bytes < (1 * 1024u64.pow(3));

        if is_low_ram || is_low_cpu || is_low_swap {
            info!("Hardware Detection: LOW SPEC DETECTED");
            if is_low_ram {
                debug!(" - RAM: {:.2}GB (< 4GB)", mem_gb);
            }
            if is_low_cpu {
                debug!(" - CPU: {} Cores (<= 2)", cpu_count);
            }
            if is_low_swap {
                debug!(" - SWAP: {:.2}GB (< 1GB)", swap_gb);
            }
            Ok(HardwareProfile::Low)
        } else {
            info!("Hardware Detection: HIGH PERFORMANCE");
            debug!("Stats: {:.2}GB RAM, {} Cores", mem_gb, cpu_count);
            Ok(HardwareProfile::High)
        }
    }

    /// Returns the recommended concurrency limit for operations.
    /// LOW: Serial (1) to prevent IO/CPU saturation.
    /// HIGH: Parallel (4) for speed.
    pub fn get_concurrency_limit() -> usize {
        let limit = match Self::get_hardware_profile() {
            HardwareProfile::Low => 1,
            HardwareProfile::High => 4,
        };
        debug!("Orchestrator Concurrency Limit: {} worker(s)", limit);
        limit
    }
}
