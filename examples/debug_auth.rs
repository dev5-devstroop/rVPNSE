use tokio;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Create a client with local SoftEther server
    let mut client = rvpnse::VpnClient::new("127.0.0.1:5555".to_string()).await?;
    
    // Try to connect
    println!("Attempting to connect...");
    client.connect("test_user", "test_password", "DEFAULT").await?;
    
    println!("Connected successfully!");
    Ok(())
}
