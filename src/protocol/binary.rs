//! Binary SoftEther VPN Protocol Implementation
//! 
//! This module implements the actual binary protocol used by SoftEther VPN
//! for high-performance packet transmission and session management

use crate::error::{Result, VpnError};
use bytes::{Bytes, BytesMut, Buf, BufMut};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// SoftEther protocol constants
pub mod protocol_constants {
    pub const SOFTETHER_MAGIC: u32 = 0x53455656; // "SEVV" 
    pub const PROTOCOL_VERSION: u16 = 0x0001;
    pub const MAX_PACKET_SIZE: usize = 65536;
    
    // Packet types
    pub const PACKET_TYPE_HELLO: u8 = 0x01;
    pub const PACKET_TYPE_AUTH_REQUEST: u8 = 0x02;
    pub const PACKET_TYPE_AUTH_RESPONSE: u8 = 0x03;
    pub const PACKET_TYPE_SESSION_ESTABLISH: u8 = 0x04;
    pub const PACKET_TYPE_SESSION_RESPONSE: u8 = 0x05;
    pub const PACKET_TYPE_DATA: u8 = 0x06;
    pub const PACKET_TYPE_KEEPALIVE: u8 = 0x07;
    pub const PACKET_TYPE_DISCONNECT: u8 = 0x08;
    pub const PACKET_TYPE_ERROR: u8 = 0xFF;
}

use protocol_constants::*;

/// Binary protocol packet structure
#[derive(Debug, Clone)]
pub struct SoftEtherPacket {
    pub packet_type: u8,
    pub sequence_number: u32,
    pub session_id: u32,
    pub data_length: u16,
    pub data: Bytes,
    pub checksum: u32,
}

impl SoftEtherPacket {
    /// Create a new SoftEther protocol packet
    pub fn new(packet_type: u8, session_id: u32, data: Bytes) -> Self {
        let data_length = data.len() as u16;
        let checksum = Self::calculate_checksum(&data);
        
        Self {
            packet_type,
            sequence_number: 0, // Set by sender
            session_id,
            data_length,
            data,
            checksum,
        }
    }

    /// Serialize packet to binary format
    pub fn to_bytes(&self) -> Bytes {
        let total_size = 4 + 2 + 1 + 4 + 4 + 2 + self.data.len() + 4;
        let mut buf = BytesMut::with_capacity(total_size);
        
        // Header
        buf.put_u32(SOFTETHER_MAGIC);           // Magic number
        buf.put_u16(PROTOCOL_VERSION);          // Protocol version
        buf.put_u8(self.packet_type);           // Packet type
        buf.put_u32(self.sequence_number);      // Sequence number
        buf.put_u32(self.session_id);           // Session ID
        buf.put_u16(self.data_length);          // Data length
        
        // Data
        buf.put(self.data.clone());
        
        // Checksum
        buf.put_u32(self.checksum);
        
        buf.freeze()
    }

    /// Deserialize packet from binary format
    pub fn from_bytes(mut data: Bytes) -> Result<Self> {
        if data.len() < 17 { // Minimum header size
            return Err(VpnError::Protocol("Packet too short".to_string()));
        }

        // Verify magic number
        let magic = data.get_u32();
        if magic != SOFTETHER_MAGIC {
            return Err(VpnError::Protocol("Invalid magic number".to_string()));
        }

        // Parse header
        let version = data.get_u16();
        if version != PROTOCOL_VERSION {
            return Err(VpnError::Protocol("Unsupported protocol version".to_string()));
        }

        let packet_type = data.get_u8();
        let sequence_number = data.get_u32();
        let session_id = data.get_u32();
        let data_length = data.get_u16();

        // Validate data length
        if data.len() < (data_length as usize + 4) { // +4 for checksum
            return Err(VpnError::Protocol("Invalid data length".to_string()));
        }

        // Extract data
        let packet_data = data.copy_to_bytes(data_length as usize);
        let checksum = data.get_u32();

        // Verify checksum
        let calculated_checksum = Self::calculate_checksum(&packet_data);
        if checksum != calculated_checksum {
            return Err(VpnError::Protocol("Checksum mismatch".to_string()));
        }

        Ok(Self {
            packet_type,
            sequence_number,
            session_id,
            data_length,
            data: packet_data,
            checksum,
        })
    }

    /// Calculate packet checksum
    fn calculate_checksum(data: &[u8]) -> u32 {
        // Simple CRC32-like checksum
        let mut checksum: u32 = 0xFFFFFFFF;
        for &byte in data {
            checksum ^= byte as u32;
            for _ in 0..8 {
                if checksum & 1 != 0 {
                    checksum = (checksum >> 1) ^ 0xEDB88320;
                } else {
                    checksum >>= 1;
                }
            }
        }
        !checksum
    }

    /// Create hello packet for protocol negotiation
    pub fn create_hello() -> Self {
        let hello_data = format!("SoftEther VPN Protocol v{}", PROTOCOL_VERSION);
        Self::new(PACKET_TYPE_HELLO, 0, Bytes::from(hello_data))
    }

    /// Create authentication request packet
    pub fn create_auth_request(username: &str, password: &str, hub: &str) -> Self {
        let auth_data = format!("{}@{}:{}", username, hub, password);
        Self::new(PACKET_TYPE_AUTH_REQUEST, 0, Bytes::from(auth_data))
    }

    /// Create session establishment packet
    pub fn create_session_establish(session_id: u32) -> Self {
        let session_data = format!("ESTABLISH_SESSION:{}", session_id);
        Self::new(PACKET_TYPE_SESSION_ESTABLISH, session_id, Bytes::from(session_data))
    }

    /// Create keepalive packet
    pub fn create_keepalive(session_id: u32, sequence: u32) -> Self {
        let mut packet = Self::new(PACKET_TYPE_KEEPALIVE, session_id, Bytes::from("KEEPALIVE"));
        packet.sequence_number = sequence;
        packet
    }

    /// Create data packet for VPN traffic
    pub fn create_data_packet(session_id: u32, sequence: u32, vpn_data: Bytes) -> Self {
        let mut packet = Self::new(PACKET_TYPE_DATA, session_id, vpn_data);
        packet.sequence_number = sequence;
        packet
    }

    /// Create disconnect packet
    pub fn create_disconnect(session_id: u32) -> Self {
        Self::new(PACKET_TYPE_DISCONNECT, session_id, Bytes::from("DISCONNECT"))
    }
}

/// High-performance binary protocol client
pub struct BinaryProtocolClient {
    stream: Option<TcpStream>,
    server_addr: SocketAddr,
    session_id: Option<u32>,
    sequence_counter: u32,
    is_connected: bool,
}

impl BinaryProtocolClient {
    /// Create a new binary protocol client
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            stream: None,
            server_addr,
            session_id: None,
            sequence_counter: 0,
            is_connected: false,
        }
    }

    /// Connect to SoftEther server using binary protocol
    pub async fn connect(&mut self) -> Result<()> {
        log::info!("Connecting to SoftEther server via binary protocol: {}", self.server_addr);
        
        // Establish TCP connection
        let stream = TcpStream::connect(self.server_addr).await
            .map_err(|e| VpnError::Network(format!("TCP connection failed: {}", e)))?;
        
        self.stream = Some(stream);
        self.is_connected = true;
        
        // Send protocol hello
        self.send_hello().await?;
        
        log::info!("Binary protocol connection established");
        Ok(())
    }

    /// Send hello packet and negotiate protocol
    async fn send_hello(&mut self) -> Result<()> {
        let hello_packet = SoftEtherPacket::create_hello();
        self.send_packet(hello_packet).await?;
        
        // Wait for hello response
        let response = self.receive_packet().await?;
        if response.packet_type != PACKET_TYPE_HELLO {
            return Err(VpnError::Protocol("Invalid hello response".to_string()));
        }
        
        log::debug!("Protocol negotiation successful");
        Ok(())
    }

    /// Authenticate using binary protocol
    pub async fn authenticate(&mut self, username: &str, password: &str, hub: &str) -> Result<u32> {
        log::info!("Authenticating via binary protocol: {}@{}", username, hub);
        
        // Send authentication request
        let auth_packet = SoftEtherPacket::create_auth_request(username, password, hub);
        self.send_packet(auth_packet).await?;
        
        // Receive authentication response
        let response = self.receive_packet().await?;
        match response.packet_type {
            PACKET_TYPE_AUTH_RESPONSE => {
                // Parse session ID from response
                let response_data = String::from_utf8_lossy(&response.data);
                if response_data.starts_with("AUTH_SUCCESS:") {
                    let session_id = response_data.trim_start_matches("AUTH_SUCCESS:")
                        .parse::<u32>()
                        .map_err(|_| VpnError::Authentication("Invalid session ID".to_string()))?;
                    
                    self.session_id = Some(session_id);
                    log::info!("Authentication successful, session ID: {}", session_id);
                    Ok(session_id)
                } else {
                    Err(VpnError::Authentication("Authentication failed".to_string()))
                }
            },
            PACKET_TYPE_ERROR => {
                let error_msg = String::from_utf8_lossy(&response.data);
                Err(VpnError::Authentication(format!("Auth error: {}", error_msg)))
            },
            _ => Err(VpnError::Protocol("Unexpected response to auth request".to_string()))
        }
    }

    /// Establish VPN session
    pub async fn establish_session(&mut self) -> Result<()> {
        let session_id = self.session_id.ok_or_else(|| 
            VpnError::Connection("Not authenticated".to_string()))?;
        
        log::info!("Establishing VPN session: {}", session_id);
        
        let session_packet = SoftEtherPacket::create_session_establish(session_id);
        self.send_packet(session_packet).await?;
        
        let response = self.receive_packet().await?;
        if response.packet_type != PACKET_TYPE_SESSION_RESPONSE {
            return Err(VpnError::Protocol("Invalid session response".to_string()));
        }
        
        log::info!("VPN session established successfully");
        Ok(())
    }

    /// Send keepalive packet
    pub async fn send_keepalive(&mut self) -> Result<()> {
        let session_id = self.session_id.ok_or_else(|| 
            VpnError::Connection("Not authenticated".to_string()))?;
        
        self.sequence_counter += 1;
        let keepalive_packet = SoftEtherPacket::create_keepalive(session_id, self.sequence_counter);
        
        self.send_packet(keepalive_packet).await?;
        log::debug!("Keepalive sent, sequence: {}", self.sequence_counter);
        Ok(())
    }

    /// Send VPN data packet
    pub async fn send_vpn_data(&mut self, data: Bytes) -> Result<()> {
        let session_id = self.session_id.ok_or_else(|| 
            VpnError::Connection("Not authenticated".to_string()))?;
        
        self.sequence_counter += 1;
        let data_packet = SoftEtherPacket::create_data_packet(session_id, self.sequence_counter, data);
        
        self.send_packet(data_packet).await?;
        Ok(())
    }

    /// Send a packet over the binary protocol
    async fn send_packet(&mut self, packet: SoftEtherPacket) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| 
            VpnError::Connection("Not connected".to_string()))?;
        
        let packet_bytes = packet.to_bytes();
        stream.write_all(&packet_bytes).await
            .map_err(|e| VpnError::Network(format!("Send failed: {}", e)))?;
        
        Ok(())
    }

    /// Receive a packet from the binary protocol
    async fn receive_packet(&mut self) -> Result<SoftEtherPacket> {
        let stream = self.stream.as_mut().ok_or_else(|| 
            VpnError::Connection("Not connected".to_string()))?;
        
        // Read packet header first
        let mut header_buf = [0u8; 17]; // Minimum header size
        stream.read_exact(&mut header_buf).await
            .map_err(|e| VpnError::Network(format!("Header read failed: {}", e)))?;
        
        // Parse data length from header
        let data_length = u16::from_be_bytes([header_buf[15], header_buf[16]]);
        
        // Read remaining data + checksum
        let mut remaining_buf = vec![0u8; data_length as usize + 4];
        stream.read_exact(&mut remaining_buf).await
            .map_err(|e| VpnError::Network(format!("Data read failed: {}", e)))?;
        
        // Combine header and data
        let mut full_packet = BytesMut::with_capacity(header_buf.len() + remaining_buf.len());
        full_packet.put_slice(&header_buf);
        full_packet.put_slice(&remaining_buf);
        
        SoftEtherPacket::from_bytes(full_packet.freeze())
    }

    /// Disconnect from server
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(session_id) = self.session_id {
            let disconnect_packet = SoftEtherPacket::create_disconnect(session_id);
            let _ = self.send_packet(disconnect_packet).await; // Best effort
        }
        
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        
        self.is_connected = false;
        self.session_id = None;
        
        log::info!("Disconnected from binary protocol server");
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected && self.session_id.is_some()
    }

    /// Get current session ID
    pub fn session_id(&self) -> Option<u32> {
        self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let original_data = Bytes::from("Test packet data");
        let packet = SoftEtherPacket::new(PACKET_TYPE_DATA, 12345, original_data.clone());
        
        let serialized = packet.to_bytes();
        let deserialized = SoftEtherPacket::from_bytes(serialized).unwrap();
        
        assert_eq!(packet.packet_type, deserialized.packet_type);
        assert_eq!(packet.session_id, deserialized.session_id);
        assert_eq!(packet.data, deserialized.data);
        assert_eq!(packet.checksum, deserialized.checksum);
    }

    #[test]
    fn test_checksum_calculation() {
        let data1 = b"Hello World";
        let data2 = b"Hello World";
        let data3 = b"Hello World!";
        
        let checksum1 = SoftEtherPacket::calculate_checksum(data1);
        let checksum2 = SoftEtherPacket::calculate_checksum(data2);
        let checksum3 = SoftEtherPacket::calculate_checksum(data3);
        
        assert_eq!(checksum1, checksum2);
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_packet_types() {
        let hello = SoftEtherPacket::create_hello();
        assert_eq!(hello.packet_type, PACKET_TYPE_HELLO);
        
        let auth = SoftEtherPacket::create_auth_request("user", "pass", "hub");
        assert_eq!(auth.packet_type, PACKET_TYPE_AUTH_REQUEST);
        
        let keepalive = SoftEtherPacket::create_keepalive(123, 456);
        assert_eq!(keepalive.packet_type, PACKET_TYPE_KEEPALIVE);
        assert_eq!(keepalive.session_id, 123);
        assert_eq!(keepalive.sequence_number, 456);
    }
}
