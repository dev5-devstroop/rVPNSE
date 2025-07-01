//! # rVPNSE Client Example
//! 
//! A comprehensive demonstration of the rVPNSE VPN library showcasing:
//! 
//! ## Core Features
//! - SoftEther SSL-VPN protocol implementation
//! - Real VPN server connectivity (VPN Gate compatible)
//! - Multi-method authentication (HTTP CONNECT, form-based)
//! - Session management with automatic keepalives
//! - Tunnel interface management (demonstration mode)
//! - Network connectivity testing and IP detection
//! 
//! ## Architecture
//! - Async networking with Tokio runtime
//! - Configuration-driven setup (TOML)
//! - Comprehensive error handling and logging
//! - Cross-platform compatibility via FFI
//! 
//! ## Usage
//! ```bash
//! cargo run -- --config config.toml --verbose
//! cargo run -- --server custom.vpn.server --timeout 60
//! cargo run -- --skip-tunnel --tunnel-duration 30
//! ```

mod cli;
mod connectivity;
mod demo;

use clap::Parser;
use cli::Args;
use demo::VpnDemo;
use rvpnse::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    args.init_logging();
    
    // Create and run VPN demonstration
    let mut demo = VpnDemo::new(&args).await?;
    demo.run(&args).await?;
    
    Ok(())
}


