//! Linux TUN Interface Implementation
//! 
//! Provides Linux-specific TUN interface management using the native TUN/TAP driver

use crate::error::{Result, VpnError};
use std::os::unix::io::{AsRawFd, RawFd};
use std::ffi::CString;
use libc::{self, c_int, c_void, c_short, c_char};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{Bytes, BytesMut};
use std::io;
use std::mem;

/// TUN/TAP interface request structure
#[repr(C)]
struct IfReq {
    ifr_name: [c_char; 16],
    ifr_flags: c_short,
}

/// TUN/TAP constants
const IFF_TUN: c_short = 0x0001;
const IFF_TAP: c_short = 0x0002;
const IFF_NO_PI: c_short = 0x1000;
const IFF_MULTI_QUEUE: c_short = 0x0100;
const TUNSETIFF: u64 = 0x400454ca;
const TUNSETPERSIST: u64 = 0x400454cb;
const TUNSETOWNER: u64 = 0x400454cc;
const TUNSETGROUP: u64 = 0x400454ce;

/// Linux TUN interface
pub struct LinuxTunInterface {
    fd: RawFd,
    interface_name: String,
    is_tun: bool, // true for TUN, false for TAP
    is_connected: bool,
    mtu: u32,
}

impl LinuxTunInterface {
    /// Create a new Linux TUN interface
    pub fn new(interface_name: Option<String>, is_tun: bool) -> Result<Self> {
        log::info!("Initializing Linux {} interface", if is_tun { "TUN" } else { "TAP" });
        
        let fd = Self::create_tun_tap_fd()?;
        let actual_name = Self::setup_interface(fd, interface_name, is_tun)?;
        
        log::info!("Created {} interface: {}", if is_tun { "TUN" } else { "TAP" }, actual_name);
        
        Ok(Self {
            fd,
            interface_name: actual_name,
            is_tun,
            is_connected: false,
            mtu: 1500, // Default MTU
        })
    }

    /// Create TUN/TAP file descriptor
    fn create_tun_tap_fd() -> Result<RawFd> {
        let tun_path = CString::new("/dev/net/tun")
            .map_err(|_| VpnError::TunTap("Invalid TUN path".to_string()))?;
        
        unsafe {
            let fd = libc::open(tun_path.as_ptr(), libc::O_RDWR);
            if fd < 0 {
                return Err(VpnError::TunTap("Failed to open /dev/net/tun. Make sure TUN/TAP is available.".to_string()));
            }
            Ok(fd)
        }
    }

    /// Setup TUN/TAP interface
    fn setup_interface(fd: RawFd, name: Option<String>, is_tun: bool) -> Result<String> {
        let mut ifr: IfReq = unsafe { mem::zeroed() };
        
        // Set interface name if provided
        if let Some(ref name) = name {
            if name.len() >= 16 {
                return Err(VpnError::TunTap("Interface name too long".to_string()));
            }
            let name_cstring = CString::new(name.as_str())
                .map_err(|_| VpnError::TunTap("Invalid interface name".to_string()))?;
            unsafe {
                libc::strcpy(ifr.ifr_name.as_mut_ptr(), name_cstring.as_ptr());
            }
        }
        
        // Set interface flags
        ifr.ifr_flags = if is_tun { IFF_TUN } else { IFF_TAP };
        ifr.ifr_flags |= IFF_NO_PI; // No packet info header
        
        // Create interface
        unsafe {
            let result = libc::ioctl(fd, TUNSETIFF, &mut ifr as *mut _ as *mut c_void);
            if result < 0 {
                libc::close(fd);
                return Err(VpnError::TunTap("Failed to create TUN/TAP interface".to_string()));
            }
        }
        
        // Get actual interface name
        let null_pos = ifr.ifr_name.iter().position(|&b| b == 0).unwrap_or(ifr.ifr_name.len());
        let actual_name = unsafe {
            String::from_utf8_lossy(
                std::slice::from_raw_parts(ifr.ifr_name.as_ptr() as *const u8, null_pos)
            ).to_string()
        };
        
        Ok(actual_name)
    }

    /// Configure interface with IP addresses
    pub fn configure(&mut self, local_ip: &str, remote_ip: &str, netmask: &str) -> Result<()> {
        log::info!("Configuring TUN interface: {} -> {} ({})", local_ip, remote_ip, netmask);
        
        // Bring interface up
        let up_cmd = format!("sudo ip link set dev {} up", self.interface_name);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&up_cmd)
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to bring interface up: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(VpnError::TunTap(format!("Failed to bring interface up: {}", error_msg)));
        }
        
        // Configure IP address
        if self.is_tun {
            // For TUN (point-to-point)
            let addr_cmd = format!(
                "sudo ip addr add {} peer {} dev {}",
                local_ip, remote_ip, self.interface_name
            );
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&addr_cmd)
                .output()
                .map_err(|e| VpnError::TunTap(format!("Failed to configure address: {}", e)))?;
            
            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                log::warn!("Address configuration warning: {}", error_msg);
            }
        } else {
            // For TAP (bridge mode)
            let addr_cmd = format!(
                "sudo ip addr add {}/{} dev {}",
                local_ip, Self::netmask_to_cidr(netmask)?, self.interface_name
            );
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&addr_cmd)
                .output()
                .map_err(|e| VpnError::TunTap(format!("Failed to configure address: {}", e)))?;
            
            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                log::warn!("Address configuration warning: {}", error_msg);
            }
        }
        
        self.is_connected = true;
        log::info!("TUN interface configured successfully");
        Ok(())
    }

    /// Convert netmask to CIDR notation
    fn netmask_to_cidr(netmask: &str) -> Result<u8> {
        let addr = netmask.parse::<std::net::Ipv4Addr>()
            .map_err(|_| VpnError::Configuration("Invalid netmask".to_string()))?;
        
        let octets = addr.octets();
        let mask = u32::from_be_bytes(octets);
        let cidr = mask.leading_ones() as u8;
        
        Ok(cidr)
    }

    /// Read packet from TUN interface
    pub async fn read_packet(&mut self) -> Result<Bytes> {
        let mut buffer = vec![0u8; self.mtu as usize];
        
        let bytes_read = unsafe {
            libc::read(self.fd, buffer.as_mut_ptr() as *mut c_void, buffer.len())
        };
        
        if bytes_read < 0 {
            return Err(VpnError::TunTap("Failed to read from TUN interface".to_string()));
        }
        
        buffer.truncate(bytes_read as usize);
        Ok(Bytes::from(buffer))
    }

    /// Write packet to TUN interface
    pub async fn write_packet(&mut self, packet: Bytes) -> Result<()> {
        let bytes_written = unsafe {
            libc::write(
                self.fd,
                packet.as_ptr() as *const c_void,
                packet.len(),
            )
        };
        
        if bytes_written < 0 {
            return Err(VpnError::TunTap("Failed to write to TUN interface".to_string()));
        }
        
        if bytes_written != packet.len() as isize {
            return Err(VpnError::TunTap("Incomplete write to TUN interface".to_string()));
        }
        
        Ok(())
    }

    /// Set interface as persistent
    pub fn set_persistent(&self, persistent: bool) -> Result<()> {
        let value = if persistent { 1 } else { 0 };
        
        unsafe {
            let result = libc::ioctl(self.fd, TUNSETPERSIST, value);
            if result < 0 {
                return Err(VpnError::TunTap("Failed to set persistence".to_string()));
            }
        }
        
        log::info!("Interface persistence set to: {}", persistent);
        Ok(())
    }

    /// Set interface owner
    pub fn set_owner(&self, uid: u32) -> Result<()> {
        unsafe {
            let result = libc::ioctl(self.fd, TUNSETOWNER, uid);
            if result < 0 {
                return Err(VpnError::TunTap("Failed to set owner".to_string()));
            }
        }
        
        log::info!("Interface owner set to UID: {}", uid);
        Ok(())
    }

    /// Set interface group
    pub fn set_group(&self, gid: u32) -> Result<()> {
        unsafe {
            let result = libc::ioctl(self.fd, TUNSETGROUP, gid);
            if result < 0 {
                return Err(VpnError::TunTap("Failed to set group".to_string()));
            }
        }
        
        log::info!("Interface group set to GID: {}", gid);
        Ok(())
    }

    /// Get interface name
    pub fn interface_name(&self) -> &str {
        &self.interface_name
    }

    /// Get MTU
    pub fn mtu(&self) -> u32 {
        self.mtu
    }

    /// Set MTU
    pub fn set_mtu(&mut self, mtu: u32) -> Result<()> {
        let mtu_cmd = format!("sudo ip link set dev {} mtu {}", self.interface_name, mtu);
        
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&mtu_cmd)
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to set MTU: {}", e)))?;
        
        if output.status.success() {
            self.mtu = mtu;
            log::info!("MTU set to {}", mtu);
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            log::warn!("Failed to set MTU: {}", error_msg);
        }
        
        Ok(())
    }

    /// Check if interface is up
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Check if this is a TUN interface (vs TAP)
    pub fn is_tun(&self) -> bool {
        self.is_tun
    }

    /// Get interface statistics
    pub fn get_stats(&self) -> Result<InterfaceStats> {
        let stats_path = format!("/sys/class/net/{}/statistics", self.interface_name);
        
        let rx_bytes = Self::read_stat_file(&format!("{}/rx_bytes", stats_path))?;
        let tx_bytes = Self::read_stat_file(&format!("{}/tx_bytes", stats_path))?;
        let rx_packets = Self::read_stat_file(&format!("{}/rx_packets", stats_path))?;
        let tx_packets = Self::read_stat_file(&format!("{}/tx_packets", stats_path))?;
        let rx_errors = Self::read_stat_file(&format!("{}/rx_errors", stats_path))?;
        let tx_errors = Self::read_stat_file(&format!("{}/tx_errors", stats_path))?;
        
        Ok(InterfaceStats {
            bytes_received: rx_bytes,
            bytes_sent: tx_bytes,
            packets_received: rx_packets,
            packets_sent: tx_packets,
            errors_received: rx_errors,
            errors_sent: tx_errors,
        })
    }

    /// Read a single statistic file
    fn read_stat_file(path: &str) -> Result<u64> {
        match std::fs::read_to_string(path) {
            Ok(content) => content.trim().parse().unwrap_or(0),
            Err(_) => 0,
        }
        .pipe(Ok)
    }
}

impl AsRawFd for LinuxTunInterface {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for LinuxTunInterface {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe {
                libc::close(self.fd);
            }
            log::info!("Linux TUN interface closed: {}", self.interface_name);
        }
    }
}

// Async I/O traits implementation
impl AsyncRead for LinuxTunInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // This would use epoll for real async implementation
        // For now, return pending to avoid blocking
        Poll::Pending
    }
}

impl AsyncWrite for LinuxTunInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // This would use epoll for real async implementation
        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Interface statistics
#[derive(Debug, Default)]
pub struct InterfaceStats {
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub errors_received: u64,
    pub errors_sent: u64,
}

/// Linux-specific TUN utilities
pub mod linux_utils {
    use super::*;
    
    /// Check if running with root privileges
    pub fn is_root() -> bool {
        unsafe { libc::getuid() == 0 }
    }
    
    /// Check if TUN/TAP module is loaded
    pub fn is_tun_available() -> bool {
        std::path::Path::new("/dev/net/tun").exists()
    }
    
    /// Load TUN module if not available
    pub fn load_tun_module() -> Result<()> {
        if Self::is_tun_available() {
            return Ok(());
        }
        
        log::info!("Loading TUN module");
        let output = std::process::Command::new("sudo")
            .arg("modprobe")
            .arg("tun")
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to load TUN module: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(VpnError::TunTap(format!("Failed to load TUN module: {}", error_msg)));
        }
        
        if !Self::is_tun_available() {
            return Err(VpnError::TunTap("TUN module loaded but /dev/net/tun not available".to_string()));
        }
        
        log::info!("TUN module loaded successfully");
        Ok(())
    }
    
    /// List network interfaces
    pub fn list_interfaces() -> Result<Vec<String>> {
        let output = std::process::Command::new("ip")
            .arg("link")
            .arg("show")
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to list interfaces: {}", e)))?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();
        
        for line in output_str.lines() {
            if let Some(start) = line.find(": ") {
                if let Some(end) = line[start + 2..].find(':') {
                    let interface_name = line[start + 2..start + 2 + end].to_string();
                    interfaces.push(interface_name);
                }
            }
        }
        
        Ok(interfaces)
    }
    
    /// Get interface IP addresses
    pub fn get_interface_ips(interface: &str) -> Result<Vec<String>> {
        let output = std::process::Command::new("ip")
            .arg("addr")
            .arg("show")
            .arg("dev")
            .arg(interface)
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to get IPs: {}", e)))?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut ips = Vec::new();
        
        for line in output_str.lines() {
            if line.trim().starts_with("inet ") {
                if let Some(ip_part) = line.trim().split_whitespace().nth(1) {
                    if let Some(ip) = ip_part.split('/').next() {
                        ips.push(ip.to_string());
                    }
                }
            }
        }
        
        Ok(ips)
    }
    
    /// Add route via interface
    pub fn add_route(destination: &str, interface: &str) -> Result<()> {
        let route_cmd = format!("sudo ip route add {} dev {}", destination, interface);
        
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&route_cmd)
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to add route: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(VpnError::TunTap(format!("Failed to add route: {}", error_msg)));
        }
        
        log::info!("Route added: {} via {}", destination, interface);
        Ok(())
    }
    
    /// Delete route via interface
    pub fn delete_route(destination: &str, interface: &str) -> Result<()> {
        let route_cmd = format!("sudo ip route del {} dev {}", destination, interface);
        
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&route_cmd)
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to delete route: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            log::warn!("Failed to delete route: {}", error_msg);
        } else {
            log::info!("Route deleted: {} via {}", destination, interface);
        }
        
        Ok(())
    }
}

/// Extension trait for piping values
trait Pipe {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
        Self: Sized,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_check() {
        let _is_root = linux_utils::is_root();
        // Just ensure the function doesn't panic
    }

    #[test]
    fn test_tun_availability() {
        let _is_available = linux_utils::is_tun_available();
        // Just ensure the function doesn't panic
    }

    #[test]
    fn test_interface_listing() {
        let interfaces = linux_utils::list_interfaces().unwrap_or_default();
        println!("Found interfaces: {:?}", interfaces);
        assert!(!interfaces.is_empty()); // Should at least have loopback
    }

    #[test]
    fn test_netmask_conversion() {
        assert_eq!(LinuxTunInterface::netmask_to_cidr("255.255.255.0").unwrap(), 24);
        assert_eq!(LinuxTunInterface::netmask_to_cidr("255.255.0.0").unwrap(), 16);
        assert_eq!(LinuxTunInterface::netmask_to_cidr("255.0.0.0").unwrap(), 8);
    }
}
