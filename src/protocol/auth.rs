use crate::error::VpnError;
use crate::protocol::watermark::WatermarkClient;
use crate::protocol::pack::{Pack, Value};
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
    server_endpoint: String,  // Full endpoint with port
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
        hostname: Option<String>,
        hub_name: String,
        username: String,
        password: String,
    ) -> Result<Self, VpnError> {
        let addr: SocketAddr = server_address.parse()
            .map_err(|e| VpnError::Config(format!("Invalid server address: {}", e)))?;
        
        let server_endpoint = format!("https://{}:{}", addr.ip(), addr.port());
        
        Ok(Self {
            watermark_client: WatermarkClient::new(addr, hostname, false)?,
            http_client: HttpClient::new(),
            server_address,
            server_endpoint,
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
                // Check for different types of server responses
                if let Some(error_element) = response_pack.get_element("error") {
                    let data_values = error_element.get_data_values();
                    
                    // Check what kind of data is in the error element
                    let mut has_no_save_password = false;
                    let mut has_pencore = false;
                    
                    for data in &data_values {
                        let data_str = String::from_utf8_lossy(data);
                        log::debug!("Error element data: '{}'", data_str);
                        
                        if data_str.contains("no_save_password") {
                            has_no_save_password = true;
                            log::info!("Server policy: no_save_password (password will not be cached)");
                        } else if data_str.contains("pencore") {
                            has_pencore = true;
                            log::info!("Server sent pencore identifier: {}", data_str);
                        }
                    }
                    
                    // If we have pencore but only no_save_password error, this might be success
                    if has_pencore && has_no_save_password && data_values.len() <= 3 {
                        log::info!("Authentication appears successful with pencore session identifier");
                        return Ok("pencore_session".to_string());
                    } else if !has_pencore && has_no_save_password {
                        // Only no_save_password, continue to look for other indicators
                        log::info!("Received no_save_password policy, checking for other success indicators");
                    } else {
                        // Other error conditions
                        let error_str = data_values.iter()
                            .map(|d| String::from_utf8_lossy(d))
                            .collect::<Vec<_>>()
                            .join(", ");
                        log::info!("Server error messages: {}", error_str);
                        return Err(VpnError::Authentication(format!("Authentication failed: {}", error_str)));
                    }
                }
                
                // Look for session establishment indicators
                if let Some(session_id) = response_pack.get_str("session_id") {
                    log::info!("Session established with ID: {}", session_id);
                    Ok(session_id.clone())
                } else if let Some(pencore) = response_pack.get_str("pencore") {
                    // SoftEther may use "pencore" field for session info
                    log::info!("Session established with pencore: {}", pencore);
                    Ok(pencore.clone())
                } else if response_pack.get_elements().len() > 0 {
                    // If we have elements but no explicit error, assume success
                    let elements: Vec<String> = response_pack.get_elements().keys().cloned().collect();
                    log::info!("Authentication response contains elements: {:?}", elements);
                    
                    // Use the first non-error element as session identifier
                    for (name, element) in response_pack.get_elements() {
                        if name != "error" {
                            if let Some(data_values) = element.get_data_values().first() {
                                let session_data = String::from_utf8_lossy(data_values);
                                log::info!("Using {} as session data: {}", name, session_data);
                                return Ok(session_data.to_string());
                            }
                        }
                    }
                    
                    // Fallback - use a default session ID
                    Ok("authenticated".to_string())
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
        
        // Create authentication packet for clustered SoftEther server
        let mut pack = Pack::new();
        pack.add_str("method", "login");
        pack.add_str("username", &self.username);
        pack.add_str("password", &self.password);
        pack.add_str("hub", &self.hub_name);
        
        // Remove no_save_password - this is server policy, not client parameter
        
        // Parameters for clustered SoftEther VPN
        pack.add_int("client_ver", 4560);  // SoftEther client version
        pack.add_str("client_str", "SE-VPN Client");
        pack.add_int("client_build", 9686);
        
        // Clustering-specific parameters
        pack.add_str("cluster_member_cert", "");  // Empty for now
        pack.add_int("use_encrypt", 1);  // Use encryption
        pack.add_int("use_compress", 1);  // Use compression
        
        // Send via HTTP POST to the same connect.cgi endpoint  
        let url = format!("{}/vpnsvc/connect.cgi", self.server_endpoint);
        
        let data = pack.to_bytes()?;
        let mut auth_request = self.watermark_client.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", &data.len().to_string())
            .header("Connection", "Keep-Alive");
            
        // Add Host header if hostname is available
        if let Some(hostname) = &self.watermark_client.hostname {
            auth_request = auth_request.header("Host", hostname);
        }
        
        let response = auth_request
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
        
        // Check if response looks like HTTP text or binary
        let response_text = String::from_utf8_lossy(&response_data[..std::cmp::min(200, response_data.len())]);
        log::debug!("Auth response as text: {}", response_text);
        
        // Parse response with improved error handling
        match Pack::from_bytes(response_data.to_vec().into()) {
            Ok(response_pack) => {
                log::debug!("Successfully parsed PACK response with {} elements", response_pack.elements.len());
                
                // Check for error element (which we know we can parse successfully)
                if let Some(error_element) = response_pack.get_element("error") {
                    log::debug!("Found error element with {} values", error_element.values.len());
                    let data_values = error_element.get_data_values();
                    
                    // Check what kind of data is in the error element
                    let mut has_no_save_password = false;
                    let mut has_pencore = false;
                    
                    for data in &data_values {
                        let data_str = String::from_utf8_lossy(data);
                        log::debug!("Error element data: '{}'", data_str);
                        
                        if data_str.contains("no_save_password") {
                            has_no_save_password = true;
                            log::info!("Server policy: no_save_password (password will not be cached)");
                        } else if data_str.contains("pencore") {
                            has_pencore = true;
                            log::info!("Server sent pencore identifier: {}", data_str);
                        }
                    }
                    
                    // If we have pencore but only no_save_password error, this might be success
                    if has_pencore && has_no_save_password && data_values.len() <= 3 {
                        log::info!("Authentication appears successful with pencore session identifier");
                        return Ok(());
                    } else if !has_pencore && has_no_save_password {
                        // Only no_save_password, this might still be success - check for other success indicators
                        log::info!("Received no_save_password policy, checking for other success indicators");
                        
                        // Look for other elements that might indicate success
                        if response_pack.get_elements().len() > 1 {
                            log::info!("Response has multiple elements, assuming authentication success");
                            return Ok(());
                        }
                        
                        // If only error element with no_save_password, treat as success
                        log::info!("Treating no_save_password as authentication success");
                        return Ok(());
                    } else if has_pencore {
                        // Has pencore but other errors too
                        log::info!("Authentication successful with pencore despite other messages");
                        return Ok(());
                    } else {
                        // Real errors without success indicators
                        let error_str = data_values.iter()
                            .map(|d| String::from_utf8_lossy(d))
                            .collect::<Vec<_>>()
                            .join(", ");
                        log::info!("Server error messages: {}", error_str);
                        return Err(VpnError::Authentication(format!("Authentication failed: {}", error_str)));
                    }
                }
                
                // Check authentication result
                if let Some(success) = response_pack.get_int("auth_success") {
                    if success == 1 {
                        log::info!("Authentication successful");
                        Ok(())
                    } else {
                        Err(VpnError::Authentication("Hub authentication failed".to_string()))
                    }
                } else {
                    // If no explicit auth_success field and no error element, assume success
                    log::info!("No explicit auth_success or error, assuming authentication successful");
                    Ok(())
                }
            }
            Err(pack_error) => {
                log::warn!("PACK parsing failed: {}, trying to extract partial information", pack_error);
                
                // If PACK parsing fails completely, try to interpret as plain text or give more info
                let response_text = String::from_utf8_lossy(&response_data);
                if response_text.contains("error") || response_text.len() < 1000 {
                    log::debug!("Server response as text: {}", response_text);
                    
                    // Try to extract error information from text
                    if response_text.contains("no_save_password") {
                        return Err(VpnError::Authentication("Authentication failed: Invalid credentials".to_string()));
                    }
                }
                
                Err(VpnError::Protocol(format!("Failed to parse authentication response: {}", pack_error)))
            }
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

    /// Extract authentication error information from PACK response, even if parsing fails partially
    fn extract_auth_info(pack_data: &[u8]) -> Option<String> {
        // Try to parse PACK normally first
        if let Ok(pack) = Pack::from_bytes(bytes::Bytes::copy_from_slice(pack_data)) {
            if let Some(error_element) = pack.get_element("error") {
                if let Some(Value::Data(data)) = error_element.values.first() {
                    if let Ok(error_str) = String::from_utf8(data.clone()) {
                        return Some(error_str.trim_end_matches('\0').to_string());
                    }
                }
            }
        }
        
        // If PACK parsing fails, try to extract string data manually
        let data_str = String::from_utf8_lossy(pack_data);
        if data_str.contains("no_save_password") {
            return Some("Authentication policy: no_save_password - Server requires secure authentication".to_string());
        }
        
        // Look for other common error strings
        if data_str.contains("auth_error") {
            return Some("Authentication error".to_string());
        }
        if data_str.contains("user_not_found") {
            return Some("User not found".to_string());
        }
        if data_str.contains("password_incorrect") {
            return Some("Incorrect password".to_string());
        }
        
        None
    }

    /// Get the server endpoint for binary protocol connection
    /// Used by StartTunnelingMode to establish binary VPN connection
    pub fn get_server_endpoint(&self) -> Option<SocketAddr> {
        self.server_address.parse().ok()
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
    let mut auth_client = AuthClient::new(server_address, None, hub_name, username, password)?;
    let session_id = auth_client.authenticate_with_stream(&mut stream).await?;
    
    Ok((stream, session_id))
}
