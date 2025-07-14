//! `SoftEther` SSL-VPN protocol implementation for static library

use crate::error::{Result, VpnError};
use std::net::SocketAddr;

pub mod auth;
pub mod session;
pub mod watermark;
pub mod pack;
pub mod binary;

// Re-export main types
pub use auth::AuthClient;
pub use pack::{Pack, Element, Value, ElementType};
pub use watermark::{WatermarkClient, WatermarkResponse, SOFTETHER_WATERMARK};
pub use binary::BinaryProtocolClient;

// Protocol constants
pub mod constants {
    pub const DEFAULT_PORT: u16 = 443;
    pub const DEFAULT_HUB: &str = "VPN";
    
    // HTTP endpoints for SoftEther protocol
    pub const WATERMARK_ENDPOINT: &str = "/vpnsvc/connect.cgi";
    pub const HTTP_CONTENT_TYPE_PACK: &str = "application/octet-stream";
    pub const HTTP_KEEP_ALIVE: &str = "timeout=15, max=100";
}

/// `SoftEther` SSL-VPN protocol handler
///
/// This implements the actual SoftEther SSL-VPN protocol:
/// 1. HTTP Watermark handshake to establish session
/// 2. PACK binary format over HTTPS for all data communication
pub struct ProtocolHandler {
    server_addr: SocketAddr,
    watermark_client: Option<WatermarkClient>,
    session_established: bool,
    session_id: Option<String>,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new(server_addr: SocketAddr, verify_certificate: bool) -> Result<Self> {
        let watermark_client = WatermarkClient::new(server_addr, None, verify_certificate)?;
        
        Ok(ProtocolHandler {
            server_addr,
            watermark_client: Some(watermark_client),
            session_established: false,
            session_id: None,
        })
    }

    /// Get server address
    pub fn server_address(&self) -> SocketAddr {
        self.server_addr
    }

    /// Establish VPN session using HTTP watermark handshake
    pub async fn establish_session(&mut self) -> Result<()> {
        let watermark_client = self.watermark_client.as_ref().ok_or_else(|| {
            VpnError::Protocol("Watermark client not initialized".to_string())
        })?;

        let response = watermark_client.send_watermark_handshake().await?;
        
        if response.is_session_established() {
            self.session_established = true;
            // Generate a session ID (in real implementation, this would come from server)
            self.session_id = Some(format!("session_{}", fastrand::u64(..)));
            Ok(())
        } else {
            Err(VpnError::Protocol("Failed to establish session".to_string()))
        }
    }

    /// Check if session is established
    pub fn has_session(&self) -> bool {
        self.session_established
    }

    /// Get current session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Send PACK data over HTTPS (post-watermark communication)
    pub async fn send_pack(&self, pack: &Pack) -> Result<Pack> {
        if !self.session_established {
            return Err(VpnError::Protocol("Session not established".to_string()));
        }

        let watermark_client = self.watermark_client.as_ref().ok_or_else(|| {
            VpnError::Protocol("Watermark client not available".to_string())
        })?;

        // Serialize PACK to binary format
        let pack_data = pack.to_bytes()?;

        // Send via HTTP POST with binary PACK data
        let response = watermark_client.http_client
            .post(&format!("{}{}", watermark_client.base_url, constants::WATERMARK_ENDPOINT))
            .header("Content-Type", constants::HTTP_CONTENT_TYPE_PACK)
            .header("Connection", "Keep-Alive")
            .header("Keep-Alive", constants::HTTP_KEEP_ALIVE)
            .body(pack_data.to_vec())
            .send()
            .await
            .map_err(|e| VpnError::Network(format!("PACK send failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(VpnError::Protocol(format!(
                "PACK communication failed: HTTP {}",
                response.status()
            )));
        }

        // Read and parse response PACK
        let response_bytes = response.bytes().await.map_err(|e| {
            VpnError::Network(format!("Failed to read PACK response: {}", e))
        })?;

        Pack::from_bytes(response_bytes)
    }

    /// Create a data PACK for VPN communication
    pub fn create_data_pack(&self, packet_data: &[u8]) -> Pack {
        let mut pack = Pack::new();
        
        if let Some(session_id) = &self.session_id {
            pack.add_str("session_id", session_id);
        }
        
        pack.add_data("packet_data", packet_data.to_vec());
        pack.add_int64("timestamp", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());
        
        pack
    }

    /// Create a keepalive PACK
    pub fn create_keepalive_pack(&self) -> Pack {
        let mut pack = Pack::new();
        
        if let Some(session_id) = &self.session_id {
            pack.add_str("session_id", session_id);
        }
        
        pack.add_str("type", "keepalive");
        pack.add_int64("timestamp", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());
        
        pack
    }
}
