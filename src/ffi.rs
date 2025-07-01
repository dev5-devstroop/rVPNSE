//! C FFI Interface for Rust VPNSE Static Library
//!
//! This module provides C-compatible functions for integrating Rust VPNSE
//! into applications written in other languages (Swift, Kotlin, C#, etc.).

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use crate::{Config, VpnClient, VpnError};

/// Error codes returned by C FFI functions
#[repr(C)]
pub enum VPNSEError {
    Success = 0,
    InvalidConfig = 1,
    ConnectionFailed = 2,
    AuthenticationFailed = 3,
    NetworkError = 4,
    InvalidParameter = 5,
    TunnelError = 6,
    BufferTooSmall = 7,
    InternalError = 99,
}

impl From<VpnError> for VPNSEError {
    fn from(error: VpnError) -> Self {
        match error {
            VpnError::Config(_) => VPNSEError::InvalidConfig,
            VpnError::Connection(_) => VPNSEError::ConnectionFailed,
            VpnError::Authentication(_) => VPNSEError::AuthenticationFailed,
            VpnError::Network(_) => VPNSEError::NetworkError,
            VpnError::TunTap(_) => VPNSEError::TunnelError,
            VpnError::Routing(_) => VPNSEError::TunnelError,
            _ => VPNSEError::InternalError,
        }
    }
}

/// Parse and validate a SoftEther VPN configuration
///
/// # Parameters
/// - `config_str`: TOML configuration string
/// - `error_msg`: Output buffer for error messages (nullable)
/// - `error_msg_len`: Size of error message buffer
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_parse_config(
    config_str: *const c_char,
    error_msg: *mut c_char,
    error_msg_len: usize,
) -> c_int {
    if config_str.is_null() {
        return VPNSEError::InvalidParameter as c_int;
    }

    let config_str = match CStr::from_ptr(config_str).to_str() {
        Ok(s) => s,
        Err(_) => return VPNSEError::InvalidParameter as c_int,
    };

    match config_str.parse::<Config>() {
        Ok(_) => VPNSEError::Success as c_int,
        Err(err) => {
            if !error_msg.is_null() && error_msg_len > 0 {
                let error_str = format!("{err}");
                let error_cstr = CString::new(error_str).unwrap_or_default();
                let error_bytes = error_cstr.as_bytes_with_nul();
                let copy_len = std::cmp::min(error_bytes.len(), error_msg_len - 1);

                ptr::copy_nonoverlapping(
                    error_bytes.as_ptr() as *const c_char,
                    error_msg,
                    copy_len,
                );
                *error_msg.add(copy_len) = 0; // Null terminate
            }
            VPNSEError::from(err) as c_int
        }
    }
}

/// Create a new VPN client instance
///
/// # Parameters
/// - `config_str`: TOML configuration string
///
/// # Returns
/// - Opaque pointer to VPN client on success
/// - NULL on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_client_new(config_str: *const c_char) -> *mut VpnClient {
    if config_str.is_null() {
        return ptr::null_mut();
    }

    let config_str = match CStr::from_ptr(config_str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let config = match config_str.parse::<Config>() {
        Ok(config) => config,
        Err(_) => return ptr::null_mut(),
    };

    match VpnClient::new(config) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(_) => ptr::null_mut(),
    }
}

/// Connect to SoftEther VPN server
///
/// # Parameters
/// - `client`: VPN client instance from vpnse_client_new
/// - `server`: Server hostname or IP address
/// - `port`: Server port number
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_client_connect(
    client: *mut VpnClient,
    server: *const c_char,
    port: u16,
) -> c_int {
    if client.is_null() || server.is_null() {
        return VPNSEError::InvalidParameter as c_int;
    }

    let client = &mut *client;
    let server_str = match CStr::from_ptr(server).to_str() {
        Ok(s) => s,
        Err(_) => return VPNSEError::InvalidParameter as c_int,
    };

    match client.connect(server_str, port) {
        Ok(_) => VPNSEError::Success as c_int,
        Err(err) => VPNSEError::from(err) as c_int,
    }
}

/// Authenticate with SoftEther VPN server
///
/// # Parameters
/// - `client`: VPN client instance
/// - `username`: Username for authentication
/// - `password`: Password for authentication
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_client_authenticate(
    client: *mut VpnClient,
    username: *const c_char,
    password: *const c_char,
) -> c_int {
    if client.is_null() || username.is_null() || password.is_null() {
        return VPNSEError::InvalidParameter as c_int;
    }

    let client = &mut *client;
    let username_str = match CStr::from_ptr(username).to_str() {
        Ok(s) => s,
        Err(_) => return VPNSEError::InvalidParameter as c_int,
    };
    let password_str = match CStr::from_ptr(password).to_str() {
        Ok(s) => s,
        Err(_) => return VPNSEError::InvalidParameter as c_int,
    };

    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(client.authenticate(username_str, password_str))
    {
        Ok(_) => VPNSEError::Success as c_int,
        Err(err) => VPNSEError::from(err) as c_int,
    }
}

/// Disconnect from VPN server
///
/// # Parameters
/// - `client`: VPN client instance
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_client_disconnect(client: *mut VpnClient) -> c_int {
    if client.is_null() {
        return VPNSEError::InvalidParameter as c_int;
    }

    let client = &mut *client;
    match client.disconnect() {
        Ok(_) => VPNSEError::Success as c_int,
        Err(err) => VPNSEError::from(err) as c_int,
    }
}

/// Free VPN client instance
///
/// # Parameters
/// - `client`: VPN client instance to free
#[no_mangle]
pub unsafe extern "C" fn vpnse_client_free(client: *mut VpnClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}

/// Get library version
///
/// # Returns
/// - Version string (caller must not free)
#[no_mangle]
pub unsafe extern "C" fn vpnse_version() -> *const c_char {
    static VERSION_CSTR: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION_CSTR.as_ptr() as *const c_char
}

/// Get connection status
///
/// # Parameters
/// - `client`: VPN client instance
///
/// # Returns
/// - 0: Disconnected
/// - 1: Connecting
/// - 2: Connected (Protocol only)
/// - 3: Tunnel established
/// - -1: Error or invalid client
#[no_mangle]
pub unsafe extern "C" fn vpnse_client_status(client: *const VpnClient) -> c_int {
    if client.is_null() {
        return -1;
    }

    let client = &*client;
    match client.status() {
        crate::ConnectionStatus::Disconnected => 0,
        crate::ConnectionStatus::Connecting => 1,
        crate::ConnectionStatus::Connected => 2,
        crate::ConnectionStatus::Tunneling => 3,
    }
}

/// Establish VPN tunnel (routing layer)
///
/// This function attempts to create a TUN interface and configure routing
/// to route traffic through the VPN tunnel.
///
/// # Parameters
/// - `client`: VPN client instance (must be authenticated)
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_client_establish_tunnel(client: *mut VpnClient) -> c_int {
    if client.is_null() {
        return VPNSEError::InvalidParameter as c_int;
    }

    let client = &mut *client;
    match client.establish_tunnel() {
        Ok(_) => VPNSEError::Success as c_int,
        Err(err) => VPNSEError::from(err) as c_int,
    }
}

/// Establish a VPN tunnel
///
/// # Parameters
/// - `client`: VPN client instance
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_tunnel_establish(client: *mut VpnClient) -> c_int {
    if client.is_null() {
        return VPNSEError::InvalidParameter as c_int;
    }

    let client = &mut *client;
    match client.establish_tunnel() {
        Ok(_) => VPNSEError::Success as c_int,
        Err(err) => VPNSEError::from(err) as c_int,
    }
}

/// Close the VPN tunnel
///
/// # Parameters
/// - `client`: VPN client instance
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_tunnel_close(client: *mut VpnClient) -> c_int {
    if client.is_null() {
        return VPNSEError::InvalidParameter as c_int;
    }

    let client = &mut *client;
    match client.teardown_tunnel() {
        Ok(_) => VPNSEError::Success as c_int,
        Err(err) => VPNSEError::from(err) as c_int,
    }
}

/// Get current public IP address (for testing if traffic is routed through VPN)
///
/// # Parameters
/// - `client`: VPN client instance
/// - `ip_buffer`: Buffer to store the IP address string
/// - `buffer_len`: Size of the buffer
///
/// # Returns
/// - 0 on success
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_get_public_ip(
    client: *mut VpnClient,
    ip_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if client.is_null() || ip_buffer.is_null() || buffer_len == 0 {
        return VPNSEError::InvalidParameter as c_int;
    }

    let client = &mut *client;
    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(client.get_current_public_ip())
    {
        Ok(ip) => {
            let ip_cstr = match CString::new(ip) {
                Ok(s) => s,
                Err(_) => return VPNSEError::InvalidParameter as c_int,
            };

            let ip_bytes = ip_cstr.as_bytes_with_nul();
            if ip_bytes.len() > buffer_len {
                return VPNSEError::BufferTooSmall as c_int;
            }

            unsafe {
                ptr::copy_nonoverlapping(
                    ip_bytes.as_ptr() as *const c_char,
                    ip_buffer,
                    ip_bytes.len(),
                );
            }

            VPNSEError::Success as c_int
        }
        Err(err) => VPNSEError::from(err) as c_int,
    }
}

/// Get tunnel interface name
///
/// # Parameters
/// - `client`: VPN client instance
/// - `interface_buffer`: Buffer to store the interface name
/// - `buffer_len`: Size of the buffer
///
/// # Returns
/// - 0 on success, 1 if no tunnel established
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_get_tunnel_interface(
    client: *mut VpnClient,
    interface_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if client.is_null() || interface_buffer.is_null() || buffer_len == 0 {
        return VPNSEError::InvalidParameter as c_int;
    }

    if let Some((interface_name, _, _, _)) = crate::tunnel::get_tunnel_interface() {
        let interface_cstr = match CString::new(interface_name) {
            Ok(s) => s,
            Err(_) => return VPNSEError::InvalidParameter as c_int,
        };

        let interface_bytes = interface_cstr.as_bytes_with_nul();
        if interface_bytes.len() > buffer_len {
            return VPNSEError::BufferTooSmall as c_int;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                interface_bytes.as_ptr() as *const c_char,
                interface_buffer,
                interface_bytes.len(),
            );
        }

        VPNSEError::Success as c_int
    } else {
        1 // No tunnel established
    }
}

/// Get tunnel local IP address
///
/// # Parameters
/// - `client`: VPN client instance
/// - `ip_buffer`: Buffer to store the local IP address
/// - `buffer_len`: Size of the buffer
///
/// # Returns
/// - 0 on success, 1 if no tunnel established
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_get_tunnel_local_ip(
    client: *mut VpnClient,
    ip_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if client.is_null() || ip_buffer.is_null() || buffer_len == 0 {
        return VPNSEError::InvalidParameter as c_int;
    }

    if let Some((_, local_ip, _, _)) = crate::tunnel::get_tunnel_interface() {
        let ip_cstr = match CString::new(local_ip) {
            Ok(s) => s,
            Err(_) => return VPNSEError::InvalidParameter as c_int,
        };

        let ip_bytes = ip_cstr.as_bytes_with_nul();
        if ip_bytes.len() > buffer_len {
            return VPNSEError::BufferTooSmall as c_int;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                ip_bytes.as_ptr() as *const c_char,
                ip_buffer,
                ip_bytes.len(),
            );
        }

        VPNSEError::Success as c_int
    } else {
        1 // No tunnel established
    }
}

/// Get tunnel remote IP address (gateway)
///
/// # Parameters
/// - `client`: VPN client instance
/// - `ip_buffer`: Buffer to store the remote IP address
/// - `buffer_len`: Size of the buffer
///
/// # Returns
/// - 0 on success, 1 if no tunnel established
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_get_tunnel_remote_ip(
    client: *mut VpnClient,
    ip_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if client.is_null() || ip_buffer.is_null() || buffer_len == 0 {
        return VPNSEError::InvalidParameter as c_int;
    }

    if let Some((_, _, remote_ip, _)) = crate::tunnel::get_tunnel_interface() {
        let ip_cstr = match CString::new(remote_ip) {
            Ok(s) => s,
            Err(_) => return VPNSEError::InvalidParameter as c_int,
        };

        let ip_bytes = ip_cstr.as_bytes_with_nul();
        if ip_bytes.len() > buffer_len {
            return VPNSEError::BufferTooSmall as c_int;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                ip_bytes.as_ptr() as *const c_char,
                ip_buffer,
                ip_bytes.len(),
            );
        }

        VPNSEError::Success as c_int
    } else {
        1 // No tunnel established
    }
}

/// Get tunnel subnet information
///
/// # Parameters
/// - `client`: VPN client instance
/// - `subnet_buffer`: Buffer to store the subnet (e.g., "10.0.0.2/24")
/// - `buffer_len`: Size of the buffer
///
/// # Returns
/// - 0 on success, 1 if no tunnel established
/// - Error code on failure
#[no_mangle]
pub unsafe extern "C" fn vpnse_get_tunnel_subnet(
    client: *mut VpnClient,
    subnet_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if client.is_null() || subnet_buffer.is_null() || buffer_len == 0 {
        return VPNSEError::InvalidParameter as c_int;
    }

    if let Some((_, _, _, subnet)) = crate::tunnel::get_tunnel_interface() {
        let subnet_cstr = match CString::new(subnet) {
            Ok(s) => s,
            Err(_) => return VPNSEError::InvalidParameter as c_int,
        };

        let subnet_bytes = subnet_cstr.as_bytes_with_nul();
        if subnet_bytes.len() > buffer_len {
            return VPNSEError::BufferTooSmall as c_int;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                subnet_bytes.as_ptr() as *const c_char,
                subnet_buffer,
                subnet_bytes.len(),
            );
        }

        VPNSEError::Success as c_int
    } else {
        1 // No tunnel established
    }
}
