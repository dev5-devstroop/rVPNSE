//! macOS-specific tunnel implementation
//!
//! This module provides macOS-specific implementations for tunnel management.

use crate::error::{Result, VpnError};
use crate::tunnel::TunnelConfig;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::process::Command;

/// Create a TUN interface on macOS
#[allow(dead_code)]
pub fn create_tun_interface(_config: &TunnelConfig) -> Result<String> {
    println!("Creating TUN interface on macOS...");

    // Check if we have permission to create TUN devices
    if !has_tun_permissions() {
        return Err(VpnError::Connection(
            "Insufficient permissions to create TUN interface. Please run with sudo or install proper entitlements.".to_string()
        ));
    }

    // Try to create a TUN device
    let interface_name = create_tun_device()?;

    println!("TUN interface '{interface_name}' created on macOS");
    Ok(interface_name)
}

/// Destroy a TUN interface on macOS
#[allow(dead_code)]
pub fn destroy_tun_interface(interface_name: &str) -> Result<()> {
    println!("Destroying TUN interface '{interface_name}' on macOS");

    // Bring interface down
    let _ = Command::new("ifconfig")
        .args([interface_name, "down"])
        .status();

    println!("TUN interface '{interface_name}' destroyed on macOS");
    Ok(())
}

#[allow(dead_code)]
fn has_tun_permissions() -> bool {
    // Check if we can access /dev/tun* devices
    // On macOS, TUN devices are usually /dev/tun0, /dev/tun1, etc.
    for i in 0..16 {
        let tun_path = format!("/dev/tun{i}");
        if OpenOptions::new()
            .read(true)
            .write(true)
            .mode(0o600)
            .open(&tun_path)
            .is_ok()
        {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn create_tun_device() -> Result<String> {
    // Try to find an available TUN device
    for i in 0..16 {
        let tun_path = format!("/dev/tun{i}");
        let interface_name = format!("tun{i}");

        match OpenOptions::new()
            .read(true)
            .write(true)
            .mode(0o600)
            .open(&tun_path)
        {
            Ok(_file) => {
                // TUN device opened successfully
                println!("Opened TUN device: {tun_path}");
                return Ok(interface_name);
            }
            Err(_) => {
                // Try next device
                continue;
            }
        }
    }

    Err(VpnError::Connection(
        "No available TUN devices found".to_string(),
    ))
}
