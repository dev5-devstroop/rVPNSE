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
    /// Server hostname or IP address
    pub hostname: String,
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
    #[serde(default)]
    pub max_concurrent_connections: u32,
    /// Maximum connection attempts per minute (0 = unlimited)
    #[serde(default)]
    pub max_connections_per_minute: u32,
    /// Maximum retry attempts for failed connections
    #[serde(default = "default_max_retries")]
    pub max_retry_attempts: u32,
    /// Delay between retry attempts in seconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u32,
    /// Connection queue size (for pooling)
    #[serde(default = "default_queue_size")]
    pub connection_queue_size: u32,
    /// Enable connection pooling
    #[serde(default)]
    pub enable_connection_pooling: bool,
    /// Pool idle timeout in seconds
    #[serde(default = "default_pool_idle_timeout")]
    pub pool_idle_timeout: u32,
    /// Pool maximum lifetime in seconds
    #[serde(default = "default_pool_max_lifetime")]
    pub pool_max_lifetime: u32,
}

impl Default for ConnectionLimitsConfig {
    fn default() -> Self {
        Self {
            max_concurrent_connections: 0,    // Unlimited by default
            max_connections_per_minute: 0,    // Unlimited by default
            max_retry_attempts: 3,            // Reasonable retry limit
            retry_delay: 5,                   // 5 second delay between retries
            connection_queue_size: 10,        // Queue up to 10 connections
            enable_connection_pooling: false, // Disabled by default for simplicity
            pool_idle_timeout: 300,           // 5 minutes idle timeout
            pool_max_lifetime: 3600,          // 1 hour maximum lifetime
        }
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
    /// Path to client certificate for certificate authentication
    pub certificate_path: Option<String>,
    /// Path to private key for certificate authentication
    pub private_key_path: Option<String>,
    /// Password for certificate file
    pub certificate_password: Option<String>,
}

/// Network configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Whether to automatically configure routes
    #[serde(default)]
    pub auto_route: bool,
    /// Whether to override DNS settings
    #[serde(default)]
    pub dns_override: bool,
    /// DNS server addresses
    #[serde(default)]
    pub dns_servers: Vec<String>,
    /// MTU value
    #[serde(default = "default_mtu")]
    pub mtu: u16,
    /// Local IP address
    pub local_ip: Option<String>,
    /// Custom routes
    #[serde(default)]
    pub custom_routes: Vec<String>,
    /// Excluded routes
    #[serde(default)]
    pub exclude_routes: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Whether to log to a file
    #[serde(default)]
    pub file_logging: bool,
    /// Path to log file
    pub log_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_logging: false,
            log_path: None,
        }
    }
}

/// Main configuration structure
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
}

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
        // Validate server settings
        if self.server.hostname.is_empty() {
            return Err(VpnError::Config(
                "Server hostname cannot be empty".to_string(),
            ));
        }

        if self.server.port == 0 {
            return Err(VpnError::Config("Server port cannot be zero".to_string()));
        }

        if self.server.hub.is_empty() {
            return Err(VpnError::Config("Hub name cannot be empty".to_string()));
        }

        // Validate authentication settings
        match self.auth.method {
            AuthMethod::Password => {
                if self.auth.username.is_none() || self.auth.password.is_none() {
                    return Err(VpnError::Config(
                        "Username and password are required for password authentication"
                            .to_string(),
                    ));
                }
            }
            AuthMethod::Certificate => {
                if self.auth.certificate_path.is_none() || self.auth.private_key_path.is_none() {
                    return Err(VpnError::Config(
                        "Certificate and private key paths are required for certificate authentication".to_string()
                    ));
                }
            }
            AuthMethod::Anonymous => {
                // No additional validation needed
            }
        }

        // Validate network settings
        if self.network.mtu < 576 || self.network.mtu > 9000 {
            return Err(VpnError::Config(
                "MTU must be between 576 and 9000".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a default configuration for VPN Gate
    pub fn default_vpn_gate() -> Self {
        Self {
            server: ServerConfig {
                hostname: "public-vpn-247.opengw.net".to_string(),
                port: 443,
                hub: "VPNGATE".to_string(),
                use_ssl: true,
                verify_certificate: false,
                timeout: 30,
                keepalive_interval: 50,
            },
            auth: AuthConfig {
                method: AuthMethod::Password,
                username: Some("vpn".to_string()),
                password: Some("vpn".to_string()),
                certificate_path: None,
                private_key_path: None,
                certificate_password: None,
            },
            network: NetworkConfig {
                auto_route: false,
                dns_override: false,
                dns_servers: vec![],
                mtu: 1500,
                local_ip: None,
                custom_routes: vec![],
                exclude_routes: vec![],
            },
            logging: LoggingConfig::default(),
            connection_limits: ConnectionLimitsConfig::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_vpn_gate()
    }
}

impl FromStr for Config {
    type Err = VpnError;

    fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| VpnError::Config(format!("Failed to parse TOML: {e}")))
    }
}

// Default value functions for serde
fn default_true() -> bool {
    true
}

fn default_timeout() -> u32 {
    30
}

fn default_keepalive() -> u32 {
    60
}

fn default_mtu() -> u16 {
    1500
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u32 {
    5
}

fn default_queue_size() -> u32 {
    10
}

fn default_pool_idle_timeout() -> u32 {
    300
}

fn default_pool_max_lifetime() -> u32 {
    3600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
[server]
hostname = "vpn.example.com"
port = 443
hub = "VPN"

[auth]
method = "password"
username = "testuser"
password = "testpass"

[network]
auto_route = false
dns_override = false
mtu = 1500

[logging]
level = "info"
"#;

        let config = toml_content
            .parse::<Config>()
            .expect("Failed to parse config");
        assert_eq!(config.server.hostname, "vpn.example.com");
        assert_eq!(config.server.port, 443);
        assert_eq!(config.auth.method, AuthMethod::Password);
        assert_eq!(config.auth.username, Some("testuser".to_string()));
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default_vpn_gate();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid hostname should fail
        config.server.hostname = String::new();
        assert!(config.validate().is_err());
    }
}
