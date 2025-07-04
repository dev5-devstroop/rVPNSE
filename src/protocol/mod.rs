//! `SoftEther` SSL-VPN protocol implementation for static library

use crate::error::Result;
use std::net::SocketAddr;

pub mod auth;
pub mod binary;
pub mod packets;
pub mod session;

// Protocol constants
pub mod constants {
    pub const MAGIC_NUMBER: u32 = 0x5345_5650; // "SEVP" in ASCII
    pub const DEFAULT_PORT: u16 = 443;
    pub const DEFAULT_HUB: &str = "VPN";
}

/// `SoftEther` protocol version information
#[derive(Debug, Clone)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
    pub build: u16,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self {
            major: 4,
            minor: 0,
            build: 0,
        }
    }
}

/// `SoftEther` SSL-VPN protocol handler
///
/// This is a simplified implementation for the static library.
/// Real networking is handled by the integrating application.
pub struct ProtocolHandler {
    server_addr: SocketAddr,
    protocol_version: ProtocolVersion,
    session_id: Option<String>,
    sequence_number: u32,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new(server_addr: SocketAddr) -> Result<Self> {
        Ok(ProtocolHandler {
            server_addr,
            protocol_version: ProtocolVersion::default(),
            session_id: None,
            sequence_number: 0,
        })
    }

    /// Get server address
    pub fn server_address(&self) -> SocketAddr {
        self.server_addr
    }

    /// Get protocol version
    pub fn protocol_version(&self) -> &ProtocolVersion {
        &self.protocol_version
    }

    /// Check if session is established
    pub fn has_session(&self) -> bool {
        self.session_id.is_some()
    }

    /// Get current session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Set session ID (called after authentication)
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Get next sequence number
    pub fn next_sequence(&mut self) -> u32 {
        self.sequence_number += 1;
        self.sequence_number
    }
}
