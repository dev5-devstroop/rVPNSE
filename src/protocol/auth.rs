//! Authentication handling for `SoftEther` SSL-VPN protocol

use crate::config::Config;
use crate::error::{Result, VpnError};
use std::net::SocketAddr;

/// Authentication methods supported by `SoftEther` VPN
#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    /// Password authentication
    Password,
    /// Certificate authentication  
    Certificate,
    /// Anonymous authentication
    Anonymous,
}

/// `SoftEther` authentication client
///
/// Handles authentication with `SoftEther` SSL-VPN servers.
/// This is a simplified implementation for the static library.
pub struct AuthClient {
    server_addr: SocketAddr,
    #[allow(dead_code)]
    config: Config,
    authenticated: bool,
}

impl AuthClient {
    /// Create new authentication client
    pub fn new(server_addr: SocketAddr, config: &Config) -> Result<Self> {
        Ok(AuthClient {
            server_addr,
            config: config.clone(),
            authenticated: false,
        })
    }

    /// Authenticate with username and password
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        // In a real implementation, this would:
        // 1. Establish TLS connection to server
        // 2. Send HTTP CONNECT request with auth headers
        // 3. Handle `SoftEther`-specific authentication protocol
        // 4. Exchange session keys

        // For the static library, we just validate the parameters
        if username.is_empty() {
            return Err(VpnError::Authentication(
                "Username cannot be empty".to_string(),
            ));
        }

        if password.is_empty() {
            return Err(VpnError::Authentication(
                "Password cannot be empty".to_string(),
            ));
        }

        // Mark as authenticated for static library purposes
        self.authenticated = true;

        Ok(())
    }

    /// Check if client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Get server address
    pub fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
}
