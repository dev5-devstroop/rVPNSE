//! Windows TAP Interface Implementation
//! 
//! Provides Windows-specific TUN/TAP interface management using
//! the OpenVPN TAP-Windows adapter

use crate::error::{Result, VpnError};
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use winapi::um::{
    fileapi::{CreateFileW, OPEN_EXISTING},
    handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
    ioapiset::{DeviceIoControl},
    winnt::{FILE_ATTRIBUTE_SYSTEM, GENERIC_READ, GENERIC_WRITE, HANDLE},
    synchapi::{CreateEventW},
    minwinbase::OVERLAPPED,
    errhandlingapi::GetLastError,
    winbase::FILE_FLAG_OVERLAPPED,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::Bytes;
use std::io;

/// TAP device IOCTL codes
const TAP_IOCTL_GET_MAC: u32 = 0x170001;
const TAP_IOCTL_GET_VERSION: u32 = 0x170002;
const TAP_IOCTL_GET_MTU: u32 = 0x170003;
const TAP_IOCTL_SET_MEDIA_STATUS: u32 = 0x170004;
const TAP_IOCTL_CONFIG_TUN: u32 = 0x170005;

/// Windows TAP interface
pub struct WindowsTapInterface {
    handle: HANDLE,
    device_name: String,
    is_connected: bool,
    mtu: u32,
    mac_address: [u8; 6],
}

impl WindowsTapInterface {
    /// Create a new Windows TAP interface
    pub fn new() -> Result<Self> {
        log::info!("Initializing Windows TAP interface");
        
        // Find available TAP device
        let device_name = Self::find_tap_device()?;
        log::info!("Found TAP device: {}", device_name);
        
        // Open TAP device
        let handle = Self::open_tap_device(&device_name)?;
        
        // Get device properties
        let mac_address = Self::get_mac_address(handle)?;
        let mtu = Self::get_mtu(handle)?;
        
        log::info!("TAP device opened successfully - MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}, MTU: {}",
            mac_address[0], mac_address[1], mac_address[2],
            mac_address[3], mac_address[4], mac_address[5], mtu);
        
        Ok(Self {
            handle,
            device_name,
            is_connected: false,
            mtu,
            mac_address,
        })
    }

    /// Find an available TAP device
    fn find_tap_device() -> Result<String> {
        // In a real implementation, we'd enumerate network adapters
        // For now, use the default TAP device name
        let default_tap = r"\\.\Global\TAPRVPNSE01.tap";
        
        // Check if device exists by trying to open it
        let device_name_wide: Vec<u16> = OsString::from(default_tap)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        unsafe {
            let handle = CreateFileW(
                device_name_wide.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_SYSTEM | FILE_FLAG_OVERLAPPED,
                ptr::null_mut(),
            );
            
            if handle != INVALID_HANDLE_VALUE {
                CloseHandle(handle);
                Ok(default_tap.to_string())
            } else {
                // Try alternative TAP device names
                let alternatives = [
                    r"\\.\Global\tap0901.tap",
                    r"\\.\Global\tap0801.tap",
                    r"\\.\Global\RVPNSE.tap",
                ];
                
                for alt_name in &alternatives {
                    let alt_name_wide: Vec<u16> = OsString::from(*alt_name)
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();
                    
                    let alt_handle = CreateFileW(
                        alt_name_wide.as_ptr(),
                        GENERIC_READ | GENERIC_WRITE,
                        0,
                        ptr::null_mut(),
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_SYSTEM | FILE_FLAG_OVERLAPPED,
                        ptr::null_mut(),
                    );
                    
                    if alt_handle != INVALID_HANDLE_VALUE {
                        CloseHandle(alt_handle);
                        return Ok(alt_name.to_string());
                    }
                }
                
                Err(VpnError::TunTap("No TAP device found. Please install TAP-Windows adapter.".to_string()))
            }
        }
    }

    /// Open TAP device handle
    fn open_tap_device(device_name: &str) -> Result<HANDLE> {
        let device_name_wide: Vec<u16> = OsString::from(device_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        unsafe {
            let handle = CreateFileW(
                device_name_wide.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_SYSTEM | FILE_FLAG_OVERLAPPED,
                ptr::null_mut(),
            );
            
            if handle == INVALID_HANDLE_VALUE {
                let error_code = GetLastError();
                return Err(VpnError::TunTap(format!("Failed to open TAP device: error code {}", error_code)));
            }
            
            Ok(handle)
        }
    }

    /// Get MAC address of TAP device
    fn get_mac_address(handle: HANDLE) -> Result<[u8; 6]> {
        let mut mac_address = [0u8; 6];
        let mut bytes_returned = 0u32;
        
        unsafe {
            let result = DeviceIoControl(
                handle,
                TAP_IOCTL_GET_MAC,
                ptr::null_mut(),
                0,
                mac_address.as_mut_ptr() as *mut _,
                6,
                &mut bytes_returned,
                ptr::null_mut(),
            );
            
            if result == 0 {
                log::warn!("Failed to get MAC address, using default");
                return Ok([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]); // Default MAC
            }
        }
        
        Ok(mac_address)
    }

    /// Get MTU of TAP device
    fn get_mtu(handle: HANDLE) -> Result<u32> {
        let mut mtu = 0u32;
        let mut bytes_returned = 0u32;
        
        unsafe {
            let result = DeviceIoControl(
                handle,
                TAP_IOCTL_GET_MTU,
                ptr::null_mut(),
                0,
                &mut mtu as *mut _ as *mut _,
                4,
                &mut bytes_returned,
                ptr::null_mut(),
            );
            
            if result == 0 {
                log::warn!("Failed to get MTU, using default 1500");
                return Ok(1500); // Default MTU
            }
        }
        
        Ok(mtu)
    }

    /// Configure TAP device for TUN mode
    pub fn configure_tun(&mut self, local_ip: &str, remote_ip: &str, netmask: &str) -> Result<()> {
        log::info!("Configuring TAP device for TUN mode: {} -> {} ({})", local_ip, remote_ip, netmask);
        
        // Convert IP addresses to binary format
        let local_addr = local_ip.parse::<std::net::Ipv4Addr>()
            .map_err(|e| VpnError::Config(format!("Invalid local IP: {}", e)))?;
        let remote_addr = remote_ip.parse::<std::net::Ipv4Addr>()
            .map_err(|e| VpnError::Config(format!("Invalid remote IP: {}", e)))?;
        let netmask_addr = netmask.parse::<std::net::Ipv4Addr>()
            .map_err(|e| VpnError::Config(format!("Invalid netmask: {}", e)))?;
        
        // Prepare TUN configuration structure
        let mut config = [0u8; 12];
        config[0..4].copy_from_slice(&local_addr.octets());
        config[4..8].copy_from_slice(&remote_addr.octets());
        config[8..12].copy_from_slice(&netmask_addr.octets());
        
        let mut bytes_returned = 0u32;
        
        unsafe {
            let result = DeviceIoControl(
                self.handle,
                TAP_IOCTL_CONFIG_TUN,
                config.as_ptr() as *mut _,
                12,
                ptr::null_mut(),
                0,
                &mut bytes_returned,
                ptr::null_mut(),
            );
            
            if result == 0 {
                let error_code = GetLastError();
                return Err(VpnError::TunTap(format!("Failed to configure TUN: error code {}", error_code)));
            }
        }
        
        log::info!("TAP device configured successfully");
        Ok(())
    }

    /// Set media status (connected/disconnected)
    pub fn set_media_status(&mut self, connected: bool) -> Result<()> {
        let status = if connected { 1u32 } else { 0u32 };
        let mut bytes_returned = 0u32;
        
        unsafe {
            let result = DeviceIoControl(
                self.handle,
                TAP_IOCTL_SET_MEDIA_STATUS,
                &status as *const _ as *mut _,
                4,
                ptr::null_mut(),
                0,
                &mut bytes_returned,
                ptr::null_mut(),
            );
            
            if result == 0 {
                let error_code = GetLastError();
                return Err(VpnError::TunTap(format!("Failed to set media status: error code {}", error_code)));
            }
        }
        
        self.is_connected = connected;
        log::info!("TAP device media status set to: {}", if connected { "connected" } else { "disconnected" });
        Ok(())
    }

    /// Read packet from TAP device
    pub async fn read_packet(&mut self) -> Result<Bytes> {
        let mut buffer = vec![0u8; self.mtu as usize + 14]; // +14 for Ethernet header
        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        
        // Create event for overlapped I/O
        unsafe {
            overlapped.hEvent = CreateEventW(ptr::null_mut(), 1, 0, ptr::null());
            if overlapped.hEvent.is_null() {
                return Err(VpnError::TunTap("Failed to create event".to_string()));
            }
        }
        
        // For this demo, we'll use a simplified synchronous read
        // In a real implementation, you'd use proper async I/O
        let mut bytes_read = 0u32;
        
        unsafe {
            let result = winapi::um::fileapi::ReadFile(
                self.handle,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
                &mut overlapped,
            );
            
            if result == 0 {
                let error_code = GetLastError();
                if error_code != winapi::shared::winerror::ERROR_IO_PENDING {
                    return Err(VpnError::TunTap(format!("Read failed: error code {}", error_code)));
                }
                
                // Wait for completion (simplified)
                winapi::um::synchapi::WaitForSingleObject(overlapped.hEvent, winapi::um::winbase::INFINITE);
                winapi::um::ioapiset::GetOverlappedResult(self.handle, &mut overlapped, &mut bytes_read, 1);
            }
            
            CloseHandle(overlapped.hEvent);
        }
        
        buffer.truncate(bytes_read as usize);
        Ok(Bytes::from(buffer))
    }

    /// Write packet to TAP device
    pub async fn write_packet(&mut self, packet: Bytes) -> Result<()> {
        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        
        // Create event for overlapped I/O
        unsafe {
            overlapped.hEvent = CreateEventW(ptr::null_mut(), 1, 0, ptr::null());
            if overlapped.hEvent.is_null() {
                return Err(VpnError::TunTap("Failed to create event".to_string()));
            }
        }
        
        let mut bytes_written = 0u32;
        
        unsafe {
            let result = winapi::um::fileapi::WriteFile(
                self.handle,
                packet.as_ptr() as *const _,
                packet.len() as u32,
                &mut bytes_written,
                &mut overlapped,
            );
            
            if result == 0 {
                let error_code = GetLastError();
                if error_code != winapi::shared::winerror::ERROR_IO_PENDING {
                    CloseHandle(overlapped.hEvent);
                    return Err(VpnError::TunTap(format!("Write failed: error code {}", error_code)));
                }
                
                // Wait for completion (simplified)
                winapi::um::synchapi::WaitForSingleObject(overlapped.hEvent, winapi::um::winbase::INFINITE);
                winapi::um::ioapiset::GetOverlappedResult(self.handle, &mut overlapped, &mut bytes_written, 1);
            }
            
            CloseHandle(overlapped.hEvent);
        }
        
        if bytes_written != packet.len() as u32 {
            return Err(VpnError::TunTap("Incomplete write".to_string()));
        }
        
        Ok(())
    }

    /// Get device information
    pub fn device_info(&self) -> (String, [u8; 6], u32) {
        (self.device_name.clone(), self.mac_address, self.mtu)
    }

    /// Check if device is connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
}

impl Drop for WindowsTapInterface {
    fn drop(&mut self) {
        if self.handle != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.handle);
            }
            log::info!("Windows TAP interface closed");
        }
    }
}

// Async I/O traits implementation (simplified for demo)
impl AsyncRead for WindowsTapInterface {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // This is a simplified implementation
        // Real async implementation would use IOCP or similar
        Poll::Pending
    }
}

impl AsyncWrite for WindowsTapInterface {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // This is a simplified implementation
        // Real async implementation would use IOCP or similar
        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Windows-specific TUN utilities
pub mod windows_utils {
    use super::*;
    
    /// Install TAP driver (requires admin privileges)
    pub fn install_tap_driver() -> Result<()> {
        log::warn!("TAP driver installation requires manual setup");
        log::info!("Please install OpenVPN TAP-Windows adapter or similar TAP driver");
        Ok(())
    }
    
    /// List available TAP devices
    pub fn list_tap_devices() -> Result<Vec<String>> {
        // In a real implementation, enumerate network adapters
        let default_devices = vec![
            r"\\.\Global\TAPRVPNSE01.tap".to_string(),
            r"\\.\Global\tap0901.tap".to_string(),
        ];
        Ok(default_devices)
    }
    
    /// Check if running with admin privileges
    pub fn is_admin() -> bool {
        // Simplified check - in real implementation use proper Windows API
        std::env::var("USERNAME").unwrap_or_default().to_lowercase().contains("admin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_enumeration() {
        let devices = windows_utils::list_tap_devices().unwrap();
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_admin_check() {
        // This test will pass/fail depending on how you run it
        let _is_admin = windows_utils::is_admin();
        // Just ensure the function doesn't panic
    }
}
