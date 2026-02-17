//! mDNS/DNS-SD service advertisement support for gateway discovery.
//!
//! This implementation uses platform tooling:
//! - Linux: `avahi-publish-service`
//! - macOS: `dns-sd -R`
//! - Other platforms: logs a warning and no-ops.

use crate::config::MdnsConfig;
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

fn normalized_mode(mode: &str) -> &str {
    match mode {
        "minimal" | "full" => mode,
        _ => "off",
    }
}

/// Run mDNS advertisement until `cancel` is triggered.
pub async fn run_mdns_service(
    cfg: &MdnsConfig,
    agent_name: &str,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    if !cfg.enabled || normalized_mode(&cfg.mode) == "off" {
        return Ok(());
    }

    let service_name = cfg
        .service_name
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(agent_name);
    let service_type = cfg.service_type.trim();
    if service_type.is_empty() {
        anyhow::bail!("mDNS service_type cannot be empty");
    }

    #[cfg(target_os = "linux")]
    {
        run_with_avahi(service_name, service_type, port, cancel).await
    }

    #[cfg(target_os = "macos")]
    {
        run_with_dns_sd(service_name, service_type, port, cancel).await
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        eprintln!(
            "[mdns] mDNS discovery is not supported on this platform in the current build"
        );
        let _ = (service_name, service_type, port);
        cancel.cancelled().await;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
async fn run_with_avahi(
    service_name: &str,
    service_type: &str,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let mut child = Command::new("avahi-publish-service")
        .arg(service_name)
        .arg(service_type)
        .arg(port.to_string())
        .arg(format!("version={}", env!("CARGO_PKG_VERSION")))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to start avahi-publish-service for mDNS advertisement")?;

    eprintln!(
        "[mdns] Advertising service '{}' as {} on port {} (avahi)",
        service_name, service_type, port
    );

    tokio::select! {
        _ = cancel.cancelled() => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            eprintln!("[mdns] Advertisement stopped");
            Ok(())
        }
        status = child.wait() => {
            let status = status.context("avahi-publish-service process wait failed")?;
            if status.success() {
                Ok(())
            } else {
                anyhow::bail!("avahi-publish-service exited with status {}", status);
            }
        }
    }
}

#[cfg(target_os = "macos")]
async fn run_with_dns_sd(
    service_name: &str,
    service_type: &str,
    port: u16,
    cancel: CancellationToken,
) -> Result<()> {
    let mut child = Command::new("dns-sd")
        .arg("-R")
        .arg(service_name)
        .arg(service_type)
        .arg("local")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to start dns-sd for mDNS advertisement")?;

    eprintln!(
        "[mdns] Advertising service '{}' as {} on port {} (dns-sd)",
        service_name, service_type, port
    );

    tokio::select! {
        _ = cancel.cancelled() => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            eprintln!("[mdns] Advertisement stopped");
            Ok(())
        }
        status = child.wait() => {
            let status = status.context("dns-sd process wait failed")?;
            if status.success() {
                Ok(())
            } else {
                anyhow::bail!("dns-sd exited with status {}", status);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_normalization() {
        assert_eq!(normalized_mode("minimal"), "minimal");
        assert_eq!(normalized_mode("full"), "full");
        assert_eq!(normalized_mode("off"), "off");
        assert_eq!(normalized_mode("unexpected"), "off");
    }
}
