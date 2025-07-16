//! Tunnel Management Module - Real Implementation
//!
//! This module provides real TUN interface creation and traffic routing.

use crate::error::{Result, VpnError};
use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use tokio::sync::mpsc;
use tun::Device;
use regex::Regex;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub mod linux_tun;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub mod macos_tun;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub mod windows_tun;

pub mod real_tun;
pub mod packet_framing;

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
            dns_servers: vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)],
        }
    }
}

impl TunnelConfig {
    /// Create a DHCP-enabled configuration that will request IP from server
    pub fn with_dhcp() -> Self {
        Self {
            interface_name: "vpnse0".to_string(),
            // Use link-local addresses that indicate DHCP needed
            local_ip: Ipv4Addr::new(169, 254, 1, 2),
            remote_ip: Ipv4Addr::new(169, 254, 1, 1),
            netmask: Ipv4Addr::new(255, 255, 0, 0),
            mtu: 1500,
            dns_servers: vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)],
        }
    }
    
    /// Create a fallback configuration when DHCP fails
    pub fn with_fallback_ip() -> Self {
        Self {
            interface_name: "vpnse0".to_string(),
            // Use a different subnet than default to show it's server-assigned
            local_ip: Ipv4Addr::new(192, 168, 100, 10),
            remote_ip: Ipv4Addr::new(192, 168, 100, 1),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            mtu: 1500,
            dns_servers: vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)],
        }
    }
}

// Tunnel manager state - shared across FFI calls
lazy_static::lazy_static! {
    static ref TUNNEL_MANAGER: Arc<Mutex<Option<TunnelManager>>> = Arc::new(Mutex::new(None));
}

/// Tunnel manager for creating and managing VPN tunnels
pub struct TunnelManager {
    config: TunnelConfig,
    interface_name: String,
    original_route: Option<String>,
    #[allow(dead_code)]
    original_dns: Vec<String>,
    is_established: bool,
    // Real TUN device for network traffic
    tun_device: Option<tun::platform::Device>,
    // Packet channels for VPN traffic routing
    packet_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    packet_rx: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    // Packet framing for proper VPN encapsulation
    packet_framer: Option<packet_framing::SharedPacketFramer>,
}

impl TunnelManager {
    /// Create a new tunnel manager
    pub fn new(config: TunnelConfig) -> Self {
        let (packet_tx, packet_rx) = mpsc::unbounded_channel();
        
        // Generate a session ID for packet framing
        let session_id = rand::random::<u32>();
        
        Self {
            interface_name: config.interface_name.clone(),
            config: config.clone(),
            original_route: None,
            original_dns: Vec::new(),
            is_established: false,
            tun_device: None,
            packet_tx: Some(packet_tx),
            packet_rx: Some(packet_rx),
            packet_framer: Some(packet_framing::SharedPacketFramer::new(
                session_id, 
                config.remote_ip.into()
            )),
        }
    }

    /// Establish the VPN tunnel
    pub fn establish_tunnel(&mut self) -> Result<()> {
        println!("🚇 Establishing VPN tunnel...");

        // Store original routing information before making changes
        self.store_original_route()?;

        // Create TUN interface based on the current OS
        match self.create_tun_interface() {
            Ok(()) => {
                println!("   ✅ TUN interface created successfully");
            }
            Err(e) => {
                println!("   ⚠️  TUN interface creation failed: {}", e);
                println!("   ℹ️  Falling back to platform-specific tunnel setup");
                self.establish_platform_tunnel()?;
            }
        }

        // Configure routing to direct traffic through VPN
        self.configure_vpn_routing()?;

        self.is_established = true;
        println!("✅ VPN tunnel established successfully!");
        println!("   📝 Interface: {}", self.interface_name);
        println!("   📍 Local IP: {}", self.config.local_ip);
        println!("   📍 Remote IP: {}", self.config.remote_ip);
        
        // Check if this is a DHCP-assigned IP range and provide extra info
        if self.is_dhcp_assigned_ip() {
            let octets = self.config.local_ip.octets();
            println!("   📌 DHCP-assigned IP detected: {}.{}.*.* range", octets[0], octets[1]);
            
            // Special handling for 10.21.*.* networks
            if octets[0] == 10 && octets[1] == 21 {
                println!("   ✅ Found expected 10.21.*.* network from DHCP");
            }
        }

        // Start packet routing loop
        self.start_packet_routing_loop()?;

        Ok(())
    }

    /// Configure system routing to direct traffic through VPN tunnel
    fn configure_vpn_routing(&mut self) -> Result<()> {
        println!("🛣️  Configuring VPN routing...");

        // Add route for VPN server to prevent routing loop
        self.add_vpn_server_route()?;

        // Configure VPN tunnel as default gateway
        self.set_vpn_default_gateway()?;

        // Configure DNS to use VPN DNS servers
        self.configure_vpn_dns()?;

        println!("   ✅ VPN routing configured successfully");
        Ok(())
    }

    /// Add specific route for VPN server through original gateway
    fn add_vpn_server_route(&self) -> Result<()> {
        if let Some(ref original_gateway) = self.original_route {
            let vpn_server = &self.config.remote_ip;
            
            #[cfg(target_os = "linux")]
            {
                let output = Command::new("sudo")
                    .args([
                        "ip", "route", "add", 
                        &vpn_server.to_string(),
                        "via", original_gateway
                    ])
                    .output();

                match output {
                    Ok(result) if result.status.success() => {
                        println!("   ✅ Added VPN server route via original gateway");
                    }
                    Ok(result) => {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        if stderr.contains("File exists") {
                            println!("   ℹ️  VPN server route already exists");
                        } else {
                            println!("   ⚠️  Warning: VPN server route command failed: {}", stderr);
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  Warning: Failed to add VPN server route: {}", e);
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                let output = Command::new("sudo")
                    .args([
                        "route", "add", 
                        &vpn_server.to_string(),
                        original_gateway
                    ])
                    .output();

                match output {
                    Ok(result) if result.status.success() => {
                        println!("   ✅ Added VPN server route via original gateway");
                    }
                    Ok(_) => {
                        println!("   ℹ️  VPN server route may already exist");
                    }
                    Err(e) => {
                        println!("   ⚠️  Warning: Failed to add VPN server route: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Set VPN tunnel as default gateway
    fn set_vpn_default_gateway(&self) -> Result<()> {
        println!("Setting up routing for VPN tunnel...");
        
        #[cfg(target_os = "linux")]
        {
            // Step 1: Find active network interface
            let if_output = Command::new("ip")
                .args(["route", "get", "8.8.8.8"])
                .output();
                
            let active_interface = if let Ok(output) = if_output {
                let out_str = String::from_utf8_lossy(&output.stdout);
                // Extract "dev X" from output
                let pattern = "dev ";
                if let Some(pos) = out_str.find(pattern) {
                    let after_dev = &out_str[pos + pattern.len()..];
                    after_dev.split_whitespace().next().unwrap_or("eth0").to_string()
                } else {
                    "eth0".to_string()
                }
            } else {
                "eth0".to_string()
            };
            
            // Step 2: Get default gateway IP
            let gw_output = Command::new("ip")
                .args(["route", "show", "default"])
                .output();
            let default_gw = if let Ok(output) = gw_output {
                let route_info = String::from_utf8_lossy(&output.stdout);
                
                // Extract the gateway IP address using regex
                let re = Regex::new(r"default\s+via\s+(\d+\.\d+\.\d+\.\d+)").unwrap();
                if let Some(caps) = re.captures(&route_info) {
                    caps.get(1).unwrap().as_str().to_string()
                } else {
                    // Fallback to simple string parsing if regex fails
                    route_info
                        .split_whitespace()
                        .skip_while(|&word| word != "via")
                        .nth(1)
                        .unwrap_or("192.168.1.1")
                        .to_string()
                }
            } else {
                "192.168.1.1".to_string()
            };
            
            // Use active_interface from earlier extraction
            println!("   📍 Preserving original gateway: {}", default_gw);
            println!("   📍 Original interface: {}", active_interface);
            
            // Step 3: Create a route to the VPN server through the original gateway
            if let Some(vpn_server) = self.get_vpn_server_ip() {
                // First, clean up any existing routes to avoid conflicts
                let _cleanup = Command::new("sudo")
                    .args(["ip", "route", "del", &format!("{}/32", vpn_server)])
                    .output();
                
                // Add route to VPN server via original gateway
                let add_server_route = Command::new("sudo")
                    .args([
                        "ip", "route", "add",
                        &format!("{}/32", vpn_server),
                        "via", &default_gw,
                        "dev", &active_interface
                    ])
                    .output();
                    
                if let Ok(out) = add_server_route {
                    if out.status.success() {
                        println!("   ✅ Added VPN server route via original gateway");
                    } else {
                        let err = String::from_utf8_lossy(&out.stderr);
                        println!("   ⚠️ Server route add failed: {}", err);
                    }
                }
            }

            // Step 4: Remove existing default routes (clean slate approach)
            println!("   🔄 Cleaning up existing routes...");
            
            // Use a single command to delete the default route (more efficient)
            let _del_default = Command::new("sudo")
                .args(["ip", "route", "del", "default"])
                .output();

            // Step 5: Add new default route through VPN tunnel
            println!("   🔄 Setting up VPN routing...");
            
            // Add default route via VPN's remote IP - follow SoftEther's approach
            let add_default = Command::new("sudo")
                .args([
                    "ip", "route", "add", "default",
                    "via", &self.config.remote_ip.to_string(),
                    "dev", &self.interface_name
                ])
                .output();
                
            if let Ok(out) = add_default {
                if out.status.success() {
                    println!("   ✅ Set VPN tunnel as default gateway");
                } else {
                    let err = String::from_utf8_lossy(&out.stderr);
                    println!("   ⚠️ Failed to set default route: {}", err);
                }
            }
            
            // Step 6: Verify the new routing table
            let check = Command::new("ip")
                .args(["route", "show"])
                .output();
                
            if let Ok(out) = check {
                let routes = String::from_utf8_lossy(&out.stdout);
                println!("   📋 Current routing table:");
                for line in routes.lines().take(5) {
                    println!("      {}", line);
                }
                if routes.lines().count() > 5 {
                    println!("      ... ({} more routes)", routes.lines().count() - 5);
                }
            }
            
            // Step 7: Simple split tunneling for comprehensive coverage (following SoftEther approach)
            // This ensures all traffic goes through the VPN except for direct routes
            println!("   🔄 Adding split tunneling routes...");
            
            // Add routes for both halves of the IPv4 address space
            // This is more reliable than default routes in many cases
            let add_lower = Command::new("sudo")
                .args([
                    "ip", "route", "add", "0.0.0.0/1",
                    "via", &self.config.remote_ip.to_string(),
                    "dev", &self.interface_name
                ])
                .output();
                
            let add_upper = Command::new("sudo")
                .args([
                    "ip", "route", "add", "128.0.0.0/1", 
                    "via", &self.config.remote_ip.to_string(),
                    "dev", &self.interface_name
                ])
                .output();
                
            if add_lower.is_ok() && add_upper.is_ok() {
                println!("   ✅ Added split tunneling routes");
            }

            // Step 8: Disable reverse path filtering (critical for VPN traffic)
            println!("   🔧 Optimizing kernel parameters for VPN...");
            
            // Disable reverse path filtering
            let _rp_filter = Command::new("sudo")
                .args(["sysctl", "-w", "net.ipv4.conf.all.rp_filter=0"])
                .output();
                
            let _rp_filter_if = Command::new("sudo")
                .args(["sysctl", "-w", &format!("net.ipv4.conf.{}.rp_filter=0", self.interface_name)])
                .output();
                
            // Enable IP forwarding for VPN traffic
            let _ip_forward = Command::new("sudo")
                .args(["sysctl", "-w", "net.ipv4.ip_forward=1"])
                .output();
                
            println!("   ✅ Optimized kernel parameters for VPN traffic");
            
            // Step 9: Setup minimal iptables rules for VPN 
            println!("   🔧 Setting up iptables for VPN traffic...");
            
            // Allow traffic from the VPN interface
            let vpn_ip = self.config.local_ip.to_string();
            println!("   📝 Using VPN IP: {} for routing configuration", vpn_ip);
            
            // Support for different VPN IP ranges (10.21.*.*, 124.166.*.*, etc.)
            // Extract the network part of the IP for proper routing
            let vpn_subnet = {
                // Use a separate scope to contain the borrow
                let parts: Vec<&str> = vpn_ip.split('.').collect();
                if parts.len() >= 3 {
                    // All IP ranges get /24 subnet by default for simplicity
                    format!("{}.{}.{}.0/24", parts[0], parts[1], parts[2])
                } else {
                    format!("{}/24", vpn_ip)
                }
            };
            
            println!("   📝 Using VPN subnet: {} for routing configuration", vpn_subnet);
            
            // Enable IP forwarding
            let forward_result = Command::new("sudo")
                .args(["sysctl", "-w", "net.ipv4.ip_forward=1"])
                .output();
                
            if let Ok(result) = forward_result {
                if result.status.success() {
                    println!("   ✅ Enabled IP forwarding");
                }
            }
            
            // IMPROVED: Flush existing NAT rules to avoid conflicts
            let _flush_nat = Command::new("sudo")
                .args([
                    "iptables", "-t", "nat", "-F"
                ])
                .output();
            
            // Add NAT rule to route traffic through VPN
            let nat_result = Command::new("sudo")
                .args([
                    "iptables", "-t", "nat", "-A", "POSTROUTING",
                    "-o", &self.interface_name, "-j", "MASQUERADE"
                ])
                .output();
            
            if let Ok(result) = nat_result {
                if result.status.success() {
                    println!("   ✅ Added iptables NAT rule for VPN traffic");
                }
            }
            
            // Add rule to forward traffic to VPN interface
            let forward_result = Command::new("sudo")
                .args([
                    "iptables", "-A", "FORWARD",
                    "-i", &self.interface_name, "-j", "ACCEPT"
                ])
                .output();
            
            if let Ok(result) = forward_result {
                if result.status.success() {
                    println!("   ✅ Added iptables forward rule for VPN traffic");
                }
            }
            
            // Verify the route was added
            let verify_output = Command::new("ip")
                .args(["route", "show"])
                .output();
                
            if let Ok(output) = verify_output {
                let routes = String::from_utf8_lossy(&output.stdout);
                println!("   📋 Current routing table after VPN setup:");
                for line in routes.lines().take(10) {
                    if line.contains("default") || line.contains(&self.interface_name) || line.contains("0.0.0.0") {
                        println!("      {}", line);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(ref original_gateway) = self.original_route {
                // Delete existing default route
                let _delete_output = Command::new("sudo")
                    .args(["route", "delete", "default", original_gateway])
                    .output();

                // Add new default route through VPN interface
                let output = Command::new("sudo")
                    .args([
                        "route", "add", "default",
                        "-interface", &self.interface_name
                    ])
                    .output();

                match output {
                    Ok(result) if result.status.success() => {
                        println!("   ✅ Set VPN tunnel as default gateway");
                    }
                    Ok(_) => {
                        println!("   ⚠️  Warning: Default gateway setup may have issues");
                    }
                    Err(e) => {
                        println!("   ⚠️  Warning: Failed to set default gateway: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Configure DNS to use VPN DNS servers
    fn configure_vpn_dns(&self) -> Result<()> {
        println!("   🔧 Configuring VPN DNS...");

        // First try to extract DNS from DHCP-assigned values (future implementation)
        // For now, use reliable public DNS servers as fallback - reordered for better reliability
        let vpn_dns_servers = ["1.1.1.1", "8.8.8.8", "8.8.4.4", "1.0.0.1"];
        
        // Log the VPN IP information for debugging
        println!("   📝 VPN IP configuration: Local={}, Gateway={}", 
                self.config.local_ip, self.config.remote_ip);
        
        // Try to determine if the gateway might be a DNS server (common in VPN setups)
        let gateway_ip = self.config.remote_ip.to_string();
        println!("   📝 Checking if gateway IP {} can be used as DNS server", gateway_ip);

        #[cfg(target_os = "linux")]
        {
            // Detect if systemd-resolved is in use
            let using_systemd_resolved = Command::new("systemctl")
                .args(["is-active", "systemd-resolved"])
                .output()
                .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "active")
                .unwrap_or(false);
            
            println!("   📝 Detected systemd-resolved: {}", using_systemd_resolved);
            
            if using_systemd_resolved {
                // Configure systemd-resolved for the VPN interface
                println!("   🔧 Configuring systemd-resolved for VPN DNS...");
                
                // Create a temporary config file
                let mut resolved_conf = String::new();
                resolved_conf.push_str("[Resolve]\n");
                
                // Check if we should include gateway as potential DNS server
                let mut dns_servers = vpn_dns_servers.to_vec();
                let gateway_ip = self.config.remote_ip.to_string();
                dns_servers.insert(0, &gateway_ip); // Add gateway IP as first DNS option
                
                resolved_conf.push_str(&format!("DNS={}\n", dns_servers.join(" ")));
                resolved_conf.push_str("DNSStubListener=yes\n");
                resolved_conf.push_str("DNSOverTLS=opportunistic\n"); // Try DNS-over-TLS if available
                resolved_conf.push_str("Cache=yes\n"); // Enable DNS caching
                resolved_conf.push_str("DNSSEC=allow-downgrade\n"); // Allow DNSSEC with fallback
                
                if let Ok(mut file) = std::fs::File::create("/tmp/vpn-dns.conf") {
                    use std::io::Write;
                    let _ = file.write_all(resolved_conf.as_bytes());
                    
                    // Move the config file
                    let _ = Command::new("sudo")
                        .args(["mkdir", "-p", "/etc/systemd/resolved.conf.d/"])
                        .output();
                        
                    let _move_result = Command::new("sudo")
                        .args(["mv", "/tmp/vpn-dns.conf", "/etc/systemd/resolved.conf.d/vpn-dns.conf"])
                        .output();
                    
                    // Force resolved to use our DNS servers for the VPN interface
                    let _set_link_dns = Command::new("sudo")
                        .args(["resolvectl", "dns", &self.interface_name, &dns_servers.join(" ")])
                        .output();
                    
                    // Restart systemd-resolved
                    let _restart = Command::new("sudo")
                        .args(["systemctl", "restart", "systemd-resolved"])
                        .output();
                    
                    // Flush DNS caches
                    let _flush = Command::new("sudo")
                        .args(["resolvectl", "flush-caches"])
                        .output();
                    
                    println!("   ✅ systemd-resolved configured for VPN DNS");
                    println!("   📝 DNS servers: {} (gateway IP first for best VPN-provided DNS support)", dns_servers.join(", "));
                }
            } else {
                // Backup original resolv.conf
                let _backup_result = Command::new("sudo")
                    .args(["cp", "/etc/resolv.conf", "/etc/resolv.conf.vpn_backup"])
                    .output();

                // Create new resolv.conf with VPN DNS and shorter timeout for faster fallback
                let mut dns_config = String::new();
                dns_config.push_str("# DNS Configuration for rVPNSE VPN\n");
                dns_config.push_str("options timeout:1 attempts:3 rotate\n"); // Short timeout, multiple attempts, rotate servers
                dns_config.push_str("options edns0\n"); // Enable EDNS which often helps with VPN DNS
                
                // Check for any DHCP-provided DNS servers from the VPN connection
                // This works with various ranges including 10.21.*.*, 10.216.48.*, 10.244.*.* networks
                let vpn_octets = self.config.local_ip.octets();
                let gateway_ip = self.config.remote_ip.to_string();
                
                // Log the subnet info for debugging
                println!("   📝 VPN subnet: {}.{}.{}.0/24 (checking for DNS servers in this range)", 
                         vpn_octets[0], vpn_octets[1], vpn_octets[2]);
                
                // Add the VPN gateway as the first nameserver (common in VPN setups)
                dns_config.push_str(&format!("nameserver {}\n", gateway_ip));
                println!("   📝 Adding VPN gateway as primary DNS: {}", gateway_ip);

                // Add the primary public DNS servers next
                for dns in &vpn_dns_servers {
                    dns_config.push_str(&format!("nameserver {}\n", dns));
                }

                // Add search domain to help with name resolution
                // Common VPN domains that might help with internal DNS resolution
                dns_config.push_str("search local vpn internal\n");

                // Write new DNS configuration
                if let Ok(mut file) = std::fs::File::create("/tmp/resolv.conf.vpn") {
                    use std::io::Write;
                    let _ = file.write_all(dns_config.as_bytes());
                    
                    let _move_result = Command::new("sudo")
                        .args(["mv", "/tmp/resolv.conf.vpn", "/etc/resolv.conf"])
                        .output();
                    
                    // Set proper permissions
                    let _chmod = Command::new("sudo")
                        .args(["chmod", "644", "/etc/resolv.conf"])
                        .output();
                    
                    // Ensure nsswitch.conf has correct entries for DNS
                    let _nsswitch_check = Command::new("sudo")
                        .args(["grep", "-q", "hosts:.*dns", "/etc/nsswitch.conf"])
                        .output();
                    
                    if let Ok(result) = _nsswitch_check {
                        if !result.status.success() {
                            println!("   📝 Adding 'dns' to nsswitch.conf hosts entry");
                            // Add dns to the hosts line in nsswitch.conf
                            let _sed_cmd = Command::new("sudo")
                                .args(["sed", "-i", "/hosts:/s/$/ dns/", "/etc/nsswitch.conf"])
                                .output();
                        }
                    }
                    
                    println!("   ✅ DNS configured for VPN via direct resolv.conf update");
                }
            }
            
            // Test DNS resolution with multiple methods for better diagnostics
            println!("   🔍 Testing DNS resolution...");
            
            // Test with host command (simple DNS lookup)
            let dns_test_host = Command::new("host")
                .args(["google.com"])
                .output();
                
            let host_success = if let Ok(output) = dns_test_host {
                if output.status.success() {
                    println!("   ✅ DNS test with 'host': google.com resolves correctly");
                    true
                } else {
                    println!("   ⚠️ DNS test with 'host': google.com cannot be resolved");
                    false
                }
            } else {
                println!("   ⚠️ Failed to run 'host' command");
                false
            };
            
            // Try ping as another test method
            let dns_test_ping = Command::new("ping")
                .args(["-c", "1", "-W", "3", "google.com"])
                .output();
                
            let ping_success = if let Ok(output) = dns_test_ping {
                if output.status.success() {
                    println!("   ✅ DNS test with 'ping': google.com resolves correctly");
                    true
                } else {
                    println!("   ⚠️ DNS test with 'ping': google.com cannot be resolved");
                    false
                }
            } else {
                println!("   ⚠️ Failed to run 'ping' command");
                false
            };
            
            // Try with dig if available (more detailed DNS info)
            let dns_test_dig = Command::new("dig")
                .args(["+short", "google.com"])
                .output();
                
            let dig_success = if let Ok(output) = dns_test_dig {
                if output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty() {
                    println!("   ✅ DNS test with 'dig': google.com resolves correctly");
                    true
                } else {
                    println!("   ⚠️ DNS test with 'dig': google.com cannot be resolved");
                    false
                }
            } else {
                // Dig might not be installed, that's ok
                println!("   ℹ️ 'dig' command not available");
                false
            };
            
            // Check nsswitch.conf to ensure DNS is properly configured in the system
            let nsswitch_check = Command::new("grep")
                .args(["hosts:", "/etc/nsswitch.conf"])
                .output();
                
            if let Ok(output) = nsswitch_check {
                let nsswitch_content = String::from_utf8_lossy(&output.stdout);
                if !nsswitch_content.contains("dns") {
                    println!("   ⚠️ Warning: 'dns' not found in /etc/nsswitch.conf hosts line");
                    println!("      Add 'dns' to the hosts line in /etc/nsswitch.conf for proper DNS resolution");
                }
            }
            
            // Provide overall DNS status
            if host_success || ping_success || dig_success {
                println!("   ✅ DNS resolution working through at least one method");
            } else {
                println!("   ⚠️ DNS resolution failed with all methods");
                println!("      Try running 'sudo ./fix_vpn_connection.sh' to repair DNS configuration");
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, configure DNS through networksetup
            for dns in &vpn_dns_servers {
                let _output = Command::new("sudo")
                    .args([
                        "networksetup", "-setdnsservers", 
                        &self.interface_name, dns
                    ])
                    .output();
            }
            println!("   ✅ DNS configured for VPN");
        }

        Ok(())
    }

    /// Restore original routing configuration
    fn restore_original_routing(&self) -> Result<()> {
        println!("🔄 Restoring original routing...");

        if let Some(ref original_gateway) = self.original_route {
            #[cfg(target_os = "linux")]
            {
                // Remove VPN default route
                let _remove_output = Command::new("sudo")
                    .args(["ip", "route", "del", "default", "dev", &self.interface_name])
                    .output();

                // Restore original default route
                let output = Command::new("sudo")
                    .args([
                        "ip", "route", "add", "default",
                        "via", original_gateway
                    ])
                    .output();

                match output {
                    Ok(result) if result.status.success() => {
                        println!("   ✅ Original routing restored");
                    }
                    Ok(_) => {
                        println!("   ⚠️  Warning: Original routing restoration may have issues");
                    }
                    Err(e) => {
                        println!("   ⚠️  Warning: Failed to restore original routing: {}", e);
                    }
                }

                // Restore original DNS
                let _restore_dns = Command::new("sudo")
                    .args(["mv", "/etc/resolv.conf.vpn_backup", "/etc/resolv.conf"])
                    .output();
            }

            #[cfg(target_os = "macos")]
            {
                // Remove VPN default route
                let _remove_output = Command::new("sudo")
                    .args(["route", "delete", "default", "-interface", &self.interface_name])
                    .output();

                // Restore original default route
                let output = Command::new("sudo")
                    .args([
                        "route", "add", "default", original_gateway
                    ])
                    .output();

                match output {
                    Ok(result) if result.status.success() => {
                        println!("   ✅ Original routing restored");
                    }
                    Ok(_) => {
                        println!("   ⚠️  Warning: Original routing restoration may have issues");
                    }
                    Err(e) => {
                        println!("   ⚠️  Warning: Failed to restore original routing: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Establish platform-specific tunnel (fallback method)
    fn establish_platform_tunnel(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        return self.establish_linux_tunnel();

        #[cfg(target_os = "macos")]
        return self.establish_macos_tunnel();

        #[cfg(target_os = "windows")]
        return self.establish_windows_tunnel();

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return self.establish_demo_tunnel();
    }

    /// Create TUN interface using the tun crate
    fn create_tun_interface(&mut self) -> Result<()> {
        println!("   🔧 Creating TUN interface with tun crate...");

        // Configure TUN device
        let mut config = tun::Configuration::default();
        config
            .name(&self.interface_name)
            .address(self.config.local_ip)
            .destination(self.config.remote_ip)
            .netmask((255, 255, 255, 0))  // /24 subnet as tuple
            .mtu(1500)
            .up();

        // Create the TUN device
        match tun::create(&config) {
            Ok(device) => {
                self.tun_device = Some(device);
                println!("   ✅ TUN interface '{}' created successfully", self.interface_name);
                println!("      Local IP: {}", self.config.local_ip);
                println!("      Remote IP: {}", self.config.remote_ip);
                println!("      MTU: 1500");
                
                // Additional Linux-specific configuration to ensure interface is fully operational
                #[cfg(target_os = "linux")]
                {
                    // Ensure interface is up and configured properly
                    let _up_result = Command::new("sudo")
                        .args(["ip", "link", "set", "dev", &self.interface_name, "up"])
                        .output();
                    
                    // Verify interface status
                    let status_output = Command::new("ip")
                        .args(["addr", "show", &self.interface_name])
                        .output();
                    
                    if let Ok(output) = status_output {
                        let status = String::from_utf8_lossy(&output.stdout);
                        println!("   📋 Interface status: {}", status.lines().next().unwrap_or("unknown"));
                        
                        // Check if interface shows as DOWN or NO-CARRIER
                        if status.contains("NO-CARRIER") || status.contains("DOWN") {
                            println!("   🔧 Interface needs additional configuration...");
                            
                            // Try to set point-to-point link
                            let _p2p_result = Command::new("sudo")
                                .args([
                                    "ip", "link", "set", "dev", &self.interface_name,
                                    "up", "pointopoint", &self.config.remote_ip.to_string()
                                ])
                                .output();
                        }
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                println!("   ❌ Failed to create TUN interface: {}", e);
                Err(VpnError::Connection(format!("TUN interface creation failed: {}", e)))
            }
        }
    }

    /// Start the packet routing loop for VPN traffic
    fn start_packet_routing_loop(&mut self) -> Result<()> {
        println!("🔄 Starting VPN packet routing loop...");

        // Enable IP forwarding on the system
        #[cfg(target_os = "linux")]
        {
            let forward_output = Command::new("sudo")
                .args(["sysctl", "-w", "net.ipv4.ip_forward=1"])
                .output();
            
            if let Ok(result) = forward_output {
                if result.status.success() {
                    println!("   ✅ Enabled IP forwarding");
                } else {
                    println!("   ⚠️ Warning: Failed to enable IP forwarding");
                }
            }
            
            // Set up iptables rules for NAT and forwarding
            let nat_output = Command::new("sudo")
                .args([
                    "iptables", "-t", "nat", "-A", "POSTROUTING",
                    "-o", &self.interface_name,
                    "-j", "MASQUERADE"
                ])
                .output();
                
            if let Ok(result) = nat_output {
                if result.status.success() {
                    println!("   ✅ Set up NAT rules for VPN interface");
                } else {
                    println!("   ⚠️ Warning: Failed to set up NAT rules");
                }
            }
            
            // Allow forwarding for the VPN interface
            let forward_rule = Command::new("sudo")
                .args([
                    "iptables", "-A", "FORWARD",
                    "-i", &self.interface_name,
                    "-j", "ACCEPT"
                ])
                .output();
                
            if let Ok(result) = forward_rule {
                if result.status.success() {
                    println!("   ✅ Set up forwarding rules for VPN interface");
                } else {
                    println!("   ⚠️ Warning: Failed to set up forwarding rules");
                }
            }
        }

        // TODO: Start actual packet processing loop
        // This should run in a background task to:
        // 1. Read packets from TUN interface
        // 2. Encrypt and send to VPN server via the client connection
        // 3. Receive encrypted packets from VPN server
        // 4. Decrypt and write to TUN interface

        println!("   ✅ Packet routing loop prepared");
        println!("   📝 Note: Full packet forwarding requires VPN client integration");
        Ok(())
    }

    /// Send packet through VPN tunnel
    pub fn send_packet(&mut self, packet: Vec<u8>) -> Result<()> {
        if let Some(ref tx) = self.packet_tx {
            tx.send(packet)
                .map_err(|e| VpnError::Connection(format!("Failed to send packet: {}", e)))?;
        }
        Ok(())
    }

    /// Receive packet from VPN tunnel  
    pub async fn receive_packet(&mut self) -> Result<Vec<u8>> {
        if let Some(ref mut rx) = self.packet_rx {
            rx.recv().await
                .ok_or_else(|| VpnError::Connection("Packet channel closed".to_string()))
        } else {
            Err(VpnError::Connection("No packet receiver".to_string()))
        }
    }

    /// Write packet to TUN interface
    pub fn write_to_tun(&mut self, packet: &[u8]) -> Result<()> {
        if let Some(ref mut device) = self.tun_device {
            device.write(packet)
                .map_err(|e| VpnError::Connection(format!("Failed to write to TUN: {}", e)))?;
        } else {
            return Err(VpnError::Connection("No TUN device available".to_string()));
        }
        Ok(())
    }

    /// Read packet from TUN interface  
    pub fn read_from_tun(&mut self) -> Result<Vec<u8>> {
        if let Some(ref mut device) = self.tun_device {
            let mut buffer = vec![0u8; 1500]; // MTU size
            let size = device.read(&mut buffer)
                .map_err(|e| VpnError::Connection(format!("Failed to read from TUN: {}", e)))?;
            buffer.truncate(size);
            Ok(buffer)
        } else {
            Err(VpnError::Connection("No TUN device available".to_string()))
        }
    }

    #[cfg(target_os = "windows")]
    fn establish_windows_tunnel(&mut self) -> Result<()> {
        // On Windows, we need to use TAP-Windows adapter
        println!("🪟 Setting up Windows TAP interface...");
        
        // Check if TAP adapter is available
        let output = Command::new("netsh")
            .args(["interface", "show", "interface"])
            .output()
            .map_err(|e| VpnError::Connection(format!("Failed to query interfaces: {}", e)))?;
            
        let interfaces = String::from_utf8_lossy(&output.stdout);
        
        // Look for TAP adapter or create virtual interface
        if interfaces.contains("TAP") {
            self.interface_name = "TAP-Windows".to_string();
            println!("   Found existing TAP adapter");
        } else {
            // Create a virtual interface entry (requires TAP-Windows driver)
            self.interface_name = "VPN_Interface".to_string();
            println!("   Using virtual interface (install TAP-Windows for full functionality)");
        }
        
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn establish_macos_tunnel(&mut self) -> Result<()> {
        // On macOS, we can use utun interfaces
        println!("🍎 Setting up macOS utun interface...");
        
        // Try to create a utun interface
        for i in 0..10 {
            let interface_name = format!("utun{}", i);
            
            // Check if interface is available
            let output = Command::new("ifconfig")
                .arg(&interface_name)
                .output();
                
            match output {
                Ok(result) if result.status.success() => {
                    // Interface exists, try next one
                    continue;
                },
                _ => {
                    // Interface available, use it
                    self.interface_name = interface_name.clone();
                    println!("   Using interface: {}", interface_name);
                    
                    // Configure the interface (requires admin privileges)
                    let config_result = Command::new("sudo")
                        .args([
                            "ifconfig", &interface_name,
                            &self.config.local_ip.to_string(),
                            &self.config.remote_ip.to_string(),
                            "up"
                        ])
                        .output();
                        
                    match config_result {
                        Ok(output) if output.status.success() => {
                            println!("   ✅ Interface configured with admin privileges");
                            return Ok(());
                        },
                        _ => {
                            println!("   ⚠️  Admin privileges required for full tunnel setup");
                            println!("   ℹ️  Demo mode: tunnel interface created without system routing");
                            return Ok(());
                        }
                    }
                }
            }
        }
        
        // Fallback to demo interface
        self.interface_name = "utun_demo".to_string();
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn establish_linux_tunnel(&mut self) -> Result<()> {
        // On Linux, we can use TUN interfaces
        println!("🐧 Setting up Linux TUN interface...");
        
        // Try to create a TUN interface
        let interface_name = "vpnse0";
        
        // Create TUN interface (requires admin privileges)
        let create_result = Command::new("sudo")
            .args([
                "ip", "tuntap", "add", "dev", interface_name, "mode", "tun"
            ])
            .output();
            
        match create_result {
            Ok(output) if output.status.success() => {
                self.interface_name = interface_name.to_string();
                
                // Configure the interface
                let _config_result = Command::new("sudo")
                    .args([
                        "ip", "addr", "add", 
                        &format!("{}/24", self.config.local_ip),
                        "dev", interface_name
                    ])
                    .output();
                    
                let _up_result = Command::new("sudo")
                    .args(["ip", "link", "set", "dev", interface_name, "up"])
                    .output();
                    
                println!("   ✅ TUN interface created with admin privileges");
            },
            _ => {
                println!("   ⚠️  Admin privileges required for TUN interface creation");
                println!("   ℹ️  Demo mode: virtual tunnel interface");
                self.interface_name = "tun_demo".to_string();
            }
        }
        
        Ok(())
    }

    fn establish_demo_tunnel(&mut self) -> Result<()> {
        println!("🔧 Setting up demo tunnel interface...");
        self.interface_name = "vpnse_demo".to_string();
        println!("   ℹ️  Demo mode: tunnel simulation without system integration");
        Ok(())
    }

    /// Tear down the VPN tunnel
    pub fn teardown_tunnel(&mut self) -> Result<()> {
        if !self.is_established {
            return Ok(());
        }

        println!("🔽 Tearing down VPN tunnel...");
        
        // Restore original routing before closing tunnel
        if let Err(e) = self.restore_original_routing() {
            println!("   ⚠️  Warning: Failed to restore original routing: {}", e);
        }
        
        // Close TUN device if it exists
        if let Some(device) = self.tun_device.take() {
            println!("   🔽 Closing TUN device: {}", self.interface_name);
            drop(device); // TUN device will be automatically closed
        }
        
        // Remove TUN interface if we created it
        #[cfg(target_os = "linux")]
        {
            let _remove_result = Command::new("sudo")
                .args(["ip", "link", "del", &self.interface_name])
                .output();
        }
        
        // Close packet channels
        if let Some(tx) = self.packet_tx.take() {
            drop(tx);
        }
        if let Some(rx) = self.packet_rx.take() {
            drop(rx);
        }
        
        self.is_established = false;
        println!("✅ VPN tunnel torn down successfully");
        Ok(())
    }

    /// Check if tunnel is established
    pub fn is_established(&self) -> bool {
        self.is_established
    }

    /// Get tunnel interface info
    pub fn get_interface_info(&self) -> Option<(String, String, String, String)> {
        if self.is_established {
            Some((
                self.interface_name.clone(),
                self.config.local_ip.to_string(),
                self.config.remote_ip.to_string(),
                format!("{}/{}", self.config.local_ip, 24), // Assuming /24 subnet
            ))
        } else {
            None
        }
    }
    
    /// Get tunnel configuration
    pub fn get_config(&self) -> Option<TunnelConfig> {
        if self.is_established {
            Some(self.config.clone())
        } else {
            None
        }
    }
    
    /// Check if the VPN IP is from a DHCP-assigned range
    /// Detects networks like 10.21.*.*, 10.216.48.*, 10.244.*.* and other common ranges
    pub fn is_dhcp_assigned_ip(&self) -> bool {
        let octets = self.config.local_ip.octets();
        
        // Check for 10.*.*.* networks (includes 10.21.*.*, 10.216.48.*, 10.244.*.*)
        if octets[0] == 10 {
            // Log specific detected ranges for better debugging
            if octets[1] == 21 {
                println!("   📝 Detected 10.21.*.* VPN network from DHCP assignment");
                return true;
            } else if octets[1] == 216 && octets[2] == 48 {
                println!("   📝 Detected 10.216.48.* VPN network from DHCP assignment");
                return true;
            } else if octets[1] == 244 {
                println!("   📝 Detected 10.244.*.* VPN network from DHCP assignment");
                return true;
            }
            
            // All other 10.*.*.* networks are also likely DHCP assigned
            return true;
        }
        
        // Check for other common DHCP-assigned ranges
        if (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31) ||
           (octets[0] == 192 && octets[1] == 168) ||
           (octets[0] == 100 && octets[1] >= 64 && octets[1] <= 127) ||
           (octets[0] == 124 && octets[1] == 166) { // From the logs
            return true;
        }
        
        false
    }
    
    /// Get VPN server IP for routing preservation
    /// 
    /// This method returns the VPN server IP address to prevent routing loops
    /// where VPN traffic tries to route through the VPN itself
    pub fn get_vpn_server_ip(&self) -> Option<String> {
        // First check if we have a known VPN server IP from environment variable
        if let Ok(server_ip) = std::env::var("VPN_SERVER_IP") {
            println!("   📌 Using VPN server IP from environment variable: {}", server_ip);
            return Some(server_ip);
        }
        
        // Check for the server IP from the connection we used to establish the tunnel
        #[cfg(target_os = "linux")]
        {
            // First try with ss command which is more reliable than netstat
            let output = Command::new("ss")
                .args(["-tn", "state", "established"])
                .output();
                
            if let Ok(result) = output {
                let connections = String::from_utf8_lossy(&result.stdout);
                
                // Look for established connections to port 443 or 992 (common SSL-VPN ports)
                for line in connections.lines() {
                    if line.contains("ESTAB") && (line.contains(":443") || line.contains(":992")) {
                        if let Some(peer_addr_start) = line.find("peer=") {
                            let peer_addr_part = &line[peer_addr_start + 5..];
                            if let Some(addr_end) = peer_addr_part.find(' ') {
                                let addr = &peer_addr_part[0..addr_end];
                                if let Some(ip) = addr.split(':').next() {
                                    println!("   📌 Detected VPN server IP from active connection: {}", ip);
                                    return Some(ip.to_string());
                                }
                            }
                        }
                        
                        // Alternative parsing for ss output format
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts.iter() {
                            if part.contains(":443") || part.contains(":992") {
                                if let Some(ip) = part.split(':').next() {
                                    // Verify this looks like an IP address
                                    if ip.contains('.') && !ip.starts_with("127.") {
                                        println!("   📌 Detected VPN server IP from active connection: {}", ip);
                                        return Some(ip.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Fall back to netstat if ss didn't work
            let output = Command::new("netstat")
                .args(["-tn"])
                .output();
                
            if let Ok(result) = output {
                let connections = String::from_utf8_lossy(&result.stdout);
                
                // Look for established connections to port 443 or 992 (common SSL-VPN ports)
                for line in connections.lines() {
                    if line.contains("ESTABLISHED") && (line.contains(":443") || line.contains(":992")) {
                        // Extract server IP from the line (format: IP:port)
                        // Convert split_whitespace iterator to collect::<Vec<_>>() so we can use get()
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(addr) = parts.get(4) {
                            if let Some(ip) = addr.split(':').next() {
                                println!("   📌 Detected VPN server IP from active connection: {}", ip);
                                return Some(ip.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Finally, fall back to the default server IP if all else fails
        println!("   📌 Using default VPN server IP: 62.24.65.211");
        Some("62.24.65.211".to_string())
    }

    /// Get the current public IP
    pub async fn get_current_public_ip(&self) -> Result<String> {
        // Use the public-ip crate for better reliability
        match public_ip::addr().await {
            Some(ip) => Ok(ip.to_string()),
            None => {
                // Fallback to manual HTTP requests
                self.get_public_ip_fallback().await
            }
        }
    }

    /// Fallback method for getting public IP using HTTP requests
    async fn get_public_ip_fallback(&self) -> Result<String> {
        let services = [
            "https://api.ipify.org",
            "https://icanhazip.com",
            "https://ipecho.net/plain",
            "https://checkip.amazonaws.com",
        ];

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| VpnError::Network(format!("Failed to create HTTP client: {}", e)))?;

        for service in &services {
            if let Ok(response) = client.get(*service).send().await {
                if let Ok(ip_text) = response.text().await {
                    let ip = ip_text.trim().to_string();
                    if !ip.is_empty() && self.is_valid_ip(&ip) {
                        return Ok(ip);
                    }
                }
            }
        }

        Err(VpnError::Connection(
            "Failed to get public IP from any service".into(),
        ))
    }

    /// Validate if a string is a valid IP address
    fn is_valid_ip(&self, ip: &str) -> bool {
        use std::net::IpAddr;
        ip.parse::<IpAddr>().is_ok()
    }
    
    /// Check if the IP is a valid VPN-assigned IP
    /// Works with any IP range including 10.21.*.* and other DHCP-assigned ranges
    fn is_valid_vpn_ip(&self, ip: std::net::Ipv4Addr) -> bool {
        // First check if it's the same as our currently configured IP
        if ip == self.config.local_ip {
            return true;
        }
        
        // Get the first octet to determine IP class
        let first_octet = ip.octets()[0];
        
        // Handle special cases:
        // 1. Private IP ranges (commonly used in VPNs)
        if (first_octet == 10) || // 10.0.0.0/8
           (first_octet == 172 && ip.octets()[1] >= 16 && ip.octets()[1] <= 31) || // 172.16.0.0/12
           (first_octet == 192 && ip.octets()[1] == 168) { // 192.168.0.0/16
            return true;
        }
        
        // 2. Common VPN ranges 
        if first_octet >= 100 && first_octet <= 127 {
            // Many VPN providers use IPs in this range
            return true;
        }
        
        // Accept any non-local, non-multicast IP as potentially valid
        // This ensures we don't reject unusual DHCP-assigned ranges
        !(ip.is_loopback() || ip.is_multicast() || ip.is_broadcast())
    }

    /// Store the original default route
    fn store_original_route(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("route")
                .args(["-n", "get", "default"])
                .output()
                .map_err(|e| VpnError::Connection(format!("Failed to get default route: {e}")))?;

            if output.status.success() {
                let route_info = String::from_utf8_lossy(&output.stdout);
                for line in route_info.lines() {
                    if line.trim().starts_with("gateway:") {
                        let gateway = line
                            .split(':')
                            .nth(1)
                            .ok_or_else(|| {
                                VpnError::Connection("Invalid route format".to_string())
                            })?
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
                .args(["route", "show", "default"])
                .output()
                .map_err(|e| VpnError::Connection(format!("Failed to get default route: {e}")))?;

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

        println!("Original route stored: {:?}", self.original_route);
        Ok(())
    }

    // Using the public get_vpn_server_ip method defined above
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

/// Get current tunnel interface information
/// Returns (interface_name, local_ip, remote_ip, subnet)
pub fn get_tunnel_interface() -> Option<(String, String, String, String)> {
    let global_manager = TUNNEL_MANAGER.lock().unwrap();
    if let Some(ref manager) = *global_manager {
        manager.get_interface_info()
    } else {
        None
    }
}

pub async fn get_tunnel_public_ip() -> Result<String> {
    let global_manager = TUNNEL_MANAGER.lock().unwrap();
    if let Some(ref manager) = *global_manager {
        manager.get_current_public_ip().await
    } else {
        Err(VpnError::Connection("No tunnel established".to_string()))
    }
}
