//! High-Performance VPN Client with Optimizations
//! 
//! This module provides an optimized VPN client implementation with:
//! - Async packet processing with batching
//! - Connection pooling and load balancing
//! - Adaptive performance tuning
//! - Real-time monitoring and statistics

use crate::error::{Result, VpnError};
use crate::config::VpnConfig;
// Note: Binary protocol removed - using HTTP Watermark + PACK instead
// use crate::protocol::binary::BinaryProtocolClient;
use crate::tunnel::real_tun::RealTunInterface;
use bytes::Bytes;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::{RwLock, mpsc, Semaphore};
use tokio::time::{Duration, Instant, interval};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Packet batch size for bulk processing
    pub packet_batch_size: usize,
    /// Buffer sizes
    pub send_buffer_size: usize,
    pub receive_buffer_size: usize,
    /// Timeout settings
    pub connection_timeout: Duration,
    pub keepalive_interval: Duration,
    /// Performance tuning
    pub enable_compression: bool,
    pub enable_packet_batching: bool,
    pub adaptive_mtu: bool,
    /// Monitoring
    pub stats_interval: Duration,
    pub enable_detailed_stats: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            packet_batch_size: 32,
            send_buffer_size: 65536,
            receive_buffer_size: 65536,
            connection_timeout: Duration::from_secs(30),
            keepalive_interval: Duration::from_secs(30),
            enable_compression: true,
            enable_packet_batching: true,
            adaptive_mtu: true,
            stats_interval: Duration::from_secs(10),
            enable_detailed_stats: true,
        }
    }
}

/// Real-time performance statistics
#[derive(Debug)]
pub struct PerformanceStats {
    // Traffic statistics
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub packets_sent: AtomicU64,
    pub packets_received: AtomicU64,
    
    // Performance metrics
    pub avg_latency_ms: AtomicU64,
    pub throughput_mbps: AtomicU64,
    pub packet_loss_percent: AtomicU64,
    pub connection_drops: AtomicU64,
    
    // Resource usage
    pub cpu_usage_percent: AtomicU64,
    pub memory_usage_mb: AtomicU64,
    pub active_connections: AtomicU64,
    
    // Error statistics
    pub protocol_errors: AtomicU64,
    pub network_errors: AtomicU64,
    pub tunnel_errors: AtomicU64,
    
    // Performance tracking
    pub last_update: RwLock<Instant>,
    pub is_monitoring: AtomicBool,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            packets_sent: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            avg_latency_ms: AtomicU64::new(0),
            throughput_mbps: AtomicU64::new(0),
            packet_loss_percent: AtomicU64::new(0),
            connection_drops: AtomicU64::new(0),
            cpu_usage_percent: AtomicU64::new(0),
            memory_usage_mb: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            protocol_errors: AtomicU64::new(0),
            network_errors: AtomicU64::new(0),
            tunnel_errors: AtomicU64::new(0),
            last_update: RwLock::new(Instant::now()),
            is_monitoring: AtomicBool::new(false),
        }
    }
}

impl PerformanceStats {
    /// Create new performance statistics
    pub fn new() -> Self {
        Self {
            last_update: RwLock::new(Instant::now()),
            is_monitoring: AtomicBool::new(false),
            ..Default::default()
        }
    }

    /// Update traffic statistics
    pub fn update_traffic(&self, bytes_sent: u64, bytes_received: u64, packets_sent: u64, packets_received: u64) {
        self.bytes_sent.fetch_add(bytes_sent, Ordering::Relaxed);
        self.bytes_received.fetch_add(bytes_received, Ordering::Relaxed);
        self.packets_sent.fetch_add(packets_sent, Ordering::Relaxed);
        self.packets_received.fetch_add(packets_received, Ordering::Relaxed);
    }

    /// Update performance metrics
    pub fn update_performance(&self, latency_ms: u64, throughput_mbps: u64) {
        // Use exponential moving average for smoother metrics
        let current_latency = self.avg_latency_ms.load(Ordering::Relaxed);
        let new_latency = if current_latency == 0 {
            latency_ms
        } else {
            (current_latency * 7 + latency_ms) / 8 // 87.5% weight to history
        };
        self.avg_latency_ms.store(new_latency, Ordering::Relaxed);
        
        let current_throughput = self.throughput_mbps.load(Ordering::Relaxed);
        let new_throughput = if current_throughput == 0 {
            throughput_mbps
        } else {
            (current_throughput * 7 + throughput_mbps) / 8
        };
        self.throughput_mbps.store(new_throughput, Ordering::Relaxed);
    }

    /// Get current statistics as a snapshot
    pub fn snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot {
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            packets_sent: self.packets_sent.load(Ordering::Relaxed),
            packets_received: self.packets_received.load(Ordering::Relaxed),
            avg_latency_ms: self.avg_latency_ms.load(Ordering::Relaxed),
            throughput_mbps: self.throughput_mbps.load(Ordering::Relaxed),
            packet_loss_percent: self.packet_loss_percent.load(Ordering::Relaxed),
            connection_drops: self.connection_drops.load(Ordering::Relaxed),
            cpu_usage_percent: self.cpu_usage_percent.load(Ordering::Relaxed),
            memory_usage_mb: self.memory_usage_mb.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            protocol_errors: self.protocol_errors.load(Ordering::Relaxed),
            network_errors: self.network_errors.load(Ordering::Relaxed),
            tunnel_errors: self.tunnel_errors.load(Ordering::Relaxed),
            timestamp: Instant::now(),
        }
    }
}

/// Performance statistics snapshot
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub avg_latency_ms: u64,
    pub throughput_mbps: u64,
    pub packet_loss_percent: u64,
    pub connection_drops: u64,
    pub cpu_usage_percent: u64,
    pub memory_usage_mb: u64,
    pub active_connections: u64,
    pub protocol_errors: u64,
    pub network_errors: u64,
    pub tunnel_errors: u64,
    pub timestamp: Instant,
}

/// Packet batch for optimized processing
#[derive(Debug)]
struct PacketBatch {
    packets: Vec<Bytes>,
    total_size: usize,
    created_at: Instant,
}

impl PacketBatch {
    fn new() -> Self {
        Self {
            packets: Vec::new(),
            total_size: 0,
            created_at: Instant::now(),
        }
    }

    fn add_packet(&mut self, packet: Bytes) -> bool {
        self.total_size += packet.len();
        self.packets.push(packet);
        
        // Return true if batch should be flushed
        self.packets.len() >= 32 || self.total_size >= 65536 || 
        self.created_at.elapsed() > Duration::from_millis(10)
    }

    fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    fn len(&self) -> usize {
        self.packets.len()
    }

    fn total_size(&self) -> usize {
        self.total_size
    }

    fn drain(&mut self) -> Vec<Bytes> {
        let packets = std::mem::take(&mut self.packets);
        self.total_size = 0;
        self.created_at = Instant::now();
        packets
    }
}

/// High-performance optimized VPN client
pub struct OptimizedVpnClient {
    config: VpnConfig,
    perf_config: PerformanceConfig,
    stats: Arc<PerformanceStats>,
    // Note: Binary protocol removed - using HTTP Watermark + PACK instead
    // protocol_client: Option<BinaryProtocolClient>,
    tun_interface: Option<RealTunInterface>,
    
    // Async channels for packet processing
    outbound_tx: Option<mpsc::Sender<Bytes>>,
    inbound_tx: Option<mpsc::Sender<Bytes>>,
    
    // Connection management
    connection_semaphore: Arc<Semaphore>,
    is_running: Arc<AtomicBool>,
    
    // Performance optimization
    packet_batches: Arc<RwLock<PacketBatch>>,
    adaptive_mtu: Arc<AtomicU64>,
}

impl OptimizedVpnClient {
    /// Create new optimized VPN client
    pub fn new(config: VpnConfig, perf_config: Option<PerformanceConfig>) -> Self {
        let perf_config = perf_config.unwrap_or_default();
        let connection_semaphore = Arc::new(Semaphore::new(perf_config.max_connections));
        
        Self {
            config,
            perf_config,
            stats: Arc::new(PerformanceStats::new()),
            tun_interface: None,
            outbound_tx: None,
            inbound_tx: None,
            connection_semaphore,
            is_running: Arc::new(AtomicBool::new(false)),
            packet_batches: Arc::new(RwLock::new(PacketBatch::new())),
            adaptive_mtu: Arc::new(AtomicU64::new(1500)),
        }
    }

    /// Connect to VPN server with optimizations
    pub async fn connect(&mut self) -> Result<()> {
        log::info!("Connecting to VPN with performance optimizations");
        
        // Acquire connection permit
        let _permit = self.connection_semaphore.acquire().await
            .map_err(|_| VpnError::Connection("Connection limit reached".to_string()))?;
        
        // Connect using binary protocol
        let server_addr: SocketAddr = format!("{}:{}", self.config.server.address, self.config.server.port)
            .parse()
            .map_err(|e| VpnError::Config(format!("Invalid server address: {}", e)))?;
        
        // Note: Binary protocol removed - need to implement HTTP Watermark + PACK protocol
        // TODO: Replace with proper SoftEther SSL-VPN implementation
        return Err(VpnError::Network("Binary protocol no longer supported - use VpnClient instead".to_string()));
    }

    /// Start packet processing tasks
    async fn start_packet_processors(
        &self,
        mut outbound_rx: mpsc::Receiver<Bytes>,
        mut inbound_rx: mpsc::Receiver<Bytes>,
    ) -> Result<()> {
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        let _packet_batches = Arc::clone(&self.packet_batches);
        let enable_batching = self.perf_config.enable_packet_batching;
        
        // Outbound packet processor (TUN -> Server)
        tokio::spawn(async move {
            let mut batch = PacketBatch::new();
            let mut batch_timer = interval(Duration::from_millis(5));
            
            while is_running.load(Ordering::Relaxed) {
                tokio::select! {
                    packet = outbound_rx.recv() => {
                        if let Some(packet) = packet {
                            if enable_batching {
                                if batch.add_packet(packet) {
                                    // Process batch
                                    let packets = batch.drain();
                                    Self::process_outbound_batch(&stats, packets).await;
                                }
                            } else {
                                // Process individual packet
                                Self::process_outbound_packet(&stats, packet).await;
                            }
                        }
                    }
                    _ = batch_timer.tick() => {
                        if !batch.is_empty() {
                            // Flush pending batch
                            let packets = batch.drain();
                            Self::process_outbound_batch(&stats, packets).await;
                        }
                    }
                }
            }
        });

        // Inbound packet processor (Server -> TUN)
        let stats_clone = Arc::clone(&self.stats);
        let is_running_clone = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            while is_running_clone.load(Ordering::Relaxed) {
                if let Some(packet) = inbound_rx.recv().await {
                    Self::process_inbound_packet(&stats_clone, packet).await;
                }
            }
        });

        Ok(())
    }

    /// Process outbound packet batch
    async fn process_outbound_batch(stats: &PerformanceStats, packets: Vec<Bytes>) {
        let start_time = Instant::now();
        let mut total_bytes = 0;
        let packet_count = packets.len();
        
        for packet in packets {
            total_bytes += packet.len();
            // Send packet to VPN server
            // In real implementation, this would use the protocol client
        }
        
        let processing_time = start_time.elapsed();
        stats.update_traffic(total_bytes as u64, 0, packet_count as u64, 0);
        
        if processing_time > Duration::from_millis(100) {
            log::warn!("Slow outbound batch processing: {:?} for {} packets", processing_time, packet_count);
        }
    }

    /// Process individual outbound packet
    async fn process_outbound_packet(stats: &PerformanceStats, packet: Bytes) {
        let start_time = Instant::now();
        
        // Send packet to VPN server
        // In real implementation, this would use the protocol client
        
        let processing_time = start_time.elapsed();
        stats.update_traffic(packet.len() as u64, 0, 1, 0);
        
        if processing_time > Duration::from_millis(10) {
            log::warn!("Slow outbound packet processing: {:?}", processing_time);
        }
    }

    /// Process inbound packet
    async fn process_inbound_packet(stats: &PerformanceStats, packet: Bytes) {
        let start_time = Instant::now();
        
        // Send packet to TUN interface
        // In real implementation, this would use the TUN interface
        
        let processing_time = start_time.elapsed();
        stats.update_traffic(0, packet.len() as u64, 0, 1);
        
        if processing_time > Duration::from_millis(10) {
            log::warn!("Slow inbound packet processing: {:?}", processing_time);
        }
    }

    /// Start performance monitoring task
    async fn start_performance_monitor(&self) -> Result<()> {
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        let interval_duration = self.perf_config.stats_interval;
        let detailed_stats = self.perf_config.enable_detailed_stats;
        
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            let mut last_snapshot = stats.snapshot();
            
            while is_running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                let current_snapshot = stats.snapshot();
                
                // Calculate throughput
                let time_diff = current_snapshot.timestamp.duration_since(last_snapshot.timestamp);
                let bytes_diff = current_snapshot.bytes_sent + current_snapshot.bytes_received -
                                last_snapshot.bytes_sent - last_snapshot.bytes_received;
                
                if time_diff.as_secs() > 0 {
                    let throughput_mbps = (bytes_diff * 8) / (time_diff.as_secs() * 1_000_000);
                    stats.update_performance(0, throughput_mbps); // Latency updated elsewhere
                }
                
                if detailed_stats {
                    log::info!("Performance: {}MB/s, {}ms latency, {} active connections",
                        current_snapshot.throughput_mbps,
                        current_snapshot.avg_latency_ms,
                        current_snapshot.active_connections);
                }
                
                last_snapshot = current_snapshot;
            }
        });

        Ok(())
    }

    /// Start keepalive task
    async fn start_keepalive_task(&self) -> Result<()> {
        let is_running = Arc::clone(&self.is_running);
        let keepalive_interval = self.perf_config.keepalive_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(keepalive_interval);
            
            while is_running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // Send keepalive
                // In real implementation, this would use the protocol client
                log::debug!("Sending optimized keepalive");
            }
        });

        Ok(())
    }

    /// Send packet through optimized pipeline
    pub async fn send_packet(&self, packet: Bytes) -> Result<()> {
        if let Some(ref tx) = self.outbound_tx {
            tx.send(packet).await
                .map_err(|_| VpnError::Connection("Outbound channel closed".to_string()))?;
        } else {
            return Err(VpnError::Connection("Not connected".to_string()));
        }
        Ok(())
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> PerformanceSnapshot {
        self.stats.snapshot()
    }

    /// Optimize connection based on current performance
    pub async fn optimize_performance(&mut self) -> Result<()> {
        let stats = self.stats.snapshot();
        
        // Adaptive MTU adjustment
        if self.perf_config.adaptive_mtu {
            let current_mtu = self.adaptive_mtu.load(Ordering::Relaxed);
            let new_mtu = if stats.packet_loss_percent > 5 {
                // High packet loss - reduce MTU
                std::cmp::max(current_mtu - 100, 1280)
            } else if stats.avg_latency_ms < 50 && stats.throughput_mbps > 100 {
                // Good performance - try larger MTU
                std::cmp::min(current_mtu + 100, 9000)
            } else {
                current_mtu
            };
            
            if new_mtu != current_mtu {
                self.adaptive_mtu.store(new_mtu, Ordering::Relaxed);
                log::info!("Adaptive MTU adjusted to: {}", new_mtu);
            }
        }
        
        // Log performance recommendations
        if stats.avg_latency_ms > 200 {
            log::warn!("High latency detected ({}ms). Consider server optimization.", stats.avg_latency_ms);
        }
        
        if stats.throughput_mbps < 10 {
            log::warn!("Low throughput detected ({}MB/s). Check network conditions.", stats.throughput_mbps);
        }
        
        if stats.cpu_usage_percent > 80 {
            log::warn!("High CPU usage ({}%). Consider reducing packet batch size.", stats.cpu_usage_percent);
        }
        
        Ok(())
    }

    /// Disconnect with cleanup
    pub async fn disconnect(&mut self) -> Result<()> {
        log::info!("Disconnecting optimized VPN client");
        
        self.is_running.store(false, Ordering::Relaxed);
        self.stats.is_monitoring.store(false, Ordering::Relaxed);
        
        // Close channels
        self.outbound_tx = None;
        self.inbound_tx = None;
        
        // Note: Binary protocol client removed
        // Protocol client cleanup no longer needed
        
        // Close TUN interface
        if let Some(mut tun) = self.tun_interface.take() {
            tun.destroy_interface().await?;
        }
        
        log::info!("Optimized VPN client disconnected");
        Ok(())
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        // Note: Binary protocol client removed, using is_running status only
        self.is_running.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_batch() {
        let mut batch = PacketBatch::new();
        assert!(batch.is_empty());
        
        let small_packet = Bytes::from(vec![0u8; 100]);
        assert!(!batch.add_packet(small_packet)); // Should not trigger flush
        assert_eq!(batch.len(), 1);
        
        // Add many packets to trigger batch size flush
        for _ in 0..35 {
            let packet = Bytes::from(vec![0u8; 100]);
            if batch.add_packet(packet) {
                break; // Batch flushed
            }
        }
        
        assert!(batch.len() >= 32); // Should have triggered batch flush
    }

    #[test]
    fn test_performance_stats() {
        let stats = PerformanceStats::new();
        
        stats.update_traffic(1000, 2000, 10, 20);
        assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 1000);
        assert_eq!(stats.bytes_received.load(Ordering::Relaxed), 2000);
        
        stats.update_performance(50, 100);
        assert_eq!(stats.avg_latency_ms.load(Ordering::Relaxed), 50);
        assert_eq!(stats.throughput_mbps.load(Ordering::Relaxed), 100);
        
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.bytes_sent, 1000);
        assert_eq!(snapshot.avg_latency_ms, 50);
    }

    #[tokio::test]
    async fn test_optimized_client_creation() {
        let config = VpnConfig {
            server: crate::config::ServerConfig {
                hostname: "test.example.com".to_string(),
                port: 443,
                hub: "VPN".to_string(),
                use_ssl: true,
                verify_certificate: true,
                timeout: 30,
                keepalive_interval: 60,
            },
            auth: crate::config::AuthConfig {
                method: crate::config::AuthMethod::Password,
                username: Some("testuser".to_string()),
                password: Some("testpass".to_string()),
                client_cert: None,
                client_key: None,
                ca_cert: None,
            },
            connection_limits: Default::default(),
            network: Default::default(),
            logging: Default::default(),
        };
        
        let client = OptimizedVpnClient::new(config, None);
        assert!(!client.is_connected());
        assert_eq!(client.perf_config.max_connections, 10);
    }
}
