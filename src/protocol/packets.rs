//! Packet parsing and serialization for `SoftEther` protocol

use crate::error::{Result, VpnError};
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Packet types in `SoftEther` protocol
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketType {
    Handshake = 0x01,
    Auth = 0x02,
    Data = 0x03,
    Keepalive = 0x04,
    Disconnect = 0x05,
}

impl TryFrom<u8> for PacketType {
    type Error = VpnError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(PacketType::Handshake),
            0x02 => Ok(PacketType::Auth),
            0x03 => Ok(PacketType::Data),
            0x04 => Ok(PacketType::Keepalive),
            0x05 => Ok(PacketType::Disconnect),
            _ => Err(VpnError::Protocol(format!(
                "Unknown packet type: {value:#x}"
            ))),
        }
    }
}

/// `SoftEther` protocol packet
#[derive(Debug, Clone)]
pub struct Packet {
    pub packet_type: PacketType,
    pub sequence: u32,
    pub payload: Bytes,
}

impl Packet {
    /// Create a new packet
    pub fn new(packet_type: PacketType, sequence: u32, payload: Bytes) -> Self {
        Self {
            packet_type,
            sequence,
            payload,
        }
    }

    /// Serialize packet to bytes
    pub fn serialize(&self) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(9 + self.payload.len());

        // Magic number (4 bytes)
        buf.put_u32(crate::protocol::constants::MAGIC_NUMBER);

        // Packet type (1 byte)
        buf.put_u8(self.packet_type as u8);

        // Sequence number (4 bytes)
        buf.put_u32(self.sequence);

        // Payload
        buf.put_slice(&self.payload);

        Ok(buf.freeze())
    }

    /// Deserialize packet from bytes
    pub fn deserialize(mut data: Bytes) -> Result<Self> {
        if data.len() < 9 {
            return Err(VpnError::Protocol("Packet too short".to_string()));
        }

        // Check magic number
        let magic = data.get_u32();
        if magic != crate::protocol::constants::MAGIC_NUMBER {
            return Err(VpnError::Protocol(format!(
                "Invalid magic number: {magic:#x}"
            )));
        }

        // Parse packet type
        let packet_type = PacketType::try_from(data.get_u8())?;

        // Parse sequence number
        let sequence = data.get_u32();

        // Remaining data is payload
        let payload = data;

        Ok(Self {
            packet_type,
            sequence,
            payload,
        })
    }

    /// Get packet size
    pub fn size(&self) -> usize {
        9 + self.payload.len() // Header (9 bytes) + payload
    }
}

/// Packet builder for creating specific packet types
pub struct PacketBuilder;

impl PacketBuilder {
    /// Create handshake packet
    pub fn handshake(sequence: u32, version: u32, client_name: &str) -> Result<Packet> {
        let mut payload = BytesMut::new();
        payload.put_u32(version);
        payload.put_u32(u32::try_from(client_name.len()).unwrap_or(0));
        payload.put_slice(client_name.as_bytes());

        Ok(Packet::new(
            PacketType::Handshake,
            sequence,
            payload.freeze(),
        ))
    }

    /// Create authentication packet
    pub fn auth(sequence: u32, username: &str, password: &str) -> Result<Packet> {
        let mut payload = BytesMut::new();
        payload.put_u32(u32::try_from(username.len()).unwrap_or(0));
        payload.put_slice(username.as_bytes());
        payload.put_u32(u32::try_from(password.len()).unwrap_or(0));
        payload.put_slice(password.as_bytes());

        Ok(Packet::new(PacketType::Auth, sequence, payload.freeze()))
    }

    /// Create data packet
    pub fn data(sequence: u32, data: Bytes) -> Result<Packet> {
        Ok(Packet::new(PacketType::Data, sequence, data))
    }

    /// Create keepalive packet
    pub fn keepalive(sequence: u32) -> Result<Packet> {
        Ok(Packet::new(PacketType::Keepalive, sequence, Bytes::new()))
    }

    /// Create disconnect packet
    pub fn disconnect(sequence: u32, reason: &str) -> Result<Packet> {
        let mut payload = BytesMut::new();
        payload.put_u32(u32::try_from(reason.len()).unwrap_or(0));
        payload.put_slice(reason.as_bytes());

        Ok(Packet::new(
            PacketType::Disconnect,
            sequence,
            payload.freeze(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let packet = Packet::new(
            PacketType::Handshake,
            1,
            Bytes::from(b"test payload".to_vec()),
        );

        let serialized = packet.serialize().unwrap();
        let deserialized = Packet::deserialize(serialized).unwrap();

        assert_eq!(packet.packet_type, deserialized.packet_type);
        assert_eq!(packet.sequence, deserialized.sequence);
        assert_eq!(packet.payload, deserialized.payload);
    }

    #[test]
    fn test_packet_type_conversion() {
        assert_eq!(PacketType::try_from(0x01).unwrap(), PacketType::Handshake);
        assert_eq!(PacketType::try_from(0x02).unwrap(), PacketType::Auth);
        assert_eq!(PacketType::try_from(0x03).unwrap(), PacketType::Data);
        assert_eq!(PacketType::try_from(0x04).unwrap(), PacketType::Keepalive);
        assert_eq!(PacketType::try_from(0x05).unwrap(), PacketType::Disconnect);

        assert!(PacketType::try_from(0xFF).is_err());
    }

    #[test]
    fn test_packet_builder() {
        let handshake = PacketBuilder::handshake(1, 1, "test-client").unwrap();
        assert_eq!(handshake.packet_type, PacketType::Handshake);
        assert_eq!(handshake.sequence, 1);

        let auth = PacketBuilder::auth(2, "user", "pass").unwrap();
        assert_eq!(auth.packet_type, PacketType::Auth);
        assert_eq!(auth.sequence, 2);

        let keepalive = PacketBuilder::keepalive(3).unwrap();
        assert_eq!(keepalive.packet_type, PacketType::Keepalive);
        assert_eq!(keepalive.sequence, 3);
        assert!(keepalive.payload.is_empty());
    }

    #[test]
    fn test_invalid_packet() {
        // Test packet too short
        let short_data = Bytes::from(vec![0x01, 0x02]);
        assert!(Packet::deserialize(short_data).is_err());

        // Test invalid magic number
        let mut invalid_magic = BytesMut::new();
        invalid_magic.put_u32(0xDEAD_BEEF); // Wrong magic
        invalid_magic.put_u8(0x01);
        invalid_magic.put_u32(1);
        assert!(Packet::deserialize(invalid_magic.freeze()).is_err());
    }
}
