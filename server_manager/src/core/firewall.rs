use anyhow::Result;
use log::{info, warn};
use which::which;

pub fn configure_with_backend(backend: &dyn crate::core::ops::FirewallBackend) -> Result<()> {
    backend.configure_defaults()?;
    backend.allow_port(22, "tcp")?;
    backend.allow_port(8099, "tcp")?;
    backend.allow_port(80, "tcp")?;
    backend.allow_port(443, "tcp")?;
    backend.allow_port(32400, "tcp")?;
    backend.allow_port(8096, "tcp")?;
    backend.allow_port(51820, "udp")?;
    Ok(())
}

pub fn configure() -> Result<()> {
    info!("Configuring Firewall (UFW)...");

    // Check if ufw exists
    if which("ufw").is_err() {
        warn!("UFW not found. Skipping firewall configuration.");
        return Ok(());
    }

    configure_with_backend(&crate::core::ops::RealFirewallBackend)
}
