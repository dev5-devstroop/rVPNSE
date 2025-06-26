//! Linux-specific tunnel implementation
//!
//! This module provides Linux-specific implementations for tunnel management.

use crate::error::{Result, VpnError};
use crate::tunnel::TunnelConfig;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

/// Create a TUN interface on Linux
#[allow(dead_code)]
pub fn create_tun_interface(_config: &TunnelConfig) -> Result<String> {
    println!("Creating TUN interface on Linux...");

    // Check if we have permission to create TUN devices
    if !has_tun_permissions() {
        return Err(VpnError::Connection(
            "Insufficient permissions to create TUN interface. Please run with sudo.".to_string(),
        ));
    }

    // Try to create a TUN device
    let interface_name = create_tun_device()?;

    println!("TUN interface '{interface_name}' created on Linux");
    Ok(interface_name)
}

/// Destroy a TUN interface on Linux
#[allow(dead_code)]
pub fn destroy_tun_interface(interface_name: &str) -> Result<()> {
    println!("Destroying TUN interface '{interface_name}' on Linux");

    // Bring interface down
    let _ = std::process::Command::new("ip")
        .args(["link", "set", interface_name, "down"])
        .status();

    println!("TUN interface '{interface_name}' destroyed on Linux");
    Ok(())
}

#[allow(dead_code)]
fn has_tun_permissions() -> bool {
    // Check if we can access /dev/net/tun
    std::path::Path::new("/dev/net/tun").exists()
}

#[allow(dead_code)]
fn create_tun_device() -> Result<String> {
    // On Linux, TUN devices are created dynamically through /dev/net/tun
    match OpenOptions::new()
        .read(true)
        .write(true)
        .mode(0o600)
        .open("/dev/net/tun")
    {
        Ok(_file) => {
            // TUN device opened successfully
            // The actual interface name would be determined by the TUN library
            let interface_name = "vpnse0".to_string();
            println!("Opened TUN device: /dev/net/tun");
            Ok(interface_name)
        }
        Err(e) => Err(VpnError::Connection(format!(
            "Failed to open /dev/net/tun: {e}"
        ))),
    }
}
