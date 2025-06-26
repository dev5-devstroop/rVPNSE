//! VPN Client - Core `SoftEther` Protocol Client for Static Library
//!
//! This module provides the main VpnClient struct that handles `SoftEther` SSL-VPN
//! protocol communication and tunnel management.

use crate::config::Config;
use crate::error::{Result, VpnError};
use crate::protocol::auth::AuthClient;
use crate::protocol::session::SessionManager;
use crate::tunnel::{TunnelConfig, TunnelManager};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Connection status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected, // Protocol connected but no tunnel
    Tunneling, // Full tunnel established
}

/// `SoftEther` VPN Client with full tunnel support
///
/// This client handles both `SoftEther` SSL-VPN protocol communication
/// and platform-specific tunnel management including:
/// - TUN/TAP interface creation
/// - Packet routing
/// - DNS configuration
/// - Connection limits and rate limiting
/// - Connection retry management
pub struct VpnClient {
    config: Config,
    auth_client: Option<AuthClient>,
    session_manager: Option<SessionManager>,
    tunnel_manager: Option<TunnelManager>,
    status: ConnectionStatus,
    server_endpoint: Option<SocketAddr>,

    /// Global connection tracker (shared across all clients if needed)
    connection_tracker: Arc<ConnectionTracker>,
}

impl VpnClient {
    /// Create a new VPN client with the given configuration
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid or connection tracking setup fails
    pub fn new(config: Config) -> Result<Self> {
        Ok(VpnClient {
            config,
            auth_client: None,
            session_manager: None,
            tunnel_manager: None,
            status: ConnectionStatus::Disconnected,
            server_endpoint: None,
            connection_tracker: Arc::new(ConnectionTracker::new()),
        })
    }

    /// Create a new VPN client with shared connection tracking
    /// This allows multiple clients to share connection limits
    pub fn new_with_shared_tracker(
        config: Config,
        tracker: Arc<ConnectionTracker>,
    ) -> Result<Self> {
        Ok(VpnClient {
            config,
            auth_client: None,
            session_manager: None,
            tunnel_manager: None,
            status: ConnectionStatus::Disconnected,
            server_endpoint: None,
            connection_tracker: tracker,
        })
    }

    /// Connect to `SoftEther` VPN server (protocol level only)
    ///
    /// This establishes the SSL-VPN protocol connection but does NOT:
    /// - Create TUN/TAP interfaces
    /// - Configure routing tables
    /// - Set up DNS
    ///
    /// Your application must handle platform networking separately.
    pub fn connect(&mut self, server: &str, port: u16) -> Result<()> {
        if self.status != ConnectionStatus::Disconnected {
            return Err(VpnError::Connection(
                "Already connected or connecting".to_string(),
            ));
        }

        // Create endpoint identifier for retry tracking
        let endpoint_key = format!("{server}:{port}");

        // Check connection limits and retry limits
        self.connection_tracker
            .can_connect(&self.config.connection_limits)?;
        self.connection_tracker
            .can_retry(&endpoint_key, &self.config.connection_limits)?;

        self.status = ConnectionStatus::Connecting;

        // Resolve server address - handle both IP addresses and hostnames
        let server_addr = Self::resolve_server_address(server, port)?;
        self.server_endpoint = Some(server_addr);

        // Attempt connection with retry logic
        let result = self.attempt_connection(server_addr, &endpoint_key);

        match result {
            Ok(_) => {
                // Record successful connection
                self.connection_tracker.record_connection();
                self.status = ConnectionStatus::Connected;
                Ok(())
            }
            Err(e) => {
                // Record retry attempt for rate limiting
                self.connection_tracker.record_retry(&endpoint_key);
                self.status = ConnectionStatus::Disconnected;
                Err(e)
            }
        }
    }

    /// Attempt connection with timeout and error handling
    fn attempt_connection(&mut self, server_addr: SocketAddr, endpoint_key: &str) -> Result<()> {
        // Add delay if this is a retry attempt
        if self.config.connection_limits.retry_delay > 0 {
            let retry_attempts = self.connection_tracker.retry_attempts.lock().unwrap();
            if let Some((count, _)) = retry_attempts.get(endpoint_key) {
                if *count > 0 {
                    std::thread::sleep(Duration::from_secs(
                        self.config.connection_limits.retry_delay as u64,
                    ));
                }
            }
        }

        // Initialize auth client (this would establish TLS connection)
        let auth_client = AuthClient::new(server_addr, &self.config)?;
        self.auth_client = Some(auth_client);

        Ok(())
    }

    /// Resolve server address - handles both IP addresses and hostnames
    fn resolve_server_address(server: &str, port: u16) -> Result<SocketAddr> {
        // First try to parse as IP address directly
        if let Ok(addr) = format!("{server}:{port}").parse::<SocketAddr>() {
            return Ok(addr);
        }

        // If that fails, try hostname resolution
        let addr_string = format!("{server}:{port}");
        let mut addrs = addr_string.to_socket_addrs().map_err(|e| {
            VpnError::Network(format!("Failed to resolve hostname '{server}': {e}"))
        })?;

        // Return the first resolved address
        addrs
            .next()
            .ok_or_else(|| VpnError::Network(format!("No addresses found for hostname '{server}'")))
    }

    /// Authenticate with `SoftEther` VPN server
    ///
    /// # Errors
    /// Returns an error if authentication fails or client is not connected
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        let auth_client = self
            .auth_client
            .as_mut()
            .ok_or_else(|| VpnError::Connection("Not connected".to_string()))?;

        auth_client.authenticate(username, password)?;

        // Initialize session manager after successful authentication
        let mut session_manager = SessionManager::new(&self.config)?;
        session_manager.start_session()?;
        self.session_manager = Some(session_manager);

        Ok(())
    }

    /// Disconnect from VPN server
    ///
    /// # Errors
    /// Returns an error if tunnel teardown fails
    pub fn disconnect(&mut self) -> Result<()> {
        // Record disconnection for connection tracking
        if self.status == ConnectionStatus::Connected || self.status == ConnectionStatus::Tunneling
        {
            self.connection_tracker.record_disconnection();
        }

        // Tear down tunnel first
        if let Some(ref mut tunnel_manager) = self.tunnel_manager {
            tunnel_manager.teardown_tunnel()?;
        }

        self.tunnel_manager = None;
        self.session_manager = None;
        self.auth_client = None;
        self.status = ConnectionStatus::Disconnected;
        self.server_endpoint = None;
        Ok(())
    }

    /// Tear down the VPN tunnel while keeping the connection
    pub fn teardown_tunnel(&mut self) -> Result<()> {
        if let Some(ref mut tunnel_manager) = self.tunnel_manager {
            tunnel_manager.teardown_tunnel()?;
            self.status = ConnectionStatus::Connected; // Back to just connected state
        }
        Ok(())
    }

    /// Get current connection status
    #[must_use]
    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    /// Get server endpoint (if connected)
    pub fn server_endpoint(&self) -> Option<SocketAddr> {
        self.server_endpoint
    }

    /// Send keepalive packet (protocol level)
    pub fn send_keepalive(&mut self) -> Result<()> {
        let session_manager = self
            .session_manager
            .as_mut()
            .ok_or_else(|| VpnError::Connection("Not authenticated".to_string()))?;

        session_manager.send_keepalive()
    }

    /// Check if client is ready for packet forwarding
    pub fn is_ready_for_packets(&self) -> bool {
        self.status == ConnectionStatus::Connected && self.session_manager.is_some()
    }

    /// Establish VPN tunnel (create TUN interface and configure routing)
    ///
    /// This creates a real TUN interface and configures system routing
    /// to send all traffic through the VPN tunnel.
    pub fn establish_tunnel(&mut self) -> Result<()> {
        if self.status != ConnectionStatus::Connected {
            return Err(VpnError::Connection("Must be connected first".to_string()));
        }

        if self.session_manager.is_none() {
            return Err(VpnError::Connection(
                "Must be authenticated first".to_string(),
            ));
        }

        // Create tunnel manager if not exists
        if self.tunnel_manager.is_none() {
            let tunnel_config = TunnelConfig::default();
            let tunnel_manager = TunnelManager::new(tunnel_config);
            self.tunnel_manager = Some(tunnel_manager);
        }

        // Establish the actual tunnel
        if let Some(ref mut tunnel_manager) = self.tunnel_manager {
            tunnel_manager.establish_tunnel()?;
            self.status = ConnectionStatus::Tunneling;
            println!("VPN tunnel established successfully - all traffic now routed through VPN");
        }

        Ok(())
    }

    /// Check if tunnel is established
    pub fn is_tunnel_established(&self) -> bool {
        self.status == ConnectionStatus::Tunneling
            && self
                .tunnel_manager
                .as_ref()
                .is_some_and(|tm| tm.is_established())
    }

    /// Get current public IP (for testing if traffic is routed through VPN)
    pub fn get_current_public_ip(&self) -> Result<String> {
        if let Some(ref tunnel_manager) = self.tunnel_manager {
            tunnel_manager.get_current_public_ip()
        } else {
            Err(VpnError::Connection(
                "No tunnel manager available".to_string(),
            ))
        }
    }
}

impl Drop for VpnClient {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

/// Connection tracking for limits and rate limiting
#[derive(Debug)]
pub struct ConnectionTracker {
    /// Active connection count
    active_connections: AtomicU32,
    /// Connection attempts per minute tracking
    connection_attempts: Arc<Mutex<Vec<Instant>>>,
    /// Connection retry tracking per endpoint
    retry_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl ConnectionTracker {
    fn new() -> Self {
        Self {
            active_connections: AtomicU32::new(0),
            connection_attempts: Arc::new(Mutex::new(Vec::new())),
            retry_attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if we can make a new connection based on limits
    fn can_connect(&self, config: &crate::config::ConnectionLimitsConfig) -> Result<()> {
        // Check concurrent connection limit
        if config.max_concurrent_connections > 0 {
            let current_connections = self.active_connections.load(Ordering::Relaxed);
            if current_connections >= config.max_concurrent_connections {
                return Err(VpnError::ConnectionLimitReached(format!(
                    "Maximum concurrent connections reached: {}/{}",
                    current_connections, config.max_concurrent_connections
                )));
            }
        }

        // Check rate limiting (connections per minute)
        if config.max_connections_per_minute > 0 {
            let mut attempts = self.connection_attempts.lock().unwrap();
            let now = Instant::now();
            let one_minute_ago = now - Duration::from_secs(60);

            // Remove old attempts
            attempts.retain(|&attempt_time| attempt_time > one_minute_ago);

            if attempts.len() >= config.max_connections_per_minute as usize {
                return Err(VpnError::RateLimitExceeded(format!(
                    "Too many connection attempts: {}/{} per minute",
                    attempts.len(),
                    config.max_connections_per_minute
                )));
            }

            attempts.push(now);
        }

        Ok(())
    }

    /// Check retry limits for a specific endpoint
    fn can_retry(
        &self,
        endpoint: &str,
        config: &crate::config::ConnectionLimitsConfig,
    ) -> Result<()> {
        if config.max_retry_attempts == 0 {
            return Ok(());
        }

        let mut retries = self.retry_attempts.lock().unwrap();
        let now = Instant::now();

        if let Some((count, last_attempt)) = retries.get(endpoint) {
            if *count >= config.max_retry_attempts {
                let time_since_last = now.duration_since(*last_attempt);
                let retry_cooldown = Duration::from_secs(
                    config.retry_delay as u64 * (*count - config.max_retry_attempts + 1) as u64,
                );

                if time_since_last < retry_cooldown {
                    return Err(VpnError::RetryLimitExceeded(format!(
                        "Too many retry attempts for {}: {}/{}. Wait {} seconds.",
                        endpoint,
                        count,
                        config.max_retry_attempts,
                        (retry_cooldown - time_since_last).as_secs()
                    )));
                } else {
                    // Reset retry count after cooldown
                    retries.insert(endpoint.to_string(), (0, now));
                }
            }
        }

        Ok(())
    }

    /// Record a connection attempt
    fn record_connection(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a disconnection
    fn record_disconnection(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record a retry attempt
    fn record_retry(&self, endpoint: &str) {
        let mut retries = self.retry_attempts.lock().unwrap();
        let now = Instant::now();
        let count = retries.get(endpoint).map(|(c, _)| *c).unwrap_or(0);
        retries.insert(endpoint.to_string(), (count + 1, now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpn_client_creation() {
        let config = Config::default();
        let client = VpnClient::new(config);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.status(), ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_connection_states() {
        let config = Config::default();
        let mut client = VpnClient::new(config).unwrap();

        assert_eq!(client.status(), ConnectionStatus::Disconnected);
        assert!(!client.is_ready_for_packets());

        // Note: Actual connection would require a real server
        // This just tests the state machine
        client.status = ConnectionStatus::Connecting;
        assert_eq!(client.status(), ConnectionStatus::Connecting);
    }
}
