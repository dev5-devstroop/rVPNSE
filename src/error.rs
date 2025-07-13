//! Error types and handling for `SoftEther` VPN Rust client

use thiserror::Error;

/// Main error type for VPN operations
#[derive(Error, Debug)]
pub enum VpnError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Configuration validation errors
    #[error("Configuration validation error: {0}")]
    Configuration(String),

    /// Network connectivity errors
    #[error("Network error: {0}")]
    Network(String),

    /// Connection errors
    #[error("Connection failed: {0}")]
    Connection(String),

    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Protocol errors
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Cryptographic errors
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Platform-specific errors
    #[error("Platform error: {0}")]
    Platform(String),

    /// TUN/TAP interface errors
    #[error("TUN/TAP error: {0}")]
    TunTap(String),

    /// Routing errors
    #[error("Routing error: {0}")]
    Routing(String),

    /// DNS configuration errors
    #[error("DNS error: {0}")]
    Dns(String),

    /// Permission/privilege errors
    #[error("Permission error: {0}")]
    Permission(String),

    /// Connection limit errors
    #[error("Connection limit reached: {0}")]
    ConnectionLimitReached(String),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Retry limit errors
    #[error("Retry limit exceeded: {0}")]
    RetryLimitExceeded(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TLS errors
    #[error("TLS error: {0}")]
    Tls(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias for VPN operations
pub type Result<T> = std::result::Result<T, VpnError>;

/// Helper trait for converting errors to VpnError
pub trait IntoVpnError<T> {
    fn into_vpn_error(self, context: &str) -> Result<T>;
}

impl<T, E> IntoVpnError<T> for std::result::Result<T, E>
where
    E: std::fmt::Display,
{
    fn into_vpn_error(self, context: &str) -> Result<T> {
        self.map_err(|e| VpnError::Other(format!("{context}: {e}")))
    }
}

// Implement From for common error types
impl From<toml::de::Error> for VpnError {
    fn from(err: toml::de::Error) -> Self {
        VpnError::Config(format!("TOML parsing error: {err}"))
    }
}

impl From<rustls::Error> for VpnError {
    fn from(err: rustls::Error) -> Self {
        VpnError::Tls(format!("TLS error: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = VpnError::Config("test config error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test config error");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let vpn_err: VpnError = io_err.into();
        assert!(matches!(vpn_err, VpnError::Io(_)));
    }

    #[test]
    fn test_into_vpn_error_trait() {
        let result: std::result::Result<(), &str> = Err("test error");
        let vpn_result = result.into_vpn_error("test context");
        assert!(vpn_result.is_err());
        assert!(vpn_result.unwrap_err().to_string().contains("test context"));
    }
}
