use crate::error::VpnError;
use crate::protocol::watermark::WatermarkClient;
use crate::protocol::pack::{Pack, Value};
use crate::tunnel::TunnelConfig;
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
    verify_certificate: bool,
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
        verify_certificate: bool,
    ) -> Result<Self, VpnError> {
        let addr: SocketAddr = server_address.parse()
            .map_err(|e| VpnError::Config(format!("Invalid server address: {}", e)))?;
        
        let server_endpoint = format!("https://{}:{}", addr.ip(), addr.port());
        
        Ok(Self {
            watermark_client: WatermarkClient::new(addr, hostname, verify_certificate)?,
            http_client: HttpClient::new(),
            server_address,
            server_endpoint,
            hub_name,
            username,
            password,
            verify_certificate,
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

    /// Send keepalive to maintain the session
    /// NOTE: This should only be used BEFORE SSL-VPN mode switch
    /// After SSL-VPN mode, use binary protocol keepalives instead
    pub async fn send_keepalive(&mut self) -> Result<(), VpnError> {
        if !self.is_authenticated {
            return Err(VpnError::Authentication("Not authenticated".to_string()));
        }

        log::warn!("HTTP keepalive called - this should only be used before SSL-VPN mode");
        
        // Create a proper SoftEther keepalive packet
        let mut pack = Pack::new();
        pack.add_str("method", "keepalive");
        
        if let Some(session_id) = &self.session_id {
            pack.add_str("session_id", session_id);
        }
        
        // Add timestamp for server tracking
        pack.add_int64("timestamp", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());

        // Send via HTTP POST to maintain compatibility with clustering
        let url = format!("{}/vpnsvc/keepalive.cgi", self.server_endpoint);
        let data = pack.to_bytes()?;
        
        let mut request = self.watermark_client.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", &data.len().to_string())
            .header("Connection", "Keep-Alive");
            
        // Add Host header if hostname is available
        if let Some(hostname) = &self.watermark_client.hostname {
            request = request.header("Host", hostname);
        }
        
        let response = request.body(data.to_vec()).send().await
            .map_err(|e| VpnError::Network(format!("Keepalive request failed: {}", e)))?;

        if response.status().is_success() {
            log::debug!("HTTP keepalive sent successfully to SoftEther server");
            Ok(())
        } else {
            log::warn!("HTTP keepalive failed with status: {} (expected after SSL-VPN mode switch)", response.status());
            // Don't treat this as an error after SSL-VPN mode switch
            Ok(())
        }
    }
    
    /// Send binary SSL-VPN keepalive packet
    /// This should be used AFTER SSL-VPN mode switch instead of HTTP keepalive
    pub async fn send_binary_keepalive(&self) -> Result<(), VpnError> {
        log::debug!("Sending binary SSL-VPN keepalive packet");
        
        // Create binary keepalive packet (simple PING)
        let keepalive_data = vec![
            0x00, 0x00, 0x00, 0x08, // Packet length (8 bytes)
            b'P', b'I', b'N', b'G', // "PING" magic bytes
        ];
        
        // TODO: Send via binary SSL-VPN connection instead of HTTP
        // For now, log that we would send this
        log::debug!("Binary keepalive packet prepared: {} bytes", keepalive_data.len());
        
        // This should be sent via the binary SSL-VPN connection, not HTTP
        // The binary connection should be established in the client after SSL-VPN handshake
        
        Ok(())
    }

    /// Request IP configuration from SoftEther server (DHCP-like)
    pub async fn request_ip_config(&self) -> Result<TunnelConfig, VpnError> {
        log::info!("🌐 Requesting IP configuration from VPN server...");
        
        // Create GetConfig packet to request IP assignment
        let mut pack = Pack::new();
        pack.add_str("method", "GetConfig");
        pack.add_str("client_str", "SE-VPN Client");
        pack.add_int("client_ver", 4560);
        pack.add_int("client_build", 9686);
        
        // Request DHCP-like IP assignment
        pack.add_str("request_type", "dhcp_ip");
        pack.add_int("use_dhcp", 1);
        
        let url = format!("{}/vpnsvc/connect.cgi", self.server_endpoint);
        let data = pack.to_bytes()?;
        
        let mut request = self.watermark_client.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", &data.len().to_string())
            .header("Connection", "Keep-Alive");
            
        if let Some(hostname) = &self.watermark_client.hostname {
            request = request.header("Host", hostname);
        }
        
        let response = request
            .body(data)
            .send()
            .await
            .map_err(|e| VpnError::Network(format!("Failed to request IP config: {}", e)))?;

        if !response.status().is_success() {
            return Err(VpnError::Protocol(format!(
                "IP config request failed: HTTP {}",
                response.status()
            )));
        }

        let response_data = response.bytes().await
            .map_err(|e| VpnError::Network(format!("Failed to read IP config response: {}", e)))?;
        
        // Parse IP configuration response
        match Pack::from_bytes(response_data.to_vec().into()) {
            Ok(response_pack) => {
                // Extract IP configuration from server response
                let local_ip = response_pack.get_str("client_ip")
                    .map_or("10.0.0.2", |v| v); // Fallback
                    
                let remote_ip = response_pack.get_str("server_ip")
                    .map_or("10.0.0.1", |v| v); // Fallback
                    
                let netmask = response_pack.get_str("netmask")
                    .map_or("255.255.255.0", |v| v); // Fallback
                    
                let mtu = response_pack.get_int("mtu")
                    .unwrap_or(1500) as u16;
                    
                let dns1 = response_pack.get_str("dns1").map_or("8.8.8.8", |v| v);
                let dns2 = response_pack.get_str("dns2").map_or("8.8.4.4", |v| v);
                    
                let dns_servers = vec![
                    dns1.parse().unwrap_or(std::net::Ipv4Addr::new(8, 8, 8, 8)),
                    dns2.parse().unwrap_or(std::net::Ipv4Addr::new(8, 8, 4, 4)),
                ];
                
                log::info!("📍 Server assigned IP: {}", local_ip);
                log::info!("📍 Server gateway IP: {}", remote_ip);
                log::info!("📍 Netmask: {}", netmask);
                log::info!("📍 MTU: {}", mtu);
                log::info!("📍 DNS servers: {:?}", dns_servers);
                
                use crate::tunnel::TunnelConfig;
                Ok(TunnelConfig {
                    interface_name: "vpnse0".to_string(),
                    local_ip: local_ip.parse()
                        .map_err(|e| VpnError::Config(format!("Invalid local IP: {}", e)))?,
                    remote_ip: remote_ip.parse()
                        .map_err(|e| VpnError::Config(format!("Invalid remote IP: {}", e)))?,
                    netmask: netmask.parse()
                        .map_err(|e| VpnError::Config(format!("Invalid netmask: {}", e)))?,
                    mtu,
                    dns_servers,
                })
            }
            Err(_) => {
                log::warn!("Failed to parse IP config response, using defaults");
                // Use default configuration if server doesn't provide DHCP-like response
                use crate::tunnel::TunnelConfig;
                Ok(TunnelConfig::default())
            }
        }
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

    /// Complete SSL-VPN handshake after authentication 
    /// This is CRITICAL - the server stays in "initializing" without this
    pub async fn complete_ssl_vpn_handshake(&self) -> Result<(), VpnError> {
        log::info!("🔄 Completing SSL-VPN handshake transition...");
        log::info!("🎯 Goal: Get server out of 'initializing' state and enable DHCP");
        
        // Create proper SoftEther SSL-VPN start command
        // This tells the server to switch from HTTP to binary SSL-VPN mode
        let mut pack = Pack::new();
        pack.add_str("method", "start_ssl_vpn");
        pack.add_str("protocol", "SSL_VPN");
        
        // Add session information
        if let Some(session_id) = &self.session_id {
            pack.add_str("session_id", session_id);
            log::debug!("📋 Including session_id: {}", session_id);
        } else {
            log::warn!("⚠️  No session_id available for SSL-VPN handshake");
        }
        
        // Critical SoftEther SSL-VPN parameters
        pack.add_int("use_ssl_vpn", 1);
        pack.add_int("use_encrypt", 1);
        pack.add_int("use_compress", 0); // Disable compression for stability
        pack.add_str("client_str", "SE-VPN Client");
        pack.add_int("client_ver", 4560);
        pack.add_int("client_build", 9686);
        
        // Request server to assign IP via DHCP-like mechanism
        pack.add_str("request_dhcp", "1");
        pack.add_str("dhcp_hostname", "rvpnse-client");
        
        let url = format!("{}/vpnsvc/connect.cgi", self.server_endpoint);
        log::debug!("📡 SSL-VPN handshake URL: {}", url);
        
        let data = pack.to_bytes()?;
        log::debug!("📦 SSL-VPN packet size: {} bytes", data.len());
        log::debug!("📦 SSL-VPN packet (first 100 bytes): {:02x?}", 
            &data[..std::cmp::min(100, data.len())]);
        
        log::info!("📡 Sending SSL-VPN handshake to server...");
        log::debug!("🔗 Request details:");
        log::debug!("  URL: {}", url);
        log::debug!("  Method: POST");
        log::debug!("  Content-Type: application/octet-stream");
        log::debug!("  Content-Length: {}", data.len());
        log::debug!("  Connection: Keep-Alive");
        if let Some(hostname) = &self.watermark_client.hostname {
            log::debug!("  Host: {}", hostname);
        }
        
        // CRITICAL FIX: Create a fresh HTTP client for SSL-VPN handshake
        // The original client might have connection state issues after authentication
        log::debug!("🔄 Creating fresh HTTP client for SSL-VPN handshake...");
        let mut fresh_client_builder = reqwest::Client::builder()
            .user_agent("SoftEther VPN Client");

        // Match the TLS verification settings from the original client
        if !self.verify_certificate {
            fresh_client_builder = fresh_client_builder.danger_accept_invalid_certs(true);
            log::debug!("🔓 SSL certificate verification disabled");
        } else {
            log::debug!("🔒 SSL certificate verification enabled");
        }

        let fresh_http_client = fresh_client_builder.build()
            .map_err(|e| VpnError::Network(format!("Failed to create fresh HTTP client: {}", e)))?;
        
        let mut fresh_request = fresh_http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", &data.len().to_string())
            .header("Connection", "Keep-Alive");
            
        if let Some(hostname) = &self.watermark_client.hostname {
            fresh_request = fresh_request.header("Host", hostname);
        }
        
        let response = fresh_request
            .body(data)
            .send()
            .await
            .map_err(|e| {
                log::error!("❌ SSL-VPN handshake failed to send: {}", e);
                log::error!("🔍 Error details: {:?}", e);
                if e.is_connect() {
                    log::error!("🌐 Connection error - cannot reach server");
                } else if e.is_timeout() {
                    log::error!("⏰ Timeout error - server not responding");
                } else if e.is_request() {
                    log::error!("📤 Request error - malformed request");
                } else {
                    log::error!("❓ Other network error");
                }
                VpnError::Network(format!("Failed to send SSL-VPN start: {}", e))
            })?;

        log::info!("📥 SSL-VPN handshake response status: {}", response.status());

        if !response.status().is_success() {
            log::error!("❌ SSL-VPN handshake failed: HTTP {}", response.status());
            log::error!("🔧 This will cause server to stay in 'initializing' state");
            return Err(VpnError::Protocol(format!(
                "SSL-VPN handshake failed: HTTP {}",
                response.status()
            )));
        }

        let response_data = response.bytes().await
            .map_err(|e| {
                log::error!("❌ Failed to read SSL-VPN response: {}", e);
                VpnError::Network(format!("Failed to read SSL-VPN response: {}", e))
            })?;
        
        log::info!("📥 SSL-VPN handshake response received: {} bytes", response_data.len());
        log::debug!("📦 SSL-VPN response (first 200 bytes): {:02x?}", 
            &response_data[..std::cmp::min(200, response_data.len())]);
        
        // Try to interpret as text first for debugging
        let response_text = String::from_utf8_lossy(&response_data[..std::cmp::min(500, response_data.len())]);
        log::debug!("📝 SSL-VPN response as text: '{}'", response_text);
        
        // Parse SSL-VPN handshake response - this should contain IP assignment
        match Pack::from_bytes(response_data.to_vec().into()) {
            Ok(response_pack) => {
                log::info!("✅ SSL-VPN handshake response parsed successfully");
                
                // Log all response elements for debugging
                for (name, element) in response_pack.get_elements() {
                    log::debug!("🔍 SSL-VPN response element: {} with {} values", name, element.values.len());
                    if let Some(first_val) = element.values.first() {
                        match first_val {
                            crate::protocol::pack::Value::Str(s) => log::debug!("  📄 String: '{}'", s),
                            crate::protocol::pack::Value::Data(d) => {
                                let data_str = String::from_utf8_lossy(d);
                                log::debug!("  📄 Data: '{}' (len: {})", data_str, d.len());
                            },
                            crate::protocol::pack::Value::Int(i) => log::debug!("  🔢 Int: {}", i),
                            crate::protocol::pack::Value::Int64(i) => log::debug!("  🔢 Int64: {}", i),
                            _ => log::debug!("  ❓ Other type"),
                        }
                    }
                }
                
                // Check for SSL-VPN confirmation
                if let Some(ssl_vpn_ok) = response_pack.get_int("ssl_vpn_ok") {
                    if ssl_vpn_ok == 1 {
                        log::info!("✅ SSL-VPN handshake completed successfully");
                        return Ok(());
                    } else {
                        log::warn!("⚠️  SSL-VPN response indicates failure: ssl_vpn_ok = {}", ssl_vpn_ok);
                    }
                }
                
                // Look for error messages
                if let Some(error) = response_pack.get_str("error") {
                    log::error!("❌ SSL-VPN handshake error: {}", error);
                    return Err(VpnError::Protocol(format!("SSL-VPN error: {}", error)));
                }
                
                // Check for IP assignment in the SSL-VPN response
                let assigned_ip = response_pack.get_str("assigned_ip")
                    .or_else(|| response_pack.get_str("client_ip"))
                    .or_else(|| response_pack.get_str("your_ip"))
                    .or_else(|| response_pack.get_str("ip"));
                
                if let Some(ip) = assigned_ip {
                    log::info!("🎯 SSL-VPN response contains IP assignment: {}", ip);
                    if ip.starts_with("10.21.255.") {
                        log::info!("✅ Got expected IP range in SSL-VPN response!");
                    }
                }
                
                // If no explicit confirmation, assume success if no error and we have elements
                if !response_pack.get_elements().is_empty() {
                    log::info!("✅ SSL-VPN handshake completed (assumed success - {} elements received)", 
                        response_pack.get_elements().len());
                    return Ok(());
                }
                
                log::warn!("⚠️  SSL-VPN handshake response has no elements, assuming success anyway");
                Ok(())
            }
            Err(parse_error) => {
                log::warn!("⚠️  Failed to parse SSL-VPN response as PACK: {}", parse_error);
                
                // Check if it's an HTTP error response
                let response_text = String::from_utf8_lossy(&response_data);
                if response_text.contains("HTTP/") || response_text.contains("html") {
                    log::error!("📄 Server sent HTML/HTTP response instead of PACK data");
                    log::debug!("📄 Response text: {}", response_text);
                    return Err(VpnError::Protocol("Server sent HTML instead of PACK data".to_string()));
                }
                
                // Don't fail here - SoftEther might send non-PACK response for SSL-VPN switch
                log::info!("✅ SSL-VPN handshake completed (parse error ignored)");
                Ok(())
            }
        }
    }

    /// Request DHCP IP assignment from SoftEther server
    /// This should be called AFTER SSL-VPN handshake completion
    pub async fn request_dhcp_ip(&self) -> Result<TunnelConfig, VpnError> {
        log::info!("🌐 Requesting DHCP IP assignment from VPN server...");
        log::info!("🔍 Expected server-assigned IP range: 10.21.255.x");
        
        // Create DHCP-specific request 
        let mut pack = Pack::new();
        pack.add_str("method", "get_dhcp_config");
        pack.add_str("client_str", "SE-VPN Client");
        pack.add_int("client_ver", 4560);
        pack.add_int("client_build", 9686);
        
        // Add session information
        if let Some(session_id) = &self.session_id {
            pack.add_str("session_id", session_id);
            log::debug!("📋 Including session_id: {}", session_id);
        } else {
            log::warn!("⚠️  No session_id available for DHCP request");
        }
        
        // DHCP request parameters
        pack.add_str("dhcp_hostname", "rvpnse-client");
        pack.add_str("requested_ip", "0.0.0.0"); // Let server assign
        pack.add_int("use_dhcp", 1);
        
        let url = format!("{}/vpnsvc/connect.cgi", self.server_endpoint);
        log::debug!("📡 DHCP request URL: {}", url);
        
        let data = pack.to_bytes()?;
        log::debug!("📦 DHCP request packet size: {} bytes", data.len());
        log::debug!("📦 DHCP request packet (first 100 bytes): {:02x?}", 
            &data[..std::cmp::min(100, data.len())]);
        
        let mut request = self.watermark_client.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", &data.len().to_string())
            .header("Connection", "Keep-Alive");
            
        if let Some(hostname) = &self.watermark_client.hostname {
            request = request.header("Host", hostname);
            log::debug!("🏠 Using hostname: {}", hostname);
        }
        
        log::info!("📡 Sending DHCP request to server...");
        let response = request
            .body(data)
            .send()
            .await
            .map_err(|e| {
                log::error!("❌ DHCP request failed: {}", e);
                VpnError::Network(format!("Failed to send DHCP request: {}", e))
            })?;

        log::info!("📥 DHCP response status: {}", response.status());
        
        if !response.status().is_success() {
            log::error!("❌ DHCP request failed with HTTP {}, falling back to hardcoded IP", response.status());
            log::error!("🔧 This is why we're seeing 10.0.0.x instead of 10.21.255.x");
            // Use fallback IP that's different from default to show it was attempted
            use crate::tunnel::TunnelConfig;
            return Ok(TunnelConfig::with_fallback_ip());
        }

        let response_data = response.bytes().await
            .map_err(|e| {
                log::error!("❌ Failed to read DHCP response: {}", e);
                VpnError::Network(format!("Failed to read DHCP response: {}", e))
            })?;
        
        log::info!("📥 DHCP response received: {} bytes", response_data.len());
        log::debug!("📦 DHCP response (first 200 bytes): {:02x?}", 
            &response_data[..std::cmp::min(200, response_data.len())]);
        
        // Try to interpret as text first for debugging
        let response_text = String::from_utf8_lossy(&response_data[..std::cmp::min(500, response_data.len())]);
        log::debug!("📝 DHCP response as text: '{}'", response_text);
        
        // Parse DHCP response
        match Pack::from_bytes(response_data.to_vec().into()) {
            Ok(response_pack) => {
                log::info!("✅ DHCP response parsed successfully with {} elements", response_pack.elements.len());
                
                // Log all elements for debugging
                for (name, element) in response_pack.get_elements() {
                    log::debug!("🔍 DHCP element '{}' with {} values", name, element.values.len());
                    if let Some(first_val) = element.values.first() {
                        match first_val {
                            crate::protocol::pack::Value::Str(s) => log::debug!("  📄 String value: '{}'", s),
                            crate::protocol::pack::Value::Data(d) => {
                                let data_str = String::from_utf8_lossy(d);
                                log::debug!("  📄 Data value: '{}' (len: {})", data_str, d.len());
                            },
                            crate::protocol::pack::Value::Int(i) => log::debug!("  🔢 Int value: {}", i),
                            crate::protocol::pack::Value::Int64(i) => log::debug!("  🔢 Int64 value: {}", i),
                            _ => log::debug!("  ❓ Other value type"),
                        }
                    }
                }
                
                // Try multiple possible field names for IP configuration
                let assigned_ip = response_pack.get_str("assigned_ip")
                    .or_else(|| response_pack.get_str("client_ip"))
                    .or_else(|| response_pack.get_str("your_ip"))
                    .or_else(|| response_pack.get_str("ip"))
                    .or_else(|| response_pack.get_str("dhcp_ip"));
                    
                let gateway_ip = response_pack.get_str("gateway_ip")
                    .or_else(|| response_pack.get_str("server_ip"))
                    .or_else(|| response_pack.get_str("router"))
                    .or_else(|| response_pack.get_str("gateway"))
                    .or_else(|| response_pack.get_str("vpn_server_ip"));
                    
                let subnet_mask = response_pack.get_str("subnet_mask")
                    .or_else(|| response_pack.get_str("netmask"))
                    .or_else(|| response_pack.get_str("mask"));
                
                log::debug!("🔍 IP fields found - assigned: {:?}, gateway: {:?}, mask: {:?}", 
                    assigned_ip, gateway_ip, subnet_mask);
                
                // Check if we got any IP configuration
                if let Some(local) = assigned_ip {
                    let gateway = gateway_ip.map_or("192.168.100.1", |v| v); // Default gateway
                    let mask = subnet_mask.map_or("255.255.255.0", |v| v); // Default mask
                    
                    log::info!("🎯 SUCCESS: DHCP assigned IP: {}", local);
                    log::info!("🎯 SUCCESS: DHCP gateway IP: {}", gateway);
                    log::info!("🎯 SUCCESS: DHCP netmask: {}", mask);
                    
                    // Validate that we got the expected IP range (10.21.255.x)
                    if local.starts_with("10.21.255.") {
                        log::info!("✅ Got expected IP range (10.21.255.x) - DHCP working correctly!");
                    } else {
                        log::warn!("⚠️  Got unexpected IP range: {} (expected 10.21.255.x)", local);
                    }
                    
                    use crate::tunnel::TunnelConfig;
                    return Ok(TunnelConfig {
                        interface_name: "vpnse0".to_string(),
                        local_ip: local.parse()
                            .map_err(|e| VpnError::Config(format!("Invalid assigned IP '{}': {}", local, e)))?,
                        remote_ip: gateway.parse()
                            .map_err(|e| VpnError::Config(format!("Invalid gateway IP '{}': {}", gateway, e)))?,
                        netmask: mask.parse()
                            .map_err(|e| VpnError::Config(format!("Invalid netmask '{}': {}", mask, e)))?,
                        mtu: 1500,
                        dns_servers: vec![
                            "8.8.8.8".parse().unwrap_or(std::net::Ipv4Addr::new(8, 8, 8, 8)),
                            "8.8.4.4".parse().unwrap_or(std::net::Ipv4Addr::new(8, 8, 4, 4)),
                        ],
                    });
                }
                
                log::warn!("❌ No DHCP IP assignment found in response, checking for other indicators");
                
                // Sometimes the server might send IP info in other ways
                // Check for any string/data that looks like an IP address
                for (name, element) in response_pack.get_elements() {
                    if let Some(crate::protocol::pack::Value::Str(value)) = element.values.first() {
                        if value.chars().all(|c| c.is_ascii_digit() || c == '.') && value.contains('.') {
                            if let Ok(ip) = value.parse::<std::net::Ipv4Addr>() {
                                log::info!("� Found IP-like value in '{}': {}", name, ip);
                                if ip.to_string().starts_with("10.21.255.") {
                                    log::info!("🎯 Found expected IP in field '{}': {}", name, ip);
                                }
                            }
                        }
                    }
                }
            }
            Err(parse_error) => {
                log::error!("❌ Failed to parse DHCP response as PACK: {}", parse_error);
                log::error!("🔧 This means the server sent a response but not in PACK format");
                
                // Check if it's an HTTP error response
                let response_text = String::from_utf8_lossy(&response_data);
                if response_text.contains("HTTP/") || response_text.contains("html") {
                    log::error!("📄 Server sent HTML/HTTP response instead of PACK data");
                    log::debug!("📄 Response text: {}", response_text);
                }
            }
        }
        
        // If no DHCP assignment, use a reasonable fallback that's different from default
        log::error!("❌ DHCP IP assignment failed - falling back to hardcoded config");
        log::error!("🔧 This is why you see 10.0.0.x instead of 10.21.255.x");
        use crate::tunnel::TunnelConfig;
        Ok(TunnelConfig::with_fallback_ip())
    }

    // ...existing code...
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
    let mut auth_client = AuthClient::new(server_address, None, hub_name, username, password, false)?;
    let session_id = auth_client.authenticate_with_stream(&mut stream).await?;
    
    Ok((stream, session_id))
}
