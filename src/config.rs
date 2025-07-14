//! Configuration module for `SoftEther` SSL-VPN client
//!
//! This module provides TOML-based configuration parsing and validation
//! for the static library.

use crate::error::{Result, VpnError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Authentication methods supported by `SoftEther` VPN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Password authentication
    #[default]
    Password,
    /// Certificate authentication  
    Certificate,
    /// Anonymous authentication
    Anonymous,
}

/// Server configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server IP address (mandatory)
    pub address: String,
    /// Server hostname for Host header (optional)
    #[serde(default)]
    pub hostname: Option<String>,
    /// Server port (usually 443 for HTTPS)
    pub port: u16,
    /// Hub name to connect to
    pub hub: String,
    /// Use SSL/TLS connection
    #[serde(default = "default_true")]
    pub use_ssl: bool,
    /// Verify server certificate
    #[serde(default = "default_true")]
    pub verify_certificate: bool,
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    /// Keepalive interval in seconds
    #[serde(default = "default_keepalive")]
    pub keepalive_interval: u32,
}

/// Connection limits and pooling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimitsConfig {
    /// Maximum number of concurrent connections (0 = unlimited)
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Connection pooling enabled
    #[serde(default = "default_true")]
    pub enable_pooling: bool,
    /// Pool size for persistent connections
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    /// Connection idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u32,
    /// Maximum connection lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u32,
    /// Enable connection multiplexing
    #[serde(default = "default_false")]
    pub enable_multiplexing: bool,
    /// Maximum multiplexed streams per connection
    #[serde(default = "default_max_streams")]
    pub max_streams_per_connection: u32,
    /// Connection retry attempts
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
    /// Retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u32,
    /// Exponential backoff factor
    #[serde(default = "default_backoff_factor")]
    pub backoff_factor: f64,
    /// Maximum retry delay in seconds
    #[serde(default = "default_max_retry_delay")]
    pub max_retry_delay: u32,
    /// Connection health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval: u32,
    /// Rate limiting: requests per second
    #[serde(default = "default_rate_limit")]
    pub rate_limit_rps: u32,
    /// Rate limiting: burst size
    #[serde(default = "default_burst_size")]
    pub rate_limit_burst: u32,
}

/// Clustering configuration for SSL-VPN RPC farm support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringConfig {
    /// Enable clustering support
    #[serde(default = "default_false")]
    pub enabled: bool,
    /// List of cluster node addresses (hostname:port)
    #[serde(default = "default_cluster_nodes")]
    pub cluster_nodes: Vec<String>,
    /// Load balancing strategy
    #[serde(default = "default_lb_strategy")]
    pub load_balancing_strategy: LoadBalancingStrategy,
    /// Number of connections per cluster node
    #[serde(default = "default_connections_per_node")]
    pub connections_per_node: u32,
    /// Peer count tracking (current active peers)
    #[serde(default = "default_zero")]
    pub current_peer_count: u32,
    /// Maximum peers per cluster
    #[serde(default = "default_max_peers")]
    pub max_peers_per_cluster: u32,
    /// Health check interval for cluster nodes (seconds)
    #[serde(default = "default_cluster_health_interval")]
    pub health_check_interval: u32,
    /// Failover timeout (seconds)
    #[serde(default = "default_failover_timeout")]
    pub failover_timeout: u32,
    /// Enable automatic failover
    #[serde(default = "default_true")]
    pub enable_failover: bool,
    /// RPC farm protocol version
    #[serde(default = "default_rpc_version")]
    pub rpc_protocol_version: String,
    /// Session distribution mode
    #[serde(default = "default_session_distribution")]
    pub session_distribution_mode: SessionDistributionMode,
}

/// Load balancing strategies for cluster nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
    ConsistentHashing,
}

/// Session distribution modes for clustering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionDistributionMode {
    /// Distribute sessions evenly across nodes
    Distributed,
    /// Stick sessions to specific nodes
    Sticky,
    /// Replicate sessions across multiple nodes
    Replicated,
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        LoadBalancingStrategy::RoundRobin
    }
}

impl Default for SessionDistributionMode {
    fn default() -> Self {
        SessionDistributionMode::Distributed
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication method
    #[serde(default)]
    pub method: AuthMethod,
    /// Username for password authentication
    pub username: Option<String>,
    /// Password for password authentication
    pub password: Option<String>,
    /// Client certificate file path
    pub client_cert: Option<String>,
    /// Client private key file path
    pub client_key: Option<String>,
    /// CA certificate file path
    pub ca_cert: Option<String>,
}

/// Network configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable IPv6 support
    #[serde(default = "default_false")]
    pub enable_ipv6: bool,
    /// Bind to specific local address
    pub bind_address: Option<String>,
    /// Use proxy for connections
    pub proxy_url: Option<String>,
    /// User agent string
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    /// Enable HTTP/2 support
    #[serde(default = "default_true")]
    pub enable_http2: bool,
    /// TCP keep-alive enabled
    #[serde(default = "default_true")]
    pub tcp_keepalive: bool,
    /// TCP no-delay enabled
    #[serde(default = "default_true")]
    pub tcp_nodelay: bool,
    /// Socket buffer sizes
    pub socket_buffer_size: Option<u32>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log file path (optional, logs to console if not specified)
    pub file: Option<String>,
    /// Enable JSON logging format
    #[serde(default = "default_false")]
    pub json_format: bool,
    /// Enable colored output
    #[serde(default = "default_true")]
    pub colored: bool,
}

/// Main VPN configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Connection limits and pooling configuration
    #[serde(default)]
    pub connection_limits: ConnectionLimitsConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
    /// Clustering configuration
    #[serde(default)]
    pub clustering: ClusteringConfig,
}

/// Type alias for backward compatibility
pub type VpnConfig = Config;

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| VpnError::Config(format!("Failed to read config file: {e}")))?;

        <Self as FromStr>::from_str(&contents)
    }

    /// Convert configuration to TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| VpnError::Config(format!("Failed to serialize config: {e}")))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.address.is_empty() {
            return Err(VpnError::Config("Server address cannot be empty".into()));
        }

        if self.server.port == 0 {
            return Err(VpnError::Config("Server port must be non-zero".into()));
        }

        if self.server.hub.is_empty() {
            return Err(VpnError::Config("Hub name cannot be empty".into()));
        }

        // Validate authentication configuration
        match self.auth.method {
            AuthMethod::Password => {
                if self.auth.username.is_none() || self.auth.password.is_none() {
                    return Err(VpnError::Config(
                        "Username and password required for password authentication".into(),
                    ));
                }
            }
            AuthMethod::Certificate => {
                if self.auth.client_cert.is_none() || self.auth.client_key.is_none() {
                    return Err(VpnError::Config(
                        "Client certificate and key required for certificate authentication".into(),
                    ));
                }
            }
            AuthMethod::Anonymous => {
                // No additional validation required for anonymous
            }
        }

        // Validate network configuration
        if let Some(ref bind_addr) = self.network.bind_address {
            if bind_addr.parse::<std::net::IpAddr>().is_err() {
                return Err(VpnError::Config(format!(
                    "Invalid bind address: {bind_addr}"
                )));
            }
        }

        // Validate connection limits
        if self.connection_limits.max_connections > 1000 {
            return Err(VpnError::Config(
                "Maximum connections cannot exceed 1000".into(),
            ));
        }

        if self.connection_limits.pool_size > self.connection_limits.max_connections {
            return Err(VpnError::Config(
                "Pool size cannot exceed maximum connections".into(),
            ));
        }

        // Validate clustering configuration
        if self.clustering.enabled {
            if self.clustering.cluster_nodes.is_empty() {
                return Err(VpnError::Config(
                    "Cluster nodes list cannot be empty when clustering is enabled".into(),
                ));
            }

            for node in &self.clustering.cluster_nodes {
                if !node.contains(':') {
                    return Err(VpnError::Config(format!(
                        "Invalid cluster node address: {node}. Expected format: hostname:port"
                    )));
                }
            }
        }

        Ok(())
    }

    /// Create a default configuration for testing
    pub fn default_test() -> Self {
        Self {
            server: ServerConfig {
                address: "127.0.0.1".to_string(),
                hostname: Some("localhost".to_string()),
                port: 443,
                hub: "DEFAULT".to_string(),
                use_ssl: true,
                verify_certificate: false, // Disabled for testing
                timeout: 30,
                keepalive_interval: 60,
            },
            connection_limits: ConnectionLimitsConfig::default(),
            auth: AuthConfig {
                method: AuthMethod::Password,
                username: Some("test".to_string()),
                password: Some("test".to_string()),
                client_cert: None,
                client_key: None,
                ca_cert: None,
            },
            network: NetworkConfig::default(),
            logging: LoggingConfig::default(),
            clustering: ClusteringConfig::default(),
        }
    }
}

impl FromStr for Config {
    type Err = VpnError;

    fn from_str(s: &str) -> Result<Self> {
        let config: Config = toml::from_str(s)
            .map_err(|e| VpnError::Config(format!("Failed to parse TOML config: {e}")))?;

        config.validate()?;
        Ok(config)
    }
}

impl Default for ConnectionLimitsConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            enable_pooling: default_true(),
            pool_size: default_pool_size(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
            enable_multiplexing: default_false(),
            max_streams_per_connection: default_max_streams(),
            retry_attempts: default_retry_attempts(),
            retry_delay: default_retry_delay(),
            backoff_factor: default_backoff_factor(),
            max_retry_delay: default_max_retry_delay(),
            health_check_interval: default_health_check_interval(),
            rate_limit_rps: default_rate_limit(),
            rate_limit_burst: default_burst_size(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enable_ipv6: default_false(),
            bind_address: None,
            proxy_url: None,
            user_agent: default_user_agent(),
            enable_http2: default_true(),
            tcp_keepalive: default_true(),
            tcp_nodelay: default_true(),
            socket_buffer_size: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
            json_format: default_false(),
            colored: default_true(),
        }
    }
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            cluster_nodes: default_cluster_nodes(),
            load_balancing_strategy: default_lb_strategy(),
            connections_per_node: default_connections_per_node(),
            current_peer_count: default_zero(),
            max_peers_per_cluster: default_max_peers(),
            health_check_interval: default_cluster_health_interval(),
            failover_timeout: default_failover_timeout(),
            enable_failover: default_true(),
            rpc_protocol_version: default_rpc_version(),
            session_distribution_mode: default_session_distribution(),
        }
    }
}

// Default value functions
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_timeout() -> u32 { 30 }
fn default_keepalive() -> u32 { 60 }
fn default_max_connections() -> u32 { 10 }
fn default_pool_size() -> u32 { 5 }
fn default_idle_timeout() -> u32 { 300 }
fn default_max_lifetime() -> u32 { 3600 }
fn default_max_streams() -> u32 { 100 }
fn default_retry_attempts() -> u32 { 3 }
fn default_retry_delay() -> u32 { 1000 }
fn default_backoff_factor() -> f64 { 2.0 }
fn default_max_retry_delay() -> u32 { 30 }
fn default_health_check_interval() -> u32 { 30 }
fn default_rate_limit() -> u32 { 100 }
fn default_burst_size() -> u32 { 200 }
fn default_user_agent() -> String { "rVPNSE/0.1.0".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_cluster_nodes() -> Vec<String> { vec!["127.0.0.1:443".to_string()] }
fn default_lb_strategy() -> LoadBalancingStrategy { LoadBalancingStrategy::RoundRobin }
fn default_connections_per_node() -> u32 { 10 }
fn default_zero() -> u32 { 0 }
fn default_max_peers() -> u32 { 100 }
fn default_cluster_health_interval() -> u32 { 30 }
fn default_failover_timeout() -> u32 { 60 }
fn default_rpc_version() -> String { "1.0".to_string() }
fn default_session_distribution() -> SessionDistributionMode { SessionDistributionMode::Distributed }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config_toml = r#"
[server]
address = "62.24.65.211"
hostname = "vpn.example.com"
port = 443
hub = "VPN"
use_ssl = true
verify_certificate = true
timeout = 30
keepalive_interval = 60

[auth]
method = "password"
username = "testuser"
password = "testpass"

[network]
enable_ipv6 = false
user_agent = "TestClient/1.0"
enable_http2 = true
tcp_keepalive = true
tcp_nodelay = true

[logging]
level = "debug"
colored = true
json_format = false
"#;

        let config: Config = config_toml.parse().unwrap();
        assert_eq!(config.server.address, "62.24.65.211");
        assert_eq!(config.server.hostname, Some("vpn.example.com".to_string()));
        assert_eq!(config.server.port, 443);
        assert_eq!(config.auth.method, AuthMethod::Password);
        assert_eq!(config.auth.username, Some("testuser".to_string()));
        assert_eq!(config.network.user_agent, "TestClient/1.0");
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default_test();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Empty address should fail
        config.server.address = String::new();
        assert!(config.validate().is_err());
        
        // Reset address and test zero port
        config.server.address = "127.0.0.1".to_string();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default_test();
        assert_eq!(config.server.address, "127.0.0.1");
        assert_eq!(config.server.hostname, Some("localhost".to_string()));
        assert_eq!(config.server.port, 443);
        assert!(!config.server.verify_certificate); // Disabled for testing
        assert_eq!(config.auth.method, AuthMethod::Password);
        assert_eq!(config.auth.username, Some("test".to_string()));
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config::default_test();
        let toml_str = config.to_toml().unwrap();
        assert!(toml_str.contains("[server]"));
        assert!(toml_str.contains("[auth]"));
        assert!(toml_str.contains("[network]"));
        assert!(toml_str.contains("[logging]"));
        
        // Parse back to ensure round-trip works
        let parsed_config: Config = toml_str.parse().unwrap();
        assert_eq!(config.server.address, parsed_config.server.address);
        assert_eq!(config.server.hostname, parsed_config.server.hostname);
    }
}
