//! Data types for the bandwidth client.

use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;
use tracing::warn;

/// Bandwidth event received from server
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BandwidthEvent {
    pub timestamp: u64,
    pub download_bps: u64,
    pub upload_bps: u64,
    pub class: String,
}

/// Configuration for the QUIC-based bandwidth client
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BandwidthClientConfig {
    /// Server address to connect to (e.g., "127.0.0.1:4433")
    pub server_addr: String,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Maximum time to wait for server responses in milliseconds  
    pub response_timeout_ms: u64,
    /// Enable automatic reconnection on connection loss
    pub auto_reconnect: bool,
    /// Delay between reconnection attempts in milliseconds
    pub reconnect_delay_ms: u64,
    /// Maximum number of reconnection attempts (0 = infinite)
    pub max_reconnect_attempts: u32,
}

impl Default for BandwidthClientConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:4433".to_string(),
            connection_timeout_ms: 5000,
            response_timeout_ms: 10000,
            auto_reconnect: true,
            reconnect_delay_ms: 1000,
            max_reconnect_attempts: 5,
        }
    }
}

impl BandwidthClientConfig {
    /// Create new client configuration with server address
    #[inline]
    pub fn new(server_addr: impl Into<String>) -> Self {
        Self {
            server_addr: server_addr.into(),
            ..Default::default()
        }
    }
    
    /// Set connection timeout in milliseconds
    #[inline]
    pub fn with_connection_timeout(mut self, timeout_ms: u64) -> Self {
        self.connection_timeout_ms = timeout_ms;
        self
    }
    
    /// Set response timeout in milliseconds  
    #[inline]
    pub fn with_response_timeout(mut self, timeout_ms: u64) -> Self {
        self.response_timeout_ms = timeout_ms;
        self
    }
    
    /// Enable or disable automatic reconnection
    #[inline]
    pub fn with_auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }
    
    /// Set reconnection delay in milliseconds
    #[inline]
    pub fn with_reconnect_delay(mut self, delay_ms: u64) -> Self {
        self.reconnect_delay_ms = delay_ms;
        self
    }
    
    /// Set maximum reconnection attempts (0 = infinite)
    #[inline]
    pub fn with_max_reconnect_attempts(mut self, max_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }
    
    /// Get connection timeout as Duration
    #[inline]
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_millis(self.connection_timeout_ms)
    }
    
    /// Get response timeout as Duration
    #[inline]
    pub fn response_timeout(&self) -> Duration {
        Duration::from_millis(self.response_timeout_ms)
    }
    
    /// Get reconnect delay as Duration
    #[inline]
    pub fn reconnect_delay(&self) -> Duration {
        Duration::from_millis(self.reconnect_delay_ms)
    }
}

/// Get current timestamp in milliseconds with zero allocation and safe error handling
#[inline]
pub fn get_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_else(|_| {
            warn!("System clock before UNIX epoch, using 0 timestamp");
            0
        })
}
