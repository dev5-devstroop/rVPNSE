//! macOS utun Interface Implementation
//! 
//! Provides macOS-specific TUN interface management using the native utun driver

use crate::error::{Result, VpnError};
use std::os::unix::io::{AsRawFd, RawFd};
use std::ffi::CString;
use libc::{
    self, c_int, c_void, sockaddr, sockaddr_ctl, AF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL,
    PF_SYSTEM, SO_RCVBUF, SO_SNDBUF, SOL_SOCKET, CTLIOCGINFO, ctl_info,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{Bytes, BytesMut};
use std::io;
use std::mem;

/// utun control name for macOS
const UTUN_CONTROL_NAME: &str = "com.apple.net.utun_control";
const UTUN_OPT_IFNAME: c_int = 2;

/// macOS utun interface
pub struct MacOSUtunInterface {
    fd: RawFd,
    interface_name: String,
    is_connected: bool,
    mtu: u32,
}

impl MacOSUtunInterface {
    /// Create a new macOS utun interface
    pub fn new() -> Result<Self> {
        log::info!("Initializing macOS utun interface");
        
        let fd = Self::create_utun_socket()?;
        let interface_name = Self::get_interface_name(fd)?;
        
        log::info!("Created utun interface: {}", interface_name);
        
        Ok(Self {
            fd,
            interface_name,
            is_connected: false,
            mtu: 1500, // Default MTU
        })
    }

    /// Create utun socket
    fn create_utun_socket() -> Result<RawFd> {
        unsafe {
            // Create control socket
            let fd = libc::socket(PF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL);
            if fd < 0 {
                return Err(VpnError::TunTap("Failed to create control socket".to_string()));
            }

            // Get control info
            let mut ctl_info: ctl_info = mem::zeroed();
            let control_name = CString::new(UTUN_CONTROL_NAME)
                .map_err(|_| VpnError::TunTap("Invalid control name".to_string()))?;
            
            libc::strcpy(ctl_info.ctl_name.as_mut_ptr(), control_name.as_ptr());
            
            let result = libc::ioctl(fd, CTLIOCGINFO, &mut ctl_info as *mut _ as *mut c_void);
            if result < 0 {
                libc::close(fd);
                return Err(VpnError::TunTap("Failed to get control info".to_string()));
            }

            // Connect to utun control
            let mut addr: sockaddr_ctl = mem::zeroed();
            addr.sc_len = mem::size_of::<sockaddr_ctl>() as u8;
            addr.sc_family = AF_SYSTEM as u8;
            addr.ss_sysaddr = AF_SYS_CONTROL;
            addr.sc_id = ctl_info.ctl_id;
            addr.sc_unit = 0; // Let kernel assign unit number

            let result = libc::connect(
                fd,
                &addr as *const _ as *const sockaddr,
                mem::size_of::<sockaddr_ctl>() as u32,
            );

            if result < 0 {
                libc::close(fd);
                return Err(VpnError::TunTap("Failed to connect to utun control".to_string()));
            }

            // Set socket buffer sizes
            let buffer_size = 65536i32;
            libc::setsockopt(
                fd,
                SOL_SOCKET,
                SO_RCVBUF,
                &buffer_size as *const _ as *const c_void,
                mem::size_of::<i32>() as u32,
            );
            libc::setsockopt(
                fd,
                SOL_SOCKET,
                SO_SNDBUF,
                &buffer_size as *const _ as *const c_void,
                mem::size_of::<i32>() as u32,
            );

            Ok(fd)
        }
    }

    /// Get interface name
    fn get_interface_name(fd: RawFd) -> Result<String> {
        let mut ifname = [0u8; 64];
        let mut ifname_len = ifname.len() as u32;

        unsafe {
            let result = libc::getsockopt(
                fd,
                SYSPROTO_CONTROL,
                UTUN_OPT_IFNAME,
                ifname.as_mut_ptr() as *mut c_void,
                &mut ifname_len,
            );

            if result < 0 {
                return Err(VpnError::TunTap("Failed to get interface name".to_string()));
            }
        }

        let null_pos = ifname.iter().position(|&b| b == 0).unwrap_or(ifname.len());
        let name = String::from_utf8_lossy(&ifname[..null_pos]).to_string();
        
        Ok(name)
    }

    /// Configure interface with IP addresses
    pub fn configure(&mut self, local_ip: &str, remote_ip: &str, netmask: &str) -> Result<()> {
        log::info!("Configuring utun interface: {} -> {} ({})", local_ip, remote_ip, netmask);
        
        // Use ifconfig to configure the interface
        let configure_cmd = format!(
            "sudo ifconfig {} {} {} up",
            self.interface_name, local_ip, remote_ip
        );
        
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&configure_cmd)
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to run ifconfig: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(VpnError::TunTap(format!("ifconfig failed: {}", error_msg)));
        }
        
        // Add route if needed
        let route_cmd = format!(
            "sudo route add -net {} {} {}",
            remote_ip, netmask, local_ip
        );
        
        let _route_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&route_cmd)
            .output(); // Ignore errors for route command
        
        self.is_connected = true;
        log::info!("utun interface configured successfully");
        Ok(())
    }

    /// Read packet from utun interface
    pub async fn read_packet(&mut self) -> Result<Bytes> {
        let mut buffer = vec![0u8; self.mtu as usize + 4]; // +4 for protocol family
        
        let bytes_read = unsafe {
            libc::read(self.fd, buffer.as_mut_ptr() as *mut c_void, buffer.len())
        };
        
        if bytes_read < 0 {
            return Err(VpnError::TunTap("Failed to read from utun".to_string()));
        }
        
        if bytes_read < 4 {
            return Err(VpnError::TunTap("Packet too short".to_string()));
        }
        
        // Skip the 4-byte protocol family header
        buffer.drain(0..4);
        buffer.truncate(bytes_read as usize - 4);
        
        Ok(Bytes::from(buffer))
    }

    /// Write packet to utun interface
    pub async fn write_packet(&mut self, mut packet: Bytes) -> Result<()> {
        // Prepend 4-byte protocol family (AF_INET = 2)
        let mut full_packet = BytesMut::with_capacity(packet.len() + 4);
        full_packet.extend_from_slice(&[0, 0, 0, 2]); // AF_INET in network byte order
        full_packet.extend_from_slice(&packet);
        
        let bytes_written = unsafe {
            libc::write(
                self.fd,
                full_packet.as_ptr() as *const c_void,
                full_packet.len(),
            )
        };
        
        if bytes_written < 0 {
            return Err(VpnError::TunTap("Failed to write to utun".to_string()));
        }
        
        if bytes_written != full_packet.len() as isize {
            return Err(VpnError::TunTap("Incomplete write to utun".to_string()));
        }
        
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
        // Use ifconfig to set MTU
        let mtu_cmd = format!("sudo ifconfig {} mtu {}", self.interface_name, mtu);
        
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

    /// Get interface statistics
    pub fn get_stats(&self) -> Result<InterfaceStats> {
        let netstat_cmd = format!("netstat -I {} -b", self.interface_name);
        
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&netstat_cmd)
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to get stats: {}", e)))?;
        
        if !output.status.success() {
            return Ok(InterfaceStats::default());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        Self::parse_netstat_output(&output_str)
    }

    /// Parse netstat output to extract statistics
    fn parse_netstat_output(output: &str) -> Result<InterfaceStats> {
        // Parse netstat output (simplified)
        // Real implementation would parse the actual columns
        Ok(InterfaceStats {
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
            errors_received: 0,
            errors_sent: 0,
        })
    }
}

impl AsRawFd for MacOSUtunInterface {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for MacOSUtunInterface {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe {
                libc::close(self.fd);
            }
            log::info!("macOS utun interface closed: {}", self.interface_name);
        }
    }
}

// Async I/O traits implementation
impl AsyncRead for MacOSUtunInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // This would use kqueue or similar for real async implementation
        // For now, return pending to avoid blocking
        Poll::Pending
    }
}

impl AsyncWrite for MacOSUtunInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // This would use kqueue or similar for real async implementation
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

/// macOS-specific constants (should be defined in libc but may be missing)
const AF_SYS_CONTROL: u8 = 2;

/// macOS-specific TUN utilities
pub mod macos_utils {
    use super::*;
    
    /// Check if running with root privileges
    pub fn is_root() -> bool {
        unsafe { libc::getuid() == 0 }
    }
    
    /// List available utun interfaces
    pub fn list_utun_interfaces() -> Result<Vec<String>> {
        let output = std::process::Command::new("ifconfig")
            .arg("-l")
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to list interfaces: {}", e)))?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let interfaces: Vec<String> = output_str
            .split_whitespace()
            .filter(|name| name.starts_with("utun"))
            .map(|name| name.to_string())
            .collect();
        
        Ok(interfaces)
    }
    
    /// Get system routing table
    pub fn get_routing_table() -> Result<Vec<Route>> {
        let output = std::process::Command::new("netstat")
            .arg("-rn")
            .arg("-f")
            .arg("inet")
            .output()
            .map_err(|e| VpnError::TunTap(format!("Failed to get routes: {}", e)))?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        Self::parse_routing_table(&output_str)
    }
    
    /// Parse routing table output
    fn parse_routing_table(output: &str) -> Result<Vec<Route>> {
        let mut routes = Vec::new();
        
        for line in output.lines().skip(4) { // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                routes.push(Route {
                    destination: parts[0].to_string(),
                    gateway: parts[1].to_string(),
                    interface: parts[5].to_string(),
                });
            }
        }
        
        Ok(routes)
    }
}

/// Routing table entry
#[derive(Debug)]
pub struct Route {
    pub destination: String,
    pub gateway: String,
    pub interface: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_check() {
        let _is_root = macos_utils::is_root();
        // Just ensure the function doesn't panic
    }

    #[test]
    fn test_interface_listing() {
        let interfaces = macos_utils::list_utun_interfaces().unwrap_or_default();
        // May be empty if no utun interfaces exist
        println!("Found utun interfaces: {:?}", interfaces);
    }

    #[test]
    fn test_routing_table() {
        let routes = macos_utils::get_routing_table().unwrap_or_default();
        // May be empty in test environment
        println!("Found {} routes", routes.len());
    }
}
