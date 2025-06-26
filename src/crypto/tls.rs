//! TLS/SSL handling for secure connections

use crate::error::Result;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

/// Custom certificate verifier that accepts all certificates (for VPN Gate testing)
#[derive(Debug)]
struct AcceptAllVerifier;

impl rustls::client::danger::ServerCertVerifier for AcceptAllVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Accept all certificates - use only for testing with VPN Gate
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

/// TLS configuration for VPN connections
pub struct TlsConfig {
    client_config: Arc<ClientConfig>,
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new(verify_certificate: bool) -> Result<Self> {
        // Install crypto provider based on feature flags
        // Prioritize ring if both features are enabled (for CI --all-features)
        #[cfg(all(feature = "ring-crypto", not(feature = "aws-lc-crypto")))]
        {
            rustls::crypto::ring::default_provider()
                .install_default()
                .map_err(|_| {
                    crate::error::VpnError::Network("Failed to install ring crypto provider".into())
                })?;
        }

        #[cfg(all(feature = "aws-lc-crypto", not(feature = "ring-crypto")))]
        {
            rustls::crypto::aws_lc_rs::default_provider()
                .install_default()
                .map_err(|_| {
                    crate::error::VpnError::Network(
                        "Failed to install aws-lc-rs crypto provider".into(),
                    )
                })?;
        }

        // If both features are enabled, prefer ring (for CI --all-features)
        #[cfg(all(feature = "ring-crypto", feature = "aws-lc-crypto"))]
        {
            rustls::crypto::ring::default_provider()
                .install_default()
                .map_err(|_| {
                    crate::error::VpnError::Network("Failed to install ring crypto provider".into())
                })?;
        }

        let client_config = if verify_certificate {
            // Use standard certificate verification
            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        } else {
            // Use custom verifier that accepts all certificates (for VPN Gate testing)
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(AcceptAllVerifier))
                .with_no_client_auth()
        };

        Ok(Self {
            client_config: Arc::new(client_config),
        })
    }

    /// Get the client configuration
    pub fn client_config(&self) -> Arc<ClientConfig> {
        self.client_config.clone()
    }

    /// Create TLS configuration with custom certificate
    pub fn with_certificate(cert_path: &str, key_path: &str) -> Result<Self> {
        use rustls_pemfile;
        use std::fs::File;
        use std::io::BufReader;

        let cert_file = File::open(cert_path).map_err(|e| {
            crate::error::VpnError::Config(format!("Cannot open certificate file: {e}"))
        })?;
        let mut cert_reader = BufReader::new(cert_file);

        let key_file = File::open(key_path)
            .map_err(|e| crate::error::VpnError::Config(format!("Cannot open key file: {e}")))?;
        let mut key_reader = BufReader::new(key_file);

        let certs = rustls_pemfile::certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| crate::error::VpnError::Config(format!("Invalid certificate: {e}")))?;

        let private_key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| crate::error::VpnError::Config(format!("Invalid private key: {e}")))?
            .ok_or_else(|| crate::error::VpnError::Config("No private key found".into()))?;

        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let client_config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(certs, private_key)
            .map_err(|e| crate::error::VpnError::Config(format!("TLS config error: {e}")))?;

        Ok(Self {
            client_config: Arc::new(client_config),
        })
    }
}

/// TLS connection wrapper
pub struct TlsConnection {
    stream: Option<StreamOwned<ClientConnection, TcpStream>>,
}

impl TlsConnection {
    /// Create a new TLS connection
    pub fn new(config: &TlsConfig, hostname: &str, port: u16) -> Result<Self> {
        let tcp_stream = TcpStream::connect((hostname, port))
            .map_err(|e| crate::error::VpnError::Network(format!("TCP connection failed: {e}")))?;

        let server_name = ServerName::try_from(hostname)
            .map_err(|e| crate::error::VpnError::Network(format!("Invalid hostname: {e}")))?;

        let conn = ClientConnection::new(config.client_config(), server_name.to_owned())
            .map_err(|e| crate::error::VpnError::Network(format!("TLS connection failed: {e}")))?;

        let stream = StreamOwned::new(conn, tcp_stream);

        Ok(Self {
            stream: Some(stream),
        })
    }

    /// Perform TLS handshake
    pub fn handshake(&mut self) -> Result<()> {
        // Handshake is completed during connection establishment
        Ok(())
    }

    /// Send data over TLS connection
    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        if let Some(ref mut stream) = self.stream {
            stream
                .write(data)
                .map_err(|e| crate::error::VpnError::Network(format!("TLS send failed: {e}")))
        } else {
            Err(crate::error::VpnError::Network(
                "Connection not established".into(),
            ))
        }
    }

    /// Receive data from TLS connection
    pub fn receive(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if let Some(ref mut stream) = self.stream {
            stream
                .read(buffer)
                .map_err(|e| crate::error::VpnError::Network(format!("TLS receive failed: {e}")))
        } else {
            Err(crate::error::VpnError::Network(
                "Connection not established".into(),
            ))
        }
    }

    /// Close TLS connection
    pub fn close(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.flush();
        }
        Ok(())
    }
}
