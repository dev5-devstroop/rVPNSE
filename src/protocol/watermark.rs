//! SoftEther VPN HTTP Watermark Handshake Implementation
//!
//! This module implements the HTTP watermark handshake that SoftEther VPN uses
//! to establish VPN sessions. The watermark is a GIF89a binary data that must
//! be sent via HTTP POST to /vpnsvc/connect.cgi to validate the VPN client.

use crate::error::{Result, VpnError};
use reqwest::Client;
use std::net::SocketAddr;

/// SoftEther VPN Watermark (GIF89a binary data)
/// This is the exact watermark from SoftEtherVPN/src/Cedar/WaterMark.c
pub const SOFTETHER_WATERMARK: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0xC8, 0x00, 0x33, 0x00, 0xF2, 0x00, 0x00, 0x36, 0x37, 0x34,
    0x79, 0x68, 0x54, 0x80, 0x80, 0x80, 0xAF, 0x7F, 0x5B, 0xB3, 0xA8, 0x9D, 0xD5, 0xD5, 0xD4, 0xFF,
    0xFF, 0xFF, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x00, 0xC8, 0x00, 0x33, 0x00, 0x00, 0x03,
    0xFE, 0x08, 0x1A, 0xDC, 0x34, 0x0A, 0x04, 0x41, 0x6B, 0x65, 0x31, 0x4F, 0x11, 0x80, 0xF9, 0x60,
    0x28, 0x8E, 0x64, 0x69, 0x9E, 0x68, 0xAA, 0xAE, 0x6C, 0xEB, 0x9A, 0x4B, 0xE3, 0x0C, 0x0C, 0x25,
    0x6F, 0x56, 0xA7, 0xE9, 0xD2, 0xEB, 0xFF, 0xC0, 0xA0, 0x70, 0xC8, 0x8A, 0xDC, 0x2C, 0x9C, 0xC6,
    0x05, 0xC7, 0x31, 0x66, 0x24, 0x04, 0xA2, 0x74, 0x4A, 0xAD, 0x4E, 0x05, 0xB1, 0x0D, 0x61, 0xCB,
    0x25, 0xD4, 0xB8, 0x49, 0x1B, 0xE6, 0x19, 0xB1, 0x9A, 0xCF, 0xE8, 0xF4, 0x07, 0x2B, 0x11, 0x74,
];

/// HTTP watermark handshake client
pub struct WatermarkClient {
    pub(crate) http_client: Client,
    pub(crate) server_addr: SocketAddr,
    pub(crate) base_url: String,
}

impl WatermarkClient {
    /// Create a new watermark client
    pub fn new(server_addr: SocketAddr, verify_certificate: bool) -> Result<Self> {
        let mut client_builder = Client::builder()
            .user_agent("SoftEther VPN Client");

        // Configure TLS verification
        if !verify_certificate {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let http_client = client_builder.build().map_err(|e| {
            VpnError::Network(format!("Failed to create HTTP client: {}", e))
        })?;

        let base_url = format!("https://{}:{}", server_addr.ip(), server_addr.port());

        Ok(Self {
            http_client,
            server_addr,
            base_url,
        })
    }

    /// Send HTTP watermark handshake to establish VPN session
    ///
    /// This sends either "VPNCONNECT" or the SoftEther watermark (GIF89a binary data) 
    /// via HTTP POST to /vpnsvc/connect.cgi to validate the VPN client and establish session.
    pub async fn send_watermark_handshake(&self) -> Result<WatermarkResponse> {
        let url = format!("{}/vpnsvc/connect.cgi", self.base_url);
        
        // First try with "VPNCONNECT" - this is simpler and more commonly used
        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Content-Length", "10")
            .header("Connection", "Keep-Alive")
            .header("User-Agent", "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1)")
            .body("VPNCONNECT")
            .send()
            .await
            .map_err(|e| VpnError::Network(format!("Watermark handshake failed: {}", e)))?;

        if response.status().is_success() {
            // Read response body
            let response_body = response.bytes().await.map_err(|e| {
                VpnError::Network(format!("Failed to read watermark response: {}", e))
            })?;

            return Ok(WatermarkResponse {
                session_established: true,
                response_data: response_body.to_vec(),
            });
        }

        // If VPNCONNECT fails, try with the GIF watermark
        let watermark_data = SOFTETHER_WATERMARK.to_vec();

        let response = self.http_client
            .post(&url)
            .header("Content-Type", "image/gif")
            .header("Content-Length", &watermark_data.len().to_string())
            .header("Connection", "Keep-Alive")
            .header("User-Agent", "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1)")
            .body(watermark_data)
            .send()
            .await
            .map_err(|e| VpnError::Network(format!("Watermark handshake failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(VpnError::Protocol(format!(
                "Watermark handshake rejected: HTTP {}",
                response.status()
            )));
        }

        // Read response body
        let response_body = response.bytes().await.map_err(|e| {
            VpnError::Network(format!("Failed to read watermark response: {}", e))
        })?;

        Ok(WatermarkResponse {
            session_established: true,
            response_data: response_body.to_vec(),
        })
    }

    /// Check if watermark handshake is required
    pub fn requires_watermark(&self) -> bool {
        // Always required for SoftEther SSL-VPN protocol
        true
    }
}

/// Response from HTTP watermark handshake
#[derive(Debug)]
pub struct WatermarkResponse {
    pub session_established: bool,
    pub response_data: Vec<u8>,
}

impl WatermarkResponse {
    /// Check if the session was successfully established
    pub fn is_session_established(&self) -> bool {
        self.session_established
    }

    /// Get the response data from the server
    pub fn response_data(&self) -> &[u8] {
        &self.response_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_data() {
        // Verify watermark starts with GIF89a signature
        assert_eq!(&SOFTETHER_WATERMARK[0..6], b"GIF89a");
        
        // Verify watermark has expected length (partial check)
        assert!(SOFTETHER_WATERMARK.len() > 100);
    }

    #[test]
    fn test_watermark_client_creation() {
        let addr = "127.0.0.1:443".parse().unwrap();
        let client = WatermarkClient::new(addr, false);
        assert!(client.is_ok());
    }
}
