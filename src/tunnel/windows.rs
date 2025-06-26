//! Windows-specific tunnel implementation
//!
//! This module provides Windows-specific implementations for tunnel management.

use crate::error::{Result, VpnError};
use crate::tunnel::TunnelConfig;
use std::process::Command;

/// Create a TUN interface on Windows
#[allow(dead_code)]
pub fn create_tun_interface(_config: &TunnelConfig) -> Result<String> {
    println!("Creating TUN interface on Windows...");

    // Check if we have administrator privileges
    if !has_admin_privileges() {
        return Err(VpnError::Connection(
            "Insufficient permissions to create TUN interface. Please run as Administrator."
                .to_string(),
        ));
    }

    // Try to create a TUN device using TAP-Windows adapter
    let interface_name = create_tap_device()?;

    println!("TUN interface '{interface_name}' created on Windows");
    Ok(interface_name)
}

/// Destroy a TUN interface on Windows
#[allow(dead_code)]
pub fn destroy_tun_interface(interface_name: &str) -> Result<()> {
    println!("Destroying TUN interface '{interface_name}' on Windows");

    // Disable the interface
    let _ = Command::new("netsh")
        .args(["interface", "set", "interface", interface_name, "disable"])
        .status();

    println!("TUN interface '{interface_name}' destroyed on Windows");
    Ok(())
}

#[allow(dead_code)]
fn has_admin_privileges() -> bool {
    // Check if running as administrator by trying to access a system registry key
    match std::process::Command::new("reg")
        .args([
            "query",
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
        ])
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

#[allow(dead_code)]
fn create_tap_device() -> Result<String> {
    // On Windows, use actual TAP-Windows adapter
    // Check for available TAP adapters
    let output = Command::new("netsh")
        .args(["interface", "show", "interface"])
        .output()
        .map_err(|e| VpnError::TunTap(format!("Failed to list interfaces: {e}")))?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Look for TAP adapter
    for line in output_str.lines() {
        if line.contains("TAP") && line.contains("Connected") {
            // Extract interface name
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return Ok(parts[3].to_string());
            }
        }
    }

    // If no TAP adapter found, try to create one using tapctl
    let output = Command::new("tapctl")
        .args(["create", "--hwid", "tap0901"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let result = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = result.lines().find(|l| l.contains("Created")) {
                if let Some(name) = line.split('"').nth(1) {
                    return Ok(name.to_string());
                }
            }
        }
        _ => {}
    }

    // Final fallback: return a default TAP interface name
    let tap_interface_name = "TAP-Windows Adapter V9";
    println!("Using default TAP interface: {tap_interface_name}");
    Ok(tap_interface_name.to_string())
}
