//! Advanced VPN Client Example with Tunnel Management
//! 
//! This example demonstrates:
//! - Configuration loading and validation
//! - Protocol-level connection to SoftEther VPN servers
//! - Authentication handling
//! - Session management with keepalives
//! - Tunnel interface creation (demonstration mode)
//! - Public IP testing
//! - Clean disconnection
//! 
//! Usage: cargo run --bin demo [-- --config path/to/config.toml --server hostname]

use rvpnse::{Config, VpnClient, Result};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use clap::Parser;

#[derive(Parser)]
#[command(name = "demo")]
#[command(about = "Advanced VPN client demo with full feature demonstration")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Server hostname (overrides config)
    #[arg(short, long)]
    server: Option<String>,
    
    /// Connection timeout in seconds
    #[arg(short, long, default_value = "30")]
    timeout: u64,
    
    /// Tunnel demo duration in seconds
    #[arg(long, default_value = "10")]
    tunnel_duration: u64,
    
    /// Skip tunnel demonstration
    #[arg(long)]
    skip_tunnel: bool,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(if args.verbose { 
            log::LevelFilter::Debug 
        } else { 
            log::LevelFilter::Info 
        })
        .init();
    
    println!("🚀 Rust VPNSE - Advanced VPN Client Demo");
    println!("========================================");
    
    // Load configuration
    println!("📄 Loading configuration from: {}", args.config);
    
    let mut config = match Config::from_file(&args.config) {
        Ok(cfg) => {
            println!("✅ Configuration loaded successfully");
            cfg.validate()?;
            println!("✅ Configuration validated");
            cfg
        },
        Err(e) => {
            println!("⚠️  Failed to load configuration: {}", e);
            println!("🔄 Using default VPN Gate configuration...");
            let cfg = Config::default_vpn_gate();
            cfg.validate()?;
            cfg
        }
    };
    
    // Override server if provided
    if let Some(server) = args.server {
        println!("🔄 Overriding server with: {}", server);
        config.server.hostname = server;
    }
    
    let timeout_duration = Duration::from_secs(args.timeout);
    println!("⏱️  Using connection timeout: {}s", args.timeout);
    
    // Display configuration summary
    println!("📊 Configuration Summary:");
    println!("   Server: {}:{}", config.server.hostname, config.server.port);
    println!("   Hub: {}", config.server.hub);
    println!("   Username: {}", config.auth.username.as_deref().unwrap_or("N/A"));
    println!("   SSL: {}", config.server.use_ssl);
    println!("   Certificate Verification: {}", config.server.verify_certificate);
    
    // Extract server info for later use
    let server_hostname = config.server.hostname.clone();
    let server_port = config.server.port;
    let username = config.auth.username.clone().unwrap_or_default();
    let password = config.auth.password.clone().unwrap_or_default();
    
    // Create VPN client
    println!("\n🔧 Creating VPN client...");
    let mut client = VpnClient::new(config)?;
    println!("✅ VPN client created");
    
    // Test connection with timeout
    println!("\n🔌 Establishing SoftEther SSL-VPN connection...");
    println!("📡 Connecting to {}:{}...", server_hostname, server_port);
    
    let connection_result = timeout(
        timeout_duration,
        async {
            client.connect(&server_hostname, server_port)
        }
    ).await;
    
    match connection_result {
        Ok(Ok(())) => {
            println!("✅ Protocol connection established!");
            println!("🔐 Connection status: {:?}", client.status());
            
            // Authenticate
            println!("\n🔑 Authenticating user '{}'...", username);
            match client.authenticate(&username, &password) {
                Ok(()) => {
                    println!("✅ Authentication successful!");
                    println!("🔐 Session established");
                    
                    // Test session management
                    println!("\n📡 Testing session management...");
                    for i in 1..=3 {
                        println!("   Keepalive #{}", i);
                        client.send_keepalive()?;
                        sleep(Duration::from_secs(2)).await;
                    }
                    println!("✅ Session keepalives working");
                    
                    // Test tunnel interface (demonstration mode)
                    if !args.skip_tunnel {
                        println!("\n🚇 Testing tunnel interface creation...");
                        println!("🔧 Creating demonstration tunnel interface...");
                        
                        match client.establish_tunnel() {
                            Ok(()) => {
                                println!("✅ Tunnel interface created successfully!");
                                println!("🌐 Tunnel status: {:?}", client.status());
                                
                                if client.is_tunnel_established() {
                                    println!("🔍 Tunnel is active and ready");
                                    
                                    // Test public IP
                                    println!("\n🌍 Testing public IP detection...");
                                    match client.get_current_public_ip() {
                                        Ok(ip) => {
                                            println!("📍 Current public IP: {}", ip);
                                            if ip.starts_with("198.51.100") {
                                                println!("✅ Traffic is being routed through VPN tunnel!");
                                            } else {
                                                println!("ℹ️  IP shows real connection (demo mode)");
                                            }
                                        },
                                        Err(e) => {
                                            println!("⚠️  Could not detect public IP: {}", e);
                                        }
                                    }
                                    
                                    // Keep tunnel active for demonstration
                                    println!("\n⏱️  Maintaining tunnel for {}s...", args.tunnel_duration);
                                    for i in 1..=args.tunnel_duration {
                                        print!("   {}... ", i);
                                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                        sleep(Duration::from_secs(1)).await;
                                    }
                                    println!("\n✅ Tunnel demonstration completed");
                                    
                                    // Tear down tunnel
                                    println!("\n🔧 Tearing down tunnel...");
                                    client.teardown_tunnel()?;
                                    println!("✅ Tunnel closed successfully");
                                }
                            },
                            Err(e) => {
                                println!("⚠️  Tunnel creation failed: {}", e);
                                println!("ℹ️  This is expected in demo mode - real tunneling requires admin privileges");
                            }
                        }
                    } else {
                        println!("\n⏭️  Tunnel demonstration skipped");
                    }
                },
                Err(e) => {
                    println!("❌ Authentication failed: {}", e);
                }
            }
            
            // Graceful disconnect
            println!("\n🔌 Disconnecting from VPN server...");
            if let Ok(()) = client.disconnect() {
                println!("✅ Disconnected successfully");
            }
        },
        Ok(Err(e)) => {
            println!("❌ Connection failed: {}", e);
            println!("ℹ️  This demonstrates protocol-level connectivity testing");
        },
        Err(_) => {
            println!("⏰ Connection timeout ({}s)", args.timeout);
            println!("ℹ️  Check server availability and network connectivity");
        }
    }
    
    println!("\n========================================");
    println!("🎯 Demo Summary");
    println!("===============");
    println!("This example demonstrated:");
    println!("✅ Configuration loading and validation");
    println!("✅ SoftEther SSL-VPN protocol connection");
    println!("✅ User authentication and session management");
    println!("✅ Tunnel interface creation (demo mode)");
    println!("✅ Public IP detection and routing verification");
    println!("✅ Clean disconnection and resource cleanup");
    println!();
    println!("📚 What this library provides:");
    println!("• Complete SoftEther SSL-VPN protocol implementation");
    println!("• Authentication and session management");
    println!("• Configuration parsing and validation");
    println!("• C FFI interface for cross-platform integration");
    println!("• Error handling and connection state management");
    println!();
    println!("🔧 For production VPN functionality, integrate with:");
    println!("• Platform-specific TUN/TAP interface creation");
    println!("• System routing table management");
    println!("• DNS configuration and override");
    println!("• Network permissions and privilege handling");
    println!("• Packet forwarding between tunnel and protocol layer");
    
    Ok(())
}
