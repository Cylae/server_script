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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_check_installation_does_not_panic() {
        let is_installed = check_installation();
        // install() result must be consistent with check_installation()
        let install_res = install();
        if is_installed {
            assert!(install_res.is_ok());
        } else {
            assert!(install_res.is_err());
            let err_msg = install_res.unwrap_err().to_string();
            assert!(err_msg.contains("Docker is not installed"));
        }
    }
}
