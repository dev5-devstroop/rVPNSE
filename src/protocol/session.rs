//! Session management for `SoftEther` SSL-VPN protocol

use crate::config::Config;
use crate::error::{Result, VpnError};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Session manager for `SoftEther` VPN connections
///
/// Manages VPN session state and keepalive for the static library.
pub struct SessionManager {
    session_id: Option<Uuid>,
    start_time: Option<Instant>,
    last_keepalive: Option<Instant>,
    #[allow(dead_code)]
    config: Config,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            session_id: None,
            start_time: None,
            last_keepalive: None,
            config: config.clone(),
        })
    }

    /// Start a new VPN session
    pub fn start_session(&mut self) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        self.session_id = Some(session_id);
        self.start_time = Some(Instant::now());
        self.last_keepalive = Some(Instant::now());

        Ok(session_id)
    }

    /// Send keepalive packet
    pub fn send_keepalive(&mut self) -> Result<()> {
        if self.session_id.is_none() {
            return Err(VpnError::Connection("No active session".to_string()));
        }

        // In a real implementation, this would send a keepalive packet
        // to the `SoftEther` server to maintain the session

        self.last_keepalive = Some(Instant::now());
        Ok(())
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.session_id.is_some()
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<Uuid> {
        self.session_id
    }

    /// Get session duration
    pub fn session_duration(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Get time since last keepalive
    pub fn time_since_keepalive(&self) -> Option<Duration> {
        self.last_keepalive.map(|last| last.elapsed())
    }

    /// End the session
    pub fn end_session(&mut self) {
        self.session_id = None;
        self.start_time = None;
        self.last_keepalive = None;
    }
}
