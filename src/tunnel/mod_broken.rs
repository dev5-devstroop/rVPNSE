//! Tunnel Management Module
//! 
//! This module provides platform-specific TUN interface creation and management.
//! It handles the actual routing of network traffic through the VPN tunnel.

use crate::error::{Result, VpnError};
use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

/// TUN interface configuration
#[derive(Debug, Clone)]
pub struct TunnelConfig {
    pub interface_name: String,
    pub local_ip: Ipv4Addr,
    pub remote_ip: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub mtu: u16,
    pub dns_servers: Vec<Ipv4Addr>,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            interface_name: "vpnse0".to_string(),
            local_ip: Ipv4Addr::new(10, 0, 0, 2),
            remote_ip: Ipv4Addr::new(10, 0, 0, 1),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            mtu: 1500,
            dns_servers: vec![
                Ipv4Addr::new(8, 8, 8, 8),
                Ipv4Addr::new(8, 8, 4, 4),
            ],
        }
    }
}

/// Global tunnel state
lazy_static::lazy_static! {
    static ref TUNNEL_MANAGER: Arc<Mutex<Option<TunnelManager>>> = Arc::new(Mutex::new(None));
}

/// Tunnel manager for creating and managing VPN tunnels
pub struct TunnelManager {
    config: TunnelConfig,
    interface_name: String,
    original_route: Option<String>,
    original_dns: Vec<String>,
    is_established: bool,
}
impl TunnelManager {
    /// Create a new tunnel manager
    pub fn new(config: TunnelConfig) -> Self {
        Self {
            interface_name: config.interface_name.clone(),
            config,
            original_route: None,
            original_dns: Vec::new(),
            is_established: false,
        }
    }

    /// Establish the VPN tunnel
    pub fn establish_tunnel(&mut self) -> Result<()> {
        if self.is_established {
            return Ok(());
        }

        println!("Creating real VPN tunnel interface...");
        
        // Store original routing and DNS
        self.store_original_route()?;
        self.store_original_dns()?;
        
        // Create the actual TUN interface
        self.create_tun_interface()?;
        
        // Configure the interface with IP addresses
        self.configure_interface()?;
        
        // Set up routing through the tunnel
        self.setup_routing()?;
        
        // Configure DNS if specified
        self.configure_dns()?;

        self.is_established = true;

        println!("VPN tunnel established successfully!");
        println!("Interface: {}", self.interface_name);
        println!("Local IP: {}", self.config.local_ip);
        println!("Remote IP: {}", self.config.remote_ip);
        println!("All traffic is now routed through the VPN tunnel");

        Ok(())
    }

    /// Tear down the VPN tunnel
    pub fn teardown_tunnel(&mut self) -> Result<()> {
        if !self.is_established {
            return Ok(());
        }

        println!("Tearing down VPN tunnel...");

        // Restore original routing
        self.restore_original_route()?;

        // Restore original DNS
        self.restore_original_dns()?;

        // Destroy the TUN interface
        self.destroy_tun_interface()?;

        self.is_established = false;

        println!("VPN tunnel torn down successfully");
        Ok(())
    }

    /// Check if tunnel is established
    pub fn is_established(&self) -> bool {
        self.is_established
    }

    /// Get the interface name
    pub fn get_interface_name(&self) -> Option<String> {
        if self.is_established {
            Some(self.interface_name.clone())
        } else {
            None
        }
    }

    /// Get the tunnel local IP
    pub fn get_local_ip(&self) -> Option<String> {
        if self.is_established {
            Some(self.config.local_ip.to_string())
        } else {
            None
        }
    }

    /// Get the tunnel remote IP (gateway)
    pub fn get_remote_ip(&self) -> Option<String> {
        if self.is_established {
            Some(self.config.remote_ip.to_string())
        } else {
            None
        }
    }

    /// Get subnet information
    pub fn get_subnet(&self) -> Option<String> {
        if self.is_established {
            Some(format!("{}/{}", self.config.local_ip, 24))  // Assuming /24 subnet
        } else {
            None
        }
    }

    /// Get the current public IP (for testing tunnel functionality)
    pub fn get_current_public_ip(&self) -> Result<String> {
        // Query public IP through the current network configuration
        let output = Command::new("curl")
            .args(&["-s", "--max-time", "10", "https://api.ipify.org"])
            .output()
            .map_err(|e| VpnError::Connection(format!("Failed to get public IP: {}", e)))?;

        if !output.status.success() {
            return Err(VpnError::Connection("Failed to fetch public IP".to_string()));
        }

        let ip = String::from_utf8(output.stdout)
            .map_err(|e| VpnError::Connection(format!("Invalid IP response: {}", e)))?
            .trim()
            .to_string();

        if ip.is_empty() {
            return Err(VpnError::Connection("Empty IP response".to_string()));
        }

        Ok(ip)
    }

    /// Create the TUN interface
    fn create_tun_interface(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.interface_name = macos::create_tun_interface(&self.config)?;
        }

        #[cfg(target_os = "linux")]
        {
            self.interface_name = linux::create_tun_interface(&self.config)?;
        }

        #[cfg(target_os = "windows")]
        {
            self.interface_name = windows::create_tun_interface(&self.config)?;
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            return Err(VpnError::Connection("Unsupported platform for TUN interface".to_string()));
        }

        println!("TUN interface '{}' created successfully", self.interface_name);
        Ok(())
    }

    /// Destroy the TUN interface
    fn destroy_tun_interface(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            macos::destroy_tun_interface(&self.interface_name)?;
        }

        #[cfg(target_os = "linux")]
        {
            linux::destroy_tun_interface(&self.interface_name)?;
        }

        #[cfg(target_os = "windows")]
        {
            windows::destroy_tun_interface(&self.interface_name)?;
        }

        println!("TUN interface '{}' destroyed", self.interface_name);
        Ok(())
    }

        // Restore original DNS
        self.restore_dns()?;

        // The interface will be cleaned up automatically by the system
        self.is_established = false;

        println!("VPN tunnel torn down successfully");
        Ok(())
    }

    /// Store original DNS configuration
    fn store_original_dns(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("scutil")
                .args(&["--dns"])
                .output()
                .map_err(|e| VpnError::Connection(format!("Failed to get DNS config: {}", e)))?;

            if output.status.success() {
                let dns_info = String::from_utf8_lossy(&output.stdout);
                // Parse DNS servers from scutil output
                for line in dns_info.lines() {
                    if line.trim().starts_with("nameserver[") && line.contains(":") {
                        if let Some(ip_start) = line.find(": ") {
                            let ip = &line[ip_start + 2..].trim();
                            self.original_dns.push(ip.to_string());
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
                for line in content.lines() {
                    if line.starts_with("nameserver") {
                        if let Some(ip) = line.split_whitespace().nth(1) {
                            self.original_dns.push(ip.to_string());
                        }
                    }
                }
            }
        }

        println!("Original DNS servers stored: {:?}", self.original_dns);
        Ok(())
    }

    /// Store the original default route
    fn store_original_route(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("route")
                .args(&["-n", "get", "default"])
                .output()
                .map_err(|e| VpnError::Connection(format!("Failed to get default route: {}", e)))?;

            if output.status.success() {
                let route_info = String::from_utf8_lossy(&output.stdout);
                for line in route_info.lines() {
                    if line.trim().starts_with("gateway:") {
                        let gateway = line.split(':').nth(1)
                            .ok_or_else(|| VpnError::Connection("Invalid route format".to_string()))?
                            .trim();
                        self.original_route = Some(gateway.to_string());
                        break;
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("ip")
                .args(&["route", "show", "default"])
                .output()
                .map_err(|e| VpnError::Connection(format!("Failed to get default route: {}", e)))?;

            if output.status.success() {
                let route_info = String::from_utf8_lossy(&output.stdout);
                if let Some(via_pos) = route_info.find("via ") {
                    let after_via = &route_info[via_pos + 4..];
                    if let Some(space_pos) = after_via.find(' ') {
                        let gateway = &after_via[..space_pos];
                        self.original_route = Some(gateway.to_string());
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("route")
                .args(&["print", "0.0.0.0"])
                .output()
                .map_err(|e| VpnError::Connection(format!("Failed to get default route: {}", e)))?;

            if output.status.success() {
                let route_info = String::from_utf8_lossy(&output.stdout);
                for line in route_info.lines() {
                    if line.trim().starts_with("0.0.0.0") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            self.original_route = Some(parts[2].to_string());
                            break;
                        }
                    }
                }
            }
        }

        println!("Original route stored: {:?}", self.original_route);
        Ok(())
    }

    /// Configure the TUN interface with IP addresses
    fn configure_interface(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let status = Command::new("ifconfig")
                .args(&[
                    &self.interface_name,
                    &self.config.local_ip.to_string(),
                    &self.config.remote_ip.to_string(),
                    "netmask",
                    &self.config.netmask.to_string(),
                    "mtu",
                    &self.config.mtu.to_string(),
                    "up"
                ])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to configure interface: {}", e)))?;

            if !status.success() {
                return Err(VpnError::Connection("Failed to configure interface".to_string()));
            }
        }

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("ip")
                .args(&[
                    "addr", "add", &format!("{}/24", self.config.local_ip),
                    "dev", &self.interface_name
                ])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to configure interface: {}", e)))?;

            if !status.success() {
                return Err(VpnError::Connection("Failed to configure interface IP".to_string()));
            }

            let status = Command::new("ip")
                .args(&["link", "set", &self.interface_name, "mtu", &self.config.mtu.to_string()])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to set MTU: {}", e)))?;

            if !status.success() {
                return Err(VpnError::Connection("Failed to set interface MTU".to_string()));
            }

            let status = Command::new("ip")
                .args(&["link", "set", &self.interface_name, "up"])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to bring interface up: {}", e)))?;

            if !status.success() {
                return Err(VpnError::Connection("Failed to bring interface up".to_string()));
            }
        }

        #[cfg(target_os = "windows")]
        {
            let status = Command::new("netsh")
                .args(&[
                    "interface", "ipv4", "set", "address",
                    &format!("name=\"{}\"", self.interface_name),
                    "source=static",
                    &format!("address={}", self.config.local_ip),
                    &format!("mask={}", self.config.netmask),
                    &format!("gateway={}", self.config.remote_ip)
                ])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to configure interface: {}", e)))?;

            if !status.success() {
                return Err(VpnError::Connection("Failed to configure interface".to_string()));
            }
        }

        println!("Interface '{}' configured with IP {}", self.interface_name, self.config.local_ip);
        Ok(())
    }

    /// Set up routing through the tunnel
    fn setup_routing(&self) -> Result<()> {
        println!("Setting up tunnel routing...");

        #[cfg(target_os = "macos")]
        {
            // Route all traffic through the tunnel using 0.0.0.0/1 and 128.0.0.0/1
            let status1 = Command::new("route")
                .args(&["add", "-net", "0.0.0.0/1", &self.config.remote_ip.to_string()])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to add route 0.0.0.0/1: {}", e)))?;

            let status2 = Command::new("route")
                .args(&["add", "-net", "128.0.0.0/1", &self.config.remote_ip.to_string()])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to add route 128.0.0.0/1: {}", e)))?;

            if !status1.success() || !status2.success() {
                return Err(VpnError::Connection("Failed to set up tunnel routing".to_string()));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Route all traffic through the tunnel
            let status1 = Command::new("ip")
                .args(&["route", "add", "0.0.0.0/1", "via", &self.config.remote_ip.to_string(), "dev", &self.interface_name])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to add route 0.0.0.0/1: {}", e)))?;

            let status2 = Command::new("ip")
                .args(&["route", "add", "128.0.0.0/1", "via", &self.config.remote_ip.to_string(), "dev", &self.interface_name])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to add route 128.0.0.0/1: {}", e)))?;

            if !status1.success() || !status2.success() {
                return Err(VpnError::Connection("Failed to set up tunnel routing".to_string()));
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Route all traffic through the tunnel
            let status1 = Command::new("route")
                .args(&["add", "0.0.0.0", "mask", "128.0.0.0", &self.config.remote_ip.to_string()])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to add route: {}", e)))?;

            let status2 = Command::new("route")
                .args(&["add", "128.0.0.0", "mask", "128.0.0.0", &self.config.remote_ip.to_string()])
                .status()
                .map_err(|e| VpnError::Connection(format!("Failed to add route: {}", e)))?;

            if !status1.success() || !status2.success() {
                return Err(VpnError::Connection("Failed to set up tunnel routing".to_string()));
            }
        }

        println!("Tunnel routing configured successfully");
        Ok(())
    }

    /// Configure DNS servers
    fn configure_dns(&self) -> Result<()> {
        if self.config.dns_servers.is_empty() {
            return Ok(());
        }

        println!("Configuring DNS servers...");

        #[cfg(target_os = "macos")]
        {
            // Get the primary network service
            let output = Command::new("networksetup")
                .args(&["-listnetworkserviceorder"])
                .output()
                .map_err(|e| VpnError::Connection(format!("Failed to list network services: {}", e)))?;

            if output.status.success() {
                let services = String::from_utf8_lossy(&output.stdout);
                // Find the first active service (usually "Wi-Fi" or "Ethernet")
                for line in services.lines() {
                    if line.contains("Wi-Fi") || line.contains("Ethernet") {
                        if let Some(start) = line.find('"') {
                            if let Some(end) = line[start + 1..].find('"') {
                                let service_name = &line[start + 1..start + 1 + end];
                                
                                // Set DNS servers for this service
                                let dns_args: Vec<String> = self.config.dns_servers
                                    .iter()
                                    .map(|ip| ip.to_string())
                                    .collect();
                                
                                let mut args = vec!["-setdnsservers", service_name];
                                for dns in &dns_args {
                                    args.push(dns);
                                }
                                
                                let status = Command::new("networksetup")
                                    .args(&args)
                                    .status()
                                    .map_err(|e| VpnError::Connection(format!("Failed to set DNS: {}", e)))?;

                                if status.success() {
                                    println!("DNS configured for service: {}", service_name);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Create a backup of resolv.conf and update it
            std::fs::copy("/etc/resolv.conf", "/etc/resolv.conf.vpnse.backup")
                .map_err(|e| VpnError::Connection(format!("Failed to backup resolv.conf: {}", e)))?;

            let mut dns_content = String::new();
            for dns in &self.config.dns_servers {
                dns_content.push_str(&format!("nameserver {}\n", dns));
            }

            std::fs::write("/etc/resolv.conf", dns_content)
                .map_err(|e| VpnError::Connection(format!("Failed to update resolv.conf: {}", e)))?;
        }

        #[cfg(target_os = "windows")]
        {
            // Set DNS servers for the tunnel interface
            for (i, dns) in self.config.dns_servers.iter().enumerate() {
                let index = if i == 0 { "primary" } else { "secondary" };
                let status = Command::new("netsh")
                    .args(&[
                        "interface", "ipv4", "set", "dns",
                        &format!("name=\"{}\"", self.interface_name),
                        "source=static",
                        &format!("address={}", dns),
                        index
                    ])
                    .status()
                    .map_err(|e| VpnError::Connection(format!("Failed to set DNS: {}", e)))?;

                if !status.success() {
                    return Err(VpnError::Connection("Failed to configure DNS".to_string()));
                }
            }
        }

        println!("DNS servers configured: {:?}", self.config.dns_servers);
        Ok(())
    }

    /// Restore original routing
    fn restore_original_route(&self) -> Result<()> {
        if let Some(ref _original_gateway) = self.original_route {
            #[cfg(target_os = "macos")]
            {
                // Remove VPN routes
                let _ = Command::new("route")
                    .args(&["delete", "-net", "0.0.0.0/1"])
                    .status();

                let _ = Command::new("route")
                    .args(&["delete", "-net", "128.0.0.0/1"])
                    .status();

                println!("Original routing restored");
            }

            #[cfg(target_os = "linux")]
            {
                // Remove VPN routes
                let _ = Command::new("ip")
                    .args(&["route", "del", "0.0.0.0/1"])
                    .status();

                let _ = Command::new("ip")
                    .args(&["route", "del", "128.0.0.0/1"])
                    .status();

                println!("Original routing restored");
            }

            #[cfg(target_os = "windows")]
            {
                // Remove VPN routes
                let _ = Command::new("route")
                    .args(&["delete", "0.0.0.0"])
                    .status();

                let _ = Command::new("route")
                    .args(&["delete", "128.0.0.0"])
                    .status();

                println!("Original routing restored");
            }
        }

        Ok(())
    }

    /// Restore original DNS
    fn restore_dns(&self) -> Result<()> {
        // Platform-specific DNS restoration would go here
        println!("DNS configuration restored");
        Ok(())
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        let _ = self.teardown_tunnel();
    }
}

// Public API functions
pub fn create_tunnel_interface() -> Result<()> {
    let config = TunnelConfig::default();
    let mut manager = TunnelManager::new(config);
    manager.establish_tunnel()?;
    
    // Store the manager globally
    {
        let mut global_manager = TUNNEL_MANAGER.lock().unwrap();
        *global_manager = Some(manager);
    }
    
    Ok(())
}

pub fn destroy_tunnel_interface() -> Result<()> {
    let mut manager = {
        let mut global_manager = TUNNEL_MANAGER.lock().unwrap();
        global_manager.take()
    };
    
    if let Some(ref mut mgr) = manager {
        mgr.teardown_tunnel()?;
    }
    
    Ok(())
}

pub fn get_tunnel_interface() -> Option<(String, String, String, String)> {
    let global_manager = TUNNEL_MANAGER.lock().unwrap();
    if let Some(ref manager) = *global_manager {
        if manager.is_established() {
            return Some((
                manager.get_interface_name().unwrap_or_default(),
                manager.get_local_ip().unwrap_or_default(),
                manager.get_remote_ip().unwrap_or_default(),
                manager.get_subnet().unwrap_or_default()
            ));
        }
    }
    None
}

pub fn get_tunnel_public_ip() -> Result<String> {
    let global_manager = TUNNEL_MANAGER.lock().unwrap();
    if let Some(ref manager) = *global_manager {
        manager.get_current_public_ip()
    } else {
        Err(VpnError::Connection("No tunnel established".to_string()))
    }
}
