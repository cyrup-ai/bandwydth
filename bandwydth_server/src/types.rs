use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};
use std::net::SocketAddr;
use std::time::SystemTime;

/// Bandwidth event streamed to clients
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BandwidthEvent {
    pub timestamp: u64,
    pub download_bps: u64,
    pub upload_bps: u64,
    pub class: String,
}

/// Bandwidth class enumeration for zero-allocation classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum BandwidthClass {
    VeryLow,
    Low,
    Medium, 
    High,
    VeryHigh,
}

impl BandwidthClass {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VeryLow => "very_low",
            Self::Low => "low", 
            Self::Medium => "medium",
            Self::High => "high",
            Self::VeryHigh => "very_high",
        }
    }

    #[inline]
    pub const fn from_bps(bps: u64) -> Self {
        match bps {
            0..=1_000_000 => Self::VeryLow,        // < 1 Mbps
            1_000_001..=10_000_000 => Self::Low,   // 1-10 Mbps
            10_000_001..=100_000_000 => Self::Medium, // 10-100 Mbps
            100_000_001..=1_000_000_000 => Self::High, // 100 Mbps - 1 Gbps
            _ => Self::VeryHigh,                   // > 1 Gbps
        }
    }
}

/// Configuration for the QUIC-based bandwidth server
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BandwidthServerConfig {
    /// Address to bind the server to (e.g., "0.0.0.0:4433")
    pub bind_addr: String,
    /// TLS certificate chain in PEM format
    pub cert: Vec<u8>,
    /// TLS private key in PEM format
    pub key: Vec<u8>,
    /// Interval for broadcasting bandwidth updates (in milliseconds)
    pub broadcast_interval_ms: u64,
    /// Timeout for client connections (in milliseconds)
    pub client_timeout_ms: u64,
}

impl Default for BandwidthServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:4433".to_string(),
            cert: Vec::new(),
            key: Vec::new(),
            broadcast_interval_ms: 1000,
            client_timeout_ms: 5000,
        }
    }
}

/// Client connection information for event handlers
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct ClientInfo {
    /// Unique client identifier
    pub id: String,
    /// Client socket address
    pub addr: SocketAddr,
    /// Connection establishment timestamp
    pub connected_at: SystemTime,
}

impl ClientInfo {
    /// Create new client info with generated ID
    #[inline]
    pub fn new(addr: SocketAddr) -> Self {
        let id = Self::generate_client_id(&addr);
        Self {
            id,
            addr,
            connected_at: SystemTime::now(),
        }
    }
    
    /// Generate a unique client ID based on address and timestamp
    #[inline]
    fn generate_client_id(addr: &SocketAddr) -> String {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        
        format!("client_{}_{}", addr.ip(), timestamp)
    }
    
    /// Get connection duration since establishment
    #[inline]
    pub fn connection_duration(&self) -> core::result::Result<std::time::Duration, std::time::SystemTimeError> {
        SystemTime::now().duration_since(self.connected_at)
    }
}

/// Comprehensive bandwidth measurement data
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BandwidthData {
    /// Download speed in megabits per second
    pub download_mbps: f64,
    /// Upload speed in megabits per second  
    pub upload_mbps: f64,
    /// Measurement timestamp
    pub timestamp: SystemTime,
    /// Quality score (0.0 - 1.0) indicating measurement reliability
    pub quality_score: f64,
    /// Number of active client connections during measurement
    pub active_clients: u32,
    /// Total bytes transferred in measurement window
    pub total_bytes: u64,
}

impl BandwidthData {
    /// Create new bandwidth data with current timestamp
    #[inline]
    pub fn new(
        download_mbps: f64,
        upload_mbps: f64,
        quality_score: f64,
        active_clients: u32,
        total_bytes: u64,
    ) -> Self {
        Self {
            download_mbps,
            upload_mbps,
            timestamp: SystemTime::now(),
            quality_score,
            active_clients,
            total_bytes,
        }
    }
    
    /// Get total throughput in Mbps
    #[inline]
    pub const fn total_mbps(&self) -> f64 {
        self.download_mbps + self.upload_mbps
    }
    
    /// Get bandwidth class for current total throughput
    #[inline]
    pub fn bandwidth_class(&self) -> BandwidthClass {
        let total_bps = (self.total_mbps() * 1_000_000.0) as u64;
        BandwidthClass::from_bps(total_bps)
    }
    
    /// Check if measurement quality is acceptable (>= 0.7)
    #[inline]
    pub const fn is_high_quality(&self) -> bool {
        self.quality_score >= 0.7
    }
    
    /// Get measurement age since timestamp
    #[inline]
    pub fn measurement_age(&self) -> core::result::Result<std::time::Duration, std::time::SystemTimeError> {
        SystemTime::now().duration_since(self.timestamp)
    }
}