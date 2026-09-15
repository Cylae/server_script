use anyhow::bail;
use log::info;
use which::which;

pub fn check_installation() -> bool {
    which("docker").is_ok()
}

/// Verifies that Docker is installed on the system.
///
/// SECURITY (F01): previous implementation downloaded and executed an arbitrary
/// remote shell script via `curl https://get.docker.com | sh`, violating
/// REQ-SEC-001 (no shell execution with untrusted content) and introducing a
/// supply-chain risk (HTTPS alone is not integrity verification). Docker is
/// now required to be pre-installed by the system administrator.
pub fn install() -> anyhow::Result<()> {
    if check_installation() {
        info!("Docker is already installed.");
        return Ok(());
    }

    bail!(
        "Docker is not installed. Please install Docker manually before running server_manager. \
         See https://docs.docker.com/engine/install/ for installation instructions."
    );
}
