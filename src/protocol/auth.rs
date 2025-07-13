use crate::error::VpnError;
use crate::protocol::watermark::WatermarkClient;
use crate::protocol::pack::Pack;
use reqwest::Client as HttpClient;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Authentication client for SoftEther VPN protocol
pub struct AuthClient {
    watermark_client: WatermarkClient,
    http_client: HttpClient,
    server_address: String,
    hub_name: String,
    username: String,
    password: String,
    stream: Option<TcpStream>,
    session_id: Option<String>,
    is_authenticated: bool,
}

impl AuthClient {
    /// Create a new authentication client
    pub fn new(
        server_address: String,
        hub_name: String,
        username: String,
        password: String,
    ) -> Result<Self, VpnError> {
        let addr: SocketAddr = server_address.parse()
            .map_err(|e| VpnError::Config(format!("Invalid server address: {}", e)))?;
        
        Ok(Self {
            watermark_client: WatermarkClient::new(addr, false)?,
            http_client: HttpClient::new(),
            server_address,
            hub_name,
            username,
            password,
            stream: None,
            session_id: None,
            is_authenticated: false,
        })
    }

    /// Internal method for authentication with stream
    async fn authenticate_with_stream(&mut self, stream: &mut TcpStream) -> Result<String, VpnError> {
        // Step 1: HTTP Watermark handshake
        log::info!("Starting HTTP Watermark handshake");
        let _watermark_response = self.watermark_client.send_watermark_handshake().await?;
        
        // Step 2: Authenticate directly (no session establishment needed)
        self.perform_hub_authentication(stream).await?;
        
        Ok("authenticated".to_string())
    }

    /// Establish a session with the server
    async fn establish_session(&self, stream: &mut TcpStream) -> Result<String, VpnError> {
        log::info!("Establishing session with server");
        
        // Create session establishment packet
        let mut pack = Pack::new();
        pack.add_str("method", "admin");
        pack.add_str("hub", &self.hub_name);
        
        // Send via HTTP POST to the same connect.cgi endpoint
        let url = format!("https://{}:{}/vpnsvc/connect.cgi", stream.peer_addr().unwrap().ip(), 443);
        
        let data = pack.to_bytes()?;
        let response = self.watermark_client.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", &data.len().to_string())
            .header("Connection", "Keep-Alive")
            .body(data)
            .send()
            .await
            .map_err(|e| VpnError::Network(format!("Failed to send session request: {}", e)))?;

        if !response.status().is_success() {
            return Err(VpnError::Protocol(format!(
                "Session establishment failed: HTTP {}",
                response.status()
            )));
        }

        let response_data = response.bytes().await
            .map_err(|e| VpnError::Network(format!("Failed to read session response: {}", e)))?;
        
        log::debug!("Session response data length: {}", response_data.len());
        log::debug!("Session response data (first 100 bytes): {:?}", &response_data[..std::cmp::min(100, response_data.len())]);
        
        // Try to parse response, but handle errors gracefully
        match Pack::from_bytes(response_data.to_vec().into()) {
            Ok(response_pack) => {
                // Check for error in the response
                if let Some(error_msg) = response_pack.get_str("error") {
                    return Err(VpnError::Authentication(format!("Server error: {}", error_msg)));
                }
                
                // Extract session ID
                if let Some(session_id) = response_pack.get_str("session_id") {
                    log::info!("Session established with ID: {}", session_id);
                    Ok(session_id.clone())
                } else {
                    Err(VpnError::Authentication("Failed to get session ID from server".to_string()))
                }
            }
            Err(pack_error) => {
                // If PACK parsing fails, try to interpret as plain text or give more info
                let response_text = String::from_utf8_lossy(&response_data);
                if response_text.contains("error") || response_text.len() < 1000 {
                    log::debug!("Server response as text: {}", response_text);
                }
                Err(VpnError::Protocol(format!("Failed to parse session response: {}", pack_error)))
            }
        }
    }

    /// Perform hub authentication
    async fn perform_hub_authentication(&self, stream: &mut TcpStream) -> Result<(), VpnError> {
        log::info!("Authenticating with hub: {}", self.hub_name);
        
        // Create authentication packet
        let mut pack = Pack::new();
        pack.add_str("method", "login");
        pack.add_str("username", &self.username);
        pack.add_str("password", &self.password);
        pack.add_str("hub", &self.hub_name);
        
        // Send via HTTP POST to the same connect.cgi endpoint  
        let url = format!("https://{}:{}/vpnsvc/connect.cgi", stream.peer_addr().unwrap().ip(), 443);
        
        let data = pack.to_bytes()?;
        let response = self.watermark_client.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", &data.len().to_string())
            .header("Connection", "Keep-Alive")
            .body(data)
            .send()
            .await
            .map_err(|e| VpnError::Network(format!("Failed to send auth request: {}", e)))?;

        if !response.status().is_success() {
            return Err(VpnError::Protocol(format!(
                "Hub authentication failed: HTTP {}",
                response.status()
            )));
        }

        let response_data = response.bytes().await
            .map_err(|e| VpnError::Network(format!("Failed to read auth response: {}", e)))?;
        
        log::debug!("Auth response data length: {}", response_data.len());
        log::debug!("Auth response data (first 100 bytes): {:?}", &response_data[..std::cmp::min(100, response_data.len())]);
        
        // Parse response
        let response_pack = Pack::from_bytes(response_data.to_vec().into())?;
        
        // Check authentication result
        if let Some(success) = response_pack.get_int("auth_success") {
            if success == 1 {
                log::info!("Authentication successful");
                Ok(())
            } else {
                Err(VpnError::Authentication("Hub authentication failed".to_string()))
            }
        } else {
            Err(VpnError::Authentication("Invalid authentication response from server".to_string()))
        }
    }

    /// Get the configured server address
    pub fn server_address(&self) -> &str {
        &self.server_address
    }

    /// Get the configured hub name
    pub fn hub_name(&self) -> &str {
        &self.hub_name
    }

    /// Get the configured username
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Connect to the server and perform authentication
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<(), VpnError> {
        // Update credentials if provided
        if !username.is_empty() {
            self.username = username.to_string();
        }
        if !password.is_empty() {
            self.password = password.to_string();
        }

        // Connect to server if not already connected
        if self.stream.is_none() {
            let stream = TcpStream::connect(&self.server_address).await
                .map_err(|e| VpnError::Network(format!("Failed to connect to server: {}", e)))?;
            self.stream = Some(stream);
        }

        // Perform the full authentication flow
        if let Some(mut stream) = self.stream.take() {
            let session_id = self.authenticate_with_stream(&mut stream).await?;
            self.session_id = Some(session_id);
            self.is_authenticated = true;
            self.stream = Some(stream);
        }

        Ok(())
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    /// Send keepalive (placeholder for compatibility)
    pub async fn send_keepalive(&mut self) -> Result<(), VpnError> {
        // TODO: Implement proper keepalive for SoftEther protocol
        log::debug!("Keepalive sent (placeholder)");
        Ok(())
    }
}

/// Convenience function to create an authenticated connection
pub async fn authenticate_connection(
    server_address: String,
    hub_name: String,
    username: String,
    password: String,
) -> Result<(TcpStream, String), VpnError> {
    // Connect to server
    let mut stream = TcpStream::connect(&server_address).await
        .map_err(|e| VpnError::Network(format!("Failed to connect to server: {}", e)))?;
    
    // Create auth client and authenticate
    let mut auth_client = AuthClient::new(server_address, hub_name, username, password)?;
    let session_id = auth_client.authenticate_with_stream(&mut stream).await?;
    
    Ok((stream, session_id))
}
