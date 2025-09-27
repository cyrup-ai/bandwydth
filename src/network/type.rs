// src/network/types.rs
// ----------------------
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

/// Classification of bandwidth quality based on real-world download performance.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BandwidthClass {
    /// Blazing bandwidth (>500 Mbps) - Exceptional multi-gigabit performance.
    Blazing,
    /// Excellent bandwidth (200-500 Mbps) - Premium performance, maxes most services.
    Excellent,
    /// Good bandwidth (100-200 Mbps) - Solid performance for all modern needs.
    Good,
    /// Fair bandwidth (50-100 Mbps) - Adequate for modern usage.
    Fair,
    /// Poor bandwidth (<50 Mbps) - Below modern standards.
    Poor,
    /// Inconclusive - insufficient data or not at sustained load.
    Inconclusive,
}

impl Default for BandwidthClass {
    fn default() -> Self {
        Self::Inconclusive
    }
}

/// Current bandwidth statistics and history.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BandwidthStats {
    /// Current bandwidth speed in MB/s.
    pub current_speed: f64, // MBps
    /// Historical bandwidth measurements.
    pub bandwidth_history: Vec<f64>,
    /// Quality classification of current bandwidth.
    pub bandwidth_class: BandwidthClass,
}

/// Statistics for a single network interface.
#[derive(Debug, Clone, Default)]
pub struct InterfaceStats {
    /// Interface name (e.g., "eth0", "wlan0").
    pub name: String,
    /// Total bytes received since boot.
    pub received_bytes: u64,
    /// Total bytes transmitted since boot.
    pub transmitted_bytes: u64,
}

/// Snapshot of all network interface statistics.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// Map of interface names to their statistics.
    pub interfaces: HashMap<String, InterfaceStats>,
}

/// Network protocol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Protocol {
    /// TCP protocol.
    Tcp,
    /// UDP protocol.
    Udp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

/// Socket information.
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Copy)]
pub struct Socket {
    /// IP address of the socket.
    pub ip: IpAddr,
    /// Port number of the socket.
    pub port: u16,
}

impl std::fmt::Debug for Socket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Socket { ip, port } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "[{v6}]:{port}"),
        }
    }
}

/// Local socket information.
#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct LocalSocket {
    /// IP address of the local socket.
    pub ip: IpAddr,
    /// Port number of the local socket.
    pub port: u16,
    /// Protocol used by the socket.
    pub protocol: Protocol,
}

impl std::fmt::Debug for LocalSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LocalSocket { ip, port, protocol } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{protocol}://{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "{protocol}://[{v6}]:{port}"),
        }
    }
}

/// Remote socket information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoteSocket {
    /// Remote IP address.
    pub ip: IpAddr,
    /// Remote port number.
    pub port: u16,
}

/// Network connection information.
#[derive(Clone, Copy)]
pub struct Connection {
    /// Remote socket information.
    pub remote_socket: Socket,
    /// Local socket information.
    pub local_socket: LocalSocket,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Connection {
            remote_socket,
            local_socket,
        } = self;
        write!(f, "{local_socket:?} => {remote_socket:?}")
    }
}

impl Connection {
    /// Create a new connection from remote socket address and local socket information.
    pub fn new(
        remote_socket: SocketAddr,
        local_ip: IpAddr,
        local_port: u16,
        protocol: Protocol,
    ) -> Self {
        Connection {
            remote_socket: Socket {
                ip: remote_socket.ip(),
                port: remote_socket.port(),
            },
            local_socket: LocalSocket {
                ip: local_ip,
                port: local_port,
                protocol,
            },
        }
    }
}

/// Network utilization statistics.
#[derive(Debug, Clone)]
pub struct Utilization {
    /// Map of connections to their bandwidth usage.
    pub connections: HashMap<Connection, ConnectionInfo>,
}

impl Utilization {
    /// Create a new empty utilization tracker.
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Ingest a network segment and update connection statistics.
    pub fn ingest(&mut self, segment: crate::network::Segment) {
        let connection_info = self
            .connections
            .entry(segment.connection)
            .or_insert_with(|| ConnectionInfo {
                sent_bytes: 0,
                recv_bytes: 0,
                total_bytes_downloaded: 0,
                total_bytes_uploaded: 0,
                interface_name: segment.interface_name.clone(),
            });

        // Update bandwidth counters based on direction
        match segment.direction {
            crate::network::Direction::Download => {
                connection_info.total_bytes_downloaded += segment.data_length;
                connection_info.recv_bytes += segment.data_length as u64;
            }
            crate::network::Direction::Upload => {
                connection_info.total_bytes_uploaded += segment.data_length;
                connection_info.sent_bytes += segment.data_length as u64;
            }
        }
    }

    /// Clone the current utilization data and reset the tracker.
    pub fn clone_and_reset(&mut self) -> Self {
        let cloned = self.clone();
        self.connections.clear();
        cloned
    }
}

impl Default for Utilization {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a connection's bandwidth usage.
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Bytes sent.
    pub sent_bytes: u64,
    /// Bytes received.
    pub recv_bytes: u64,
    /// Total bytes downloaded.
    pub total_bytes_downloaded: u128,
    /// Total bytes uploaded.
    pub total_bytes_uploaded: u128,
    /// Interface name.
    pub interface_name: String,
}

impl std::hash::Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local_socket.hash(state);
        self.remote_socket.hash(state);
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.local_socket == other.local_socket && self.remote_socket == other.remote_socket
    }
}

impl Eq for Connection {}

/// Display helper for connection strings.
pub fn display_connection_string(
    remote_socket: &Socket,
    protocol: Protocol,
    local_socket: &LocalSocket,
    ip_to_host: &HashMap<IpAddr, String>,
) -> String {
    let remote_host = ip_to_host
        .get(&remote_socket.ip)
        .cloned()
        .unwrap_or_else(|| remote_socket.ip.to_string());

    format!(
        "{} {}:{} -> {}:{}",
        match protocol {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        },
        local_socket.ip,
        local_socket.port,
        remote_host,
        remote_socket.port
    )
}

/// Display helper for IP or hostname.
pub fn display_ip_or_host(ip: &IpAddr, _host_name: &Option<String>) -> String {
    // For now, just display the IP. DNS resolution can be added later.
    ip.to_string()
}

/// Collection of open sockets.
#[derive(Debug, Clone)]
pub struct OpenSockets {
    /// Map of local sockets to their owning processes.
    pub sockets_to_procs: std::collections::HashMap<LocalSocket, crate::os::ProcessInfo>,
}

/// Enhanced classification of network bandwidth quality based on speed thresholds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetBandwidthClass {
    /// Unable to determine bandwidth or no data available.
    Inconclusive,
    /// Poor bandwidth (10+ Mbps).
    Poor,
    /// Average bandwidth (100+ Mbps).
    Average,
    /// Good bandwidth (500+ Mbps).
    Good,
    /// Blazing fast bandwidth (1000+ Mbps).
    Blazing,
}

/// Status indicating network usage patterns over time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetUsageStatus {
    /// Consistently high bandwidth usage with low variability.
    SteadyHigh,
    /// Usage pattern is unclear or variable.
    Inconclusive,
}

/// Overall assessment of network bandwidth combining speed and usage patterns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetOverallBandwidthStatus {
    /// Unable to determine status or insufficient data.
    Inconclusive,
    /// Poor overall performance.
    Poor,
    /// Average overall performance.
    Average,
    /// Good overall performance.
    Good,
    /// Blazing fast overall performance.
    Blazing,
}

impl std::fmt::Display for NetUsageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetUsageStatus::SteadyHigh => write!(f, "steady-high"),
            NetUsageStatus::Inconclusive => write!(f, "inconclusive"),
        }
    }
}

impl std::fmt::Display for NetBandwidthClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetBandwidthClass::Inconclusive => write!(f, "inconclusive"),
            NetBandwidthClass::Poor => write!(f, "poor"),
            NetBandwidthClass::Average => write!(f, "average"),
            NetBandwidthClass::Good => write!(f, "good"),
            NetBandwidthClass::Blazing => write!(f, "blazing"),
        }
    }
}

impl std::fmt::Display for NetOverallBandwidthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetOverallBandwidthStatus::Inconclusive => write!(f, "inconclusive"),
            NetOverallBandwidthStatus::Poor => write!(f, "poor"),
            NetOverallBandwidthStatus::Average => write!(f, "average"),
            NetOverallBandwidthStatus::Good => write!(f, "good"),
            NetOverallBandwidthStatus::Blazing => write!(f, "blazing"),
        }
    }
}

/// Comprehensive bandwidth statistics with historical data.
#[derive(Debug, Clone, Copy)]
pub struct NetBandwidthStats<const N: usize> {
    /// Current bandwidth speed in Mbps.
    pub current_speed: f64, // Mbps
    /// Historical bandwidth measurements (fixed-size array).
    pub bandwidth_history: [f64; N], // Mbps
    /// Quality classification of current bandwidth.
    pub bandwidth_class: NetBandwidthClass,
    /// Usage pattern status over time.
    pub usage_status: NetUsageStatus,
}

impl<const N: usize> NetBandwidthStats<N> {
    /// Get the overall bandwidth status combining classification and usage patterns.
    pub fn overall_status(&self) -> NetOverallBandwidthStatus {
        match self.bandwidth_class {
            NetBandwidthClass::Inconclusive => NetOverallBandwidthStatus::Inconclusive,
            NetBandwidthClass::Poor => NetOverallBandwidthStatus::Poor,
            NetBandwidthClass::Average => NetOverallBandwidthStatus::Average,
            NetBandwidthClass::Good => NetOverallBandwidthStatus::Good,
            NetBandwidthClass::Blazing => NetOverallBandwidthStatus::Blazing,
        }
    }

    /// Check if bandwidth is consistently high (good performance indicator).
    pub fn is_high_performance(&self) -> bool {
        matches!(self.usage_status, NetUsageStatus::SteadyHigh)
            && matches!(
                self.bandwidth_class,
                NetBandwidthClass::Good | NetBandwidthClass::Blazing
            )
    }

    /// Get a human-readable description of the current bandwidth.
    pub fn description(&self) -> String {
        format!("{:.1} Mbps ({})", self.current_speed, self.bandwidth_class)
    }

    /// Get the average bandwidth from history.
    #[inline(always)]
    pub fn average_speed(&self) -> f64 {
        // Fast path for current speed if no history
        if self.bandwidth_history[0] == 0.0 {
            return self.current_speed;
        }

        let mut sum = 0.0;
        let mut count = 0;

        // Only average non-zero values
        for &speed in &self.bandwidth_history {
            if speed > 0.0 {
                sum += speed;
                count += 1;
            }
        }

        if count > 0 {
            sum / count as f64
        } else {
            self.current_speed
        }
    }

    /// Check if bandwidth is below a threshold.
    #[inline(always)]
    pub fn is_below_threshold(&self, threshold_mbps: f64) -> bool {
        self.current_speed < threshold_mbps
    }

    /// Check if bandwidth is above a threshold.
    #[inline(always)]
    pub fn is_above_threshold(&self, threshold_mbps: f64) -> bool {
        self.current_speed > threshold_mbps
    }
}
