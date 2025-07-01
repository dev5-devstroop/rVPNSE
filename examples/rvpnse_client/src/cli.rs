//! Command-line interface definitions and argument parsing

use clap::Parser;

#[derive(Parser)]
#[command(name = "rvpnse_client")]
#[command(about = "rVPNSE VPN client - SoftEther SSL-VPN demonstration")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "rVPNSE Team")]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,
    
    /// Server hostname (overrides config)
    #[arg(short, long)]
    pub server: Option<String>,
    
    /// Connection timeout in seconds
    #[arg(short, long, default_value = "30")]
    pub timeout: u64,
    
    /// Tunnel demo duration in seconds
    #[arg(long, default_value = "10")]
    pub tunnel_duration: u64,
    
    /// Skip tunnel demonstration
    #[arg(long)]
    pub skip_tunnel: bool,
    
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Skip connectivity tests
    #[arg(long)]
    pub skip_connectivity: bool,
    
    /// Number of keepalive tests to perform
    #[arg(long, default_value = "5")]
    pub keepalive_count: u32,
}

impl Args {
    /// Initialize logging based on verbosity level
    pub fn init_logging(&self) {
        env_logger::Builder::from_default_env()
            .filter_level(if self.verbose { 
                log::LevelFilter::Debug 
            } else { 
                log::LevelFilter::Info 
            })
            .init();
    }
}
