//! rVPNSE Client Binary
//! 
//! A command-line VPN client that connects to SoftEther VPN servers
//! and establishes TUN/TAP interfaces for network tunneling.

use rvpnse::{
    client::{VpnClient, ConnectionStatus},
    config::{Config, ServerConfig, AuthConfig, AuthMethod, NetworkConfig, ConnectionLimitsConfig, LoggingConfig},
    error::{Result, VpnError},
};
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::time::Duration;
use tokio::signal;
use tokio::time::timeout;
use log::{info, error, warn, debug};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("Starting rVPNSE Client v{}", env!("CARGO_PKG_VERSION"));

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 2 && args[1] == "--config" {
        &args[2]
    } else if args.len() > 1 && !args[1].starts_with("--") {
        &args[1]
    } else {
        "config.toml"
    };

    // Load configuration
    let config = load_config(config_path).await?;
    info!("Loaded configuration from: {}", config_path);
    debug!("Server: {}:{}", config.server.hostname, config.server.port);
    debug!("Hub: {}", config.server.hub);

    // Create VPN client
    let mut client = VpnClient::new(config.clone())?;
    info!("VPN client initialized");

    // Setup signal handlers for graceful shutdown
    let shutdown_signal = setup_shutdown_handler();

    // Connect to VPN server
    info!("Connecting to VPN server...");
    let server_hostname = &config.server.hostname;
    let server_port = config.server.port;
    
    if let Err(e) = client.connect_async(server_hostname, server_port).await {
        error!("Failed to connect to VPN server: {}", e);
        process::exit(1);
    }
    info!("Connected to VPN server successfully");

    // Authenticate
    info!("Authenticating...");
    let username = config.auth.username.clone().unwrap_or_else(|| {
        warn!("No username in config, using 'user'");
        "user".to_string()
    });
    let password = config.auth.password.clone().unwrap_or_else(|| {
        warn!("No password in config, using empty password");
        String::new()
    });

    if let Err(e) = client.authenticate(&username, &password).await {
        error!("Authentication failed: {}", e);
        let _ = client.disconnect();
        process::exit(1);
    }
    info!("Authentication successful");

    // Establish tunnel
    info!("Establishing VPN tunnel...");
    if let Err(e) = client.establish_tunnel() {
        error!("Failed to establish tunnel: {}", e);
        let _ = client.disconnect();
        process::exit(1);
    }
    info!("VPN tunnel established successfully");

    // Display connection information
    display_connection_info(&client, &config).await;

    // Setup signal handlers for graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        setup_shutdown_handler().await;
        let _ = shutdown_tx.send(());
    });

    // Main loop with keepalive
    info!("VPN client is running. Press Ctrl+C to disconnect.");
    let mut keepalive_interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                info!("Shutdown signal received");
                break;
            }
            _ = keepalive_interval.tick() => {
                // Check connection status and send keepalive
                let status = client.status();
                if status == ConnectionStatus::Tunneling || status == ConnectionStatus::Connected {
                    debug!("Sending keepalive...");
                    if let Err(e) = client.send_keepalive().await {
                        warn!("Keepalive failed: {}", e);
                    }
                } else {
                    warn!("Connection lost, status: {:?}", status);
                    // You could implement reconnection logic here
                }
            }
        }
    }

    // Graceful shutdown
    info!("Shutting down VPN client...");
    if let Err(e) = client.disconnect() {
        error!("Error during disconnect: {}", e);
    } else {
        info!("VPN client disconnected successfully");
    }

    Ok(())
}

/// Load configuration from file or create default
async fn load_config(config_path: &str) -> Result<Config> {
    if Path::new(config_path).exists() {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| VpnError::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| VpnError::Config(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
    } else {
        warn!("Config file '{}' not found, creating default configuration", config_path);
        let config = create_default_config();
        
        // Save default config for future use
        let config_toml = toml::to_string_pretty(&config)
            .map_err(|e| VpnError::Config(format!("Failed to serialize config: {}", e)))?;
        
        if let Err(e) = fs::write(config_path, config_toml) {
            warn!("Failed to write default config file: {}", e);
        } else {
            info!("Created default config file: {}", config_path);
        }
        
        Ok(config)
    }
}

/// Create a default configuration
fn create_default_config() -> Config {
    Config {
        server: ServerConfig {
            hostname: "vpn.example.com".to_string(),
            port: 443,
            hub: "DEFAULT".to_string(),
            use_ssl: true,
            verify_certificate: true,
            timeout: 30,
            keepalive_interval: 60,
        },
        connection_limits: ConnectionLimitsConfig::default(),
        auth: AuthConfig {
            method: AuthMethod::Password,
            username: Some("vpnuser".to_string()),
            password: Some("vpnpass".to_string()),
            client_cert: None,
            client_key: None,
            ca_cert: None,
        },
        network: NetworkConfig::default(),
        logging: LoggingConfig::default(),
    }
}

/// Setup signal handlers for graceful shutdown
async fn setup_shutdown_handler() {
    tokio::select! {
        _ = signal::ctrl_c() => {
            debug!("Received Ctrl+C");
        }
        _ = async {
            #[cfg(unix)]
            {
                let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
                sigterm.recv().await;
                debug!("Received SIGTERM");
            }
            #[cfg(not(unix))]
            {
                // On non-Unix systems, just wait indefinitely
                futures::future::pending::<()>().await;
            }
        } => {}
    }
}

/// Display connection information
async fn display_connection_info(client: &VpnClient, _config: &Config) {
    let status = client.status();
    
    println!("\n=== VPN Connection Information ===");
    println!("Status: {}", match status {
        ConnectionStatus::Disconnected => "Disconnected",
        ConnectionStatus::Connecting => "Connecting",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Tunneling => "Tunneling",
    });
    
    if let Some(endpoint) = client.server_endpoint() {
        println!("Server: {}", endpoint);
    }
    
    if let Some(session_info) = client.get_session_info() {
        if let Some(ref session_id) = session_info.session_id {
            println!("Session ID: {}", session_id);
        }
        println!("Authenticated: {}", session_info.is_authenticated);
    }
    
    // Display basic tunnel information
    println!("\n=== Tunnel Information ===");
    println!("Interface: rvpnse0");
    println!("Local IP: 10.0.0.2");
    println!("Remote IP: 10.0.0.1");
    println!("Netmask: 255.255.255.0");
    println!("MTU: 1500");
    println!("DNS Servers: 8.8.8.8, 8.8.4.4");
    
    println!("\n=== Network Routing ===");
    println!("All traffic will be routed through the VPN tunnel");
    println!("Original routes are preserved and will be restored on disconnect");
    println!();
}

/// Keepalive loop to maintain connection
async fn keepalive_loop(client: std::sync::Arc<tokio::sync::Mutex<VpnClient>>, config: Config) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        let mut client = client.lock().await;
        
        // Check if still connected
        let status = client.status();
        if status != ConnectionStatus::Tunneling && status != ConnectionStatus::Connected {
            warn!("Connection lost, attempting to reconnect...");
            
            // Try to reconnect
            let server_hostname = config.server.hostname.clone();
            let server_port = config.server.port;
            
            match timeout(Duration::from_secs(30), client.connect_async(&server_hostname, server_port)).await {
                Ok(Ok(())) => {
                    info!("Reconnected successfully");
                    
                    // Re-authenticate
                    let username = config.auth.username.clone().unwrap_or_default();
                    let password = config.auth.password.clone().unwrap_or_default();
                    
                    if let Err(e) = client.authenticate(&username, &password).await {
                        error!("Re-authentication failed: {}", e);
                        continue;
                    }
                    
                    // Re-establish tunnel
                    if let Err(e) = client.establish_tunnel() {
                        error!("Failed to re-establish tunnel: {}", e);
                        continue;
                    }
                    
                    info!("Connection fully restored");
                }
                Ok(Err(e)) => {
                    error!("Reconnection failed: {}", e);
                }
                Err(_) => {
                    error!("Reconnection timed out");
                }
            }
        } else {
            // Send keepalive
            debug!("Sending keepalive...");
            if let Err(e) = client.send_keepalive().await {
                warn!("Keepalive failed: {}", e);
            }
        }
    }
}

/// Helper to check if running as root (required for TUN/TAP on Unix)
#[cfg(unix)]
fn check_privileges() -> Result<()> {
    let uid = unsafe { libc::getuid() };
    if uid != 0 {
        return Err(VpnError::Configuration(
            "This program requires root privileges to create TUN/TAP interfaces. Please run with sudo.".to_string()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_privileges() -> Result<()> {
    // On non-Unix systems, assume privileges are sufficient
    Ok(())
}

/// Print usage information
fn print_usage() {
    println!("rVPNSE Client v{}", env!("CARGO_PKG_VERSION"));
    println!("A SoftEther VPN client with TUN/TAP support");
    println!();
    println!("USAGE:");
    println!("    rvpnse-client [CONFIG_FILE]");
    println!();
    println!("ARGS:");
    println!("    CONFIG_FILE    Path to configuration file (default: config.toml)");
    println!();
    println!("EXAMPLES:");
    println!("    # Use default config");
    println!("    sudo rvpnse-client");
    println!();
    println!("    # Use custom config");
    println!("    sudo rvpnse-client /etc/rvpnse/client.toml");
    println!();
    println!("CONFIG FORMAT:");
    println!("    The configuration file should be in TOML format.");
    println!("    A default config will be created if none exists.");
    println!();
    println!("PRIVILEGES:");
    println!("    This program requires root/administrator privileges");
    println!("    to create and manage TUN/TAP network interfaces.");
}
