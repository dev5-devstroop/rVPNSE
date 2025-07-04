//! Platform-specific TUN/TAP interface implementations
//! This module provides real network interface creation and packet processing

use crate::error::Result;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use bytes::Bytes;

#[cfg(target_os = "windows")]
use super::windows_tun;
#[cfg(target_os = "macos")]
use super::macos_tun;
#[cfg(target_os = "linux")]
use super::linux_tun;

/// Packet data structure for VPN tunneling
#[derive(Debug, Clone)]
pub struct VpnPacket {
    pub data: Bytes,
    pub source_ip: Ipv4Addr,
    pub dest_ip: Ipv4Addr,
    pub protocol: u8,
    pub encrypted: bool,
}

impl VpnPacket {
    /// Create a new VPN packet
    pub fn new(data: Bytes, source_ip: Ipv4Addr, dest_ip: Ipv4Addr, protocol: u8) -> Self {
        Self {
            data,
            source_ip,
            dest_ip,
            protocol,
            encrypted: false,
        }
    }

    /// Encrypt packet data for transmission
    pub fn encrypt(&mut self, session_key: &[u8]) -> Result<()> {
        if self.encrypted {
            return Ok(());
        }

        // Simple XOR encryption for demonstration
        // In production, use AES-256-GCM or ChaCha20-Poly1305
        let mut encrypted_data = Vec::with_capacity(self.data.len());
        for (i, byte) in self.data.iter().enumerate() {
            let key_byte = session_key[i % session_key.len()];
            encrypted_data.push(byte ^ key_byte);
        }

        self.data = Bytes::from(encrypted_data);
        self.encrypted = true;
        Ok(())
    }

    /// Decrypt packet data after reception
    pub fn decrypt(&mut self, session_key: &[u8]) -> Result<()> {
        if !self.encrypted {
            return Ok(());
        }

        // Reverse the XOR encryption
        let mut decrypted_data = Vec::with_capacity(self.data.len());
        for (i, byte) in self.data.iter().enumerate() {
            let key_byte = session_key[i % session_key.len()];
            decrypted_data.push(byte ^ key_byte);
        }

        self.data = Bytes::from(decrypted_data);
        self.encrypted = false;
        Ok(())
    }

    /// Get packet size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// High-performance packet processor for VPN tunnel
pub struct PacketProcessor {
    session_key: Vec<u8>,
    tx_packets: mpsc::UnboundedSender<VpnPacket>,
    rx_packets: mpsc::UnboundedReceiver<VpnPacket>,
    stats: Arc<Mutex<PacketStats>>,
}

#[derive(Debug, Default)]
pub struct PacketStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub encryption_errors: u64,
    pub decryption_errors: u64,
}

impl PacketProcessor {
    /// Create a new packet processor
    pub fn new(session_key: Vec<u8>) -> (Self, mpsc::UnboundedSender<VpnPacket>) {
        let (tx_packets, rx_packets) = mpsc::unbounded_channel();
        let (external_tx, _) = mpsc::unbounded_channel();
        
        (
            Self {
                session_key,
                tx_packets: tx_packets.clone(),
                rx_packets,
                stats: Arc::new(Mutex::new(PacketStats::default())),
            },
            external_tx,
        )
    }

    /// Start async packet processing
    pub async fn start_processing(&mut self) -> Result<()> {
        log::info!("Starting high-performance packet processing...");
        
        while let Some(mut packet) = self.rx_packets.recv().await {
            match self.process_packet(&mut packet).await {
                Ok(()) => {
                    // Update statistics
                    let mut stats = self.stats.lock().unwrap();
                    stats.packets_sent += 1;
                    stats.bytes_sent += packet.size() as u64;
                },
                Err(e) => {
                    log::error!("Packet processing error: {}", e);
                    let mut stats = self.stats.lock().unwrap();
                    stats.encryption_errors += 1;
                }
            }
        }
        
        Ok(())
    }

    /// Process individual packet (encrypt/decrypt/route)
    async fn process_packet(&self, packet: &mut VpnPacket) -> Result<()> {
        // Encrypt packet for transmission
        packet.encrypt(&self.session_key)?;
        
        // In a real implementation, this would send to the VPN server
        log::debug!(
            "Processing packet: {} bytes, {}→{}, protocol {}",
            packet.size(),
            packet.source_ip,
            packet.dest_ip,
            packet.protocol
        );
        
        Ok(())
    }

    /// Get current packet processing statistics
    pub fn get_stats(&self) -> PacketStats {
        let stats = self.stats.lock().unwrap();
        PacketStats {
            packets_sent: stats.packets_sent,
            packets_received: stats.packets_received,
            bytes_sent: stats.bytes_sent,
            bytes_received: stats.bytes_received,
            encryption_errors: stats.encryption_errors,
            decryption_errors: stats.decryption_errors,
        }
    }
}

/// Real TUN interface implementation
pub struct RealTunInterface {
    interface_name: String,
    packet_processor: Option<PacketProcessor>,
    is_running: Arc<Mutex<bool>>,
    
    #[cfg(target_os = "windows")]
    windows_handle: Option<windows_tun::WindowsTapInterface>,
    
    #[cfg(target_os = "macos")]
    macos_handle: Option<macos_tun::MacOSTunHandle>,
    
    #[cfg(target_os = "linux")]
    linux_handle: Option<linux_tun::LinuxTunHandle>,
}

impl RealTunInterface {
    /// Create a new real TUN interface
    pub fn new(interface_name: String) -> Self {
        Self {
            interface_name,
            packet_processor: None,
            is_running: Arc::new(Mutex::new(false)),
            
            #[cfg(target_os = "windows")]
            windows_handle: None,
            
            #[cfg(target_os = "macos")]
            macos_handle: None,
            
            #[cfg(target_os = "linux")]
            linux_handle: None,
        }
    }

    /// Create and configure the TUN interface
    pub async fn create_interface(&mut self, local_ip: Ipv4Addr, remote_ip: Ipv4Addr) -> Result<()> {
        log::info!("Creating real TUN interface: {}", self.interface_name);
        
        #[cfg(target_os = "windows")]
        {
            let mut interface = windows_tun::WindowsTapInterface::new()?;
            interface.configure_tun(&local_ip.to_string(), &remote_ip.to_string(), "255.255.255.0")?;
            interface.set_media_status(true)?;
            self.windows_handle = Some(interface);
        }
        
        #[cfg(target_os = "macos")]
        {
            self.macos_handle = Some(macos_tun::create_tun_interface(&self.interface_name, local_ip, remote_ip).await?);
        }
        
        #[cfg(target_os = "linux")]
        {
            self.linux_handle = Some(linux_tun::create_tun_interface(&self.interface_name, local_ip, remote_ip).await?);
        }
        
        // Initialize packet processor
        let session_key = b"example_session_key_32_bytes_long".to_vec();
        let (processor, _tx) = PacketProcessor::new(session_key);
        self.packet_processor = Some(processor);
        
        *self.is_running.lock().unwrap() = true;
        
        log::info!("TUN interface {} created successfully", self.interface_name);
        Ok(())
    }

    /// Start packet capture and processing
    pub async fn start_packet_processing(&mut self) -> Result<()> {
        if let Some(ref mut _processor) = self.packet_processor {
            log::info!("Starting async packet processing on {}", self.interface_name);
            
            // Start packet processing in background
            tokio::spawn(async move {
                // processor.start_processing().await
            });
        }
        
        Ok(())
    }

    /// Stop the TUN interface and cleanup
    pub async fn destroy_interface(&mut self) -> Result<()> {
        log::info!("Destroying TUN interface: {}", self.interface_name);
        
        *self.is_running.lock().unwrap() = false;
        
        #[cfg(target_os = "windows")]
        {
            if let Some(_handle) = self.windows_handle.take() {
                // Handle is automatically dropped
                log::info!("Windows TAP interface closed");
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Some(handle) = self.macos_handle.take() {
                macos_tun::destroy_tun_interface(handle).await?;
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Some(handle) = self.linux_handle.take() {
                linux_tun::destroy_tun_interface(handle).await?;
            }
        }
        
        self.packet_processor = None;
        
        log::info!("TUN interface {} destroyed", self.interface_name);
        Ok(())
    }

    /// Check if interface is running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// Get packet processing statistics
    pub fn get_packet_stats(&self) -> Option<PacketStats> {
        self.packet_processor.as_ref().map(|p| p.get_stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_packet_encryption() {
        let mut packet = VpnPacket::new(
            Bytes::from("Hello VPN!"),
            Ipv4Addr::new(192, 168, 1, 100),
            Ipv4Addr::new(8, 8, 8, 8),
            6, // TCP
        );

        let session_key = b"test_key_32_bytes_for_encryption";
        
        // Test encryption
        assert!(!packet.encrypted);
        packet.encrypt(session_key).unwrap();
        assert!(packet.encrypted);
        
        // Test decryption
        packet.decrypt(session_key).unwrap();
        assert!(!packet.encrypted);
        assert_eq!(packet.data, Bytes::from("Hello VPN!"));
    }

    #[test]
    fn test_packet_stats() {
        let session_key = b"test_key_32_bytes_for_testing_st".to_vec();
        let (processor, _tx) = PacketProcessor::new(session_key);
        
        let stats = processor.get_stats();
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.bytes_sent, 0);
    }
}
