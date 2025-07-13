//! Rust VPNSE - Static Library Framework for ``SoftEther`` SSL-VPN Protocol
//!
//! This is a **static library framework** that provides the foundation for
//! ``SoftEther`` SSL-VPN protocol implementation and integration into any application.
//!
//! ## What This Framework Provides
//! - Configuration parsing and validation (TOML format)
//! - Connection state management and session tracking
//! - Cross-platform error handling system
//! - C FFI bindings for integration with other languages
//! - Platform abstraction layer structure
//! - Example integration patterns
//!
//! ## What Your Application Must Implement
//! - TLS connection implementation (using your preferred TLS library)
//! - ``SoftEther`` SSL-VPN protocol communication
//! - Platform-specific TUN/TAP interface creation
//! - Platform-specific routing management
//! - Platform-specific DNS configuration
//! - Platform-specific permissions/privileges
//!
//! ## Integration Examples
//! See the `examples/` directory for integration patterns and the
//! documentation in `docs/integration/` for platform-specific guides.

pub mod client;
pub mod client_optimized;
pub mod config;
pub mod crypto;
pub mod error;
pub mod protocol;
pub mod tunnel;

// Re-export core types for static library interface
pub use client::{ConnectionStatus, VpnClient};
pub use client_optimized::{OptimizedVpnClient, PerformanceConfig, PerformanceSnapshot};
pub use config::Config;
pub use error::{Result, VpnError};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// C FFI Interface for cross-platform integration
pub mod ffi;
