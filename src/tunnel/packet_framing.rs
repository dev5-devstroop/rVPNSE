// Improved packet framing implementation based on SoftEther VPN
// This module handles proper encapsulation and framing of packets for VPN tunnels

use crate::error::{VpnError as Error, Result};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Packet header structure
/// Based on SoftEther's implementation but simplified for our needs
#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub version: u8,       // Protocol version
    pub packet_type: u8,   // Type of packet
    pub session_id: u32,   // Session identifier
    pub payload_size: u32, // Size of the payload data
}

impl PacketHeader {
    pub const SIZE: usize = 10; // 1 + 1 + 4 + 4
    pub const VERSION: u8 = 1;
    
    // Packet types
    pub const TYPE_DATA: u8 = 0;      // Regular data packet
    pub const TYPE_CONTROL: u8 = 1;   // Control packet
    pub const TYPE_ACK: u8 = 2;       // Acknowledgment packet
    pub const TYPE_KEEPALIVE: u8 = 3; // Keep-alive packet
    
    pub fn new(packet_type: u8, session_id: u32, payload_size: u32) -> Self {
        Self {
            version: Self::VERSION,
            packet_type,
            session_id,
            payload_size,
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(Self::SIZE);
        buffer.push(self.version);
        buffer.push(self.packet_type);
        buffer.extend_from_slice(&self.session_id.to_be_bytes());
        buffer.extend_from_slice(&self.payload_size.to_be_bytes());
        buffer
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(Error::PacketError("Header data too small".into()));
        }
        
        let version = data[0];
        let packet_type = data[1];
        
        let mut session_id_bytes = [0u8; 4];
        session_id_bytes.copy_from_slice(&data[2..6]);
        let session_id = u32::from_be_bytes(session_id_bytes);
        
        let mut payload_size_bytes = [0u8; 4];
        payload_size_bytes.copy_from_slice(&data[6..10]);
        let payload_size = u32::from_be_bytes(payload_size_bytes);
        
        Ok(Self {
            version,
            packet_type,
            session_id,
            payload_size,
        })
    }
}

/// PacketFramer - Handles packet framing for the VPN tunnel
pub struct PacketFramer {
    session_id: u32,
    remote_ip: IpAddr,
    // Stats for debugging
    sent_packets: u64,
    received_packets: u64,
    errors: u64,
}

impl PacketFramer {
    pub fn new(session_id: u32, remote_ip: IpAddr) -> Self {
        Self {
            session_id,
            remote_ip,
            sent_packets: 0,
            received_packets: 0,
            errors: 0,
        }
    }
    
    /// Frame a packet for sending through the tunnel
    pub fn frame_packet(&mut self, data: &[u8]) -> Vec<u8> {
        let header = PacketHeader::new(
            PacketHeader::TYPE_DATA,
            self.session_id,
            data.len() as u32
        );
        
        let mut framed_packet = header.to_bytes();
        framed_packet.extend_from_slice(data);
        
        self.sent_packets += 1;
        framed_packet
    }
    
    /// Decode a received packet
    pub fn decode_packet(&mut self, data: &[u8]) -> Result<(PacketHeader, Vec<u8>)> {
        if data.len() < PacketHeader::SIZE {
            self.errors += 1;
            return Err(Error::PacketError("Packet too small".into()));
        }
        
        let header = PacketHeader::from_bytes(&data[0..PacketHeader::SIZE])?;
        
        // Validate header
        if header.version != PacketHeader::VERSION {
            self.errors += 1;
            return Err(Error::PacketError(format!("Invalid packet version: {}", header.version)));
        }
        
        if (header.payload_size as usize) != data.len() - PacketHeader::SIZE {
            self.errors += 1;
            return Err(Error::PacketError(format!(
                "Payload size mismatch: expected {}, got {}",
                header.payload_size,
                data.len() - PacketHeader::SIZE
            )));
        }
        
        let payload = data[PacketHeader::SIZE..].to_vec();
        self.received_packets += 1;
        
        Ok((header, payload))
    }
    
    /// Create a keepalive packet
    pub fn create_keepalive(&self) -> Vec<u8> {
        let header = PacketHeader::new(
            PacketHeader::TYPE_KEEPALIVE,
            self.session_id,
            0
        );
        
        header.to_bytes()
    }
    
    /// Check if packet is a keepalive packet
    pub fn is_keepalive(&self, data: &[u8]) -> bool {
        if data.len() >= PacketHeader::SIZE {
            if let Ok(header) = PacketHeader::from_bytes(&data[0..PacketHeader::SIZE]) {
                return header.packet_type == PacketHeader::TYPE_KEEPALIVE;
            }
        }
        false
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (self.sent_packets, self.received_packets, self.errors)
    }
}

/// Thread-safe packet framer wrapper
pub struct SharedPacketFramer {
    inner: Arc<Mutex<PacketFramer>>,
}

impl SharedPacketFramer {
    pub fn new(session_id: u32, remote_ip: IpAddr) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PacketFramer::new(session_id, remote_ip))),
        }
    }
    
    pub fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
    
    pub async fn frame_packet(&self, data: &[u8]) -> Vec<u8> {
        let mut framer = self.inner.lock().await;
        framer.frame_packet(data)
    }
    
    pub async fn decode_packet(&self, data: &[u8]) -> Result<(PacketHeader, Vec<u8>)> {
        let mut framer = self.inner.lock().await;
        framer.decode_packet(data)
    }
    
    pub async fn create_keepalive(&self) -> Vec<u8> {
        let framer = self.inner.lock().await;
        framer.create_keepalive()
    }
    
    pub async fn is_keepalive(&self, data: &[u8]) -> bool {
        let framer = self.inner.lock().await;
        framer.is_keepalive(data)
    }
    
    pub async fn get_stats(&self) -> (u64, u64, u64) {
        let framer = self.inner.lock().await;
        framer.get_stats()
    }
}
