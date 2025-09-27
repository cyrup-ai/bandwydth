/// Bandwidth aggregation and calculation logic.
pub mod aggregator;
/// Network bandwidth monitoring and analysis.
pub mod bandwidth;
/// DNS resolution for network endpoints.
pub mod dns;
/// Interface statistics reading from system.
pub mod ifstat;
/// Main monitoring loop and thread management.
pub mod monitor;
/// Packet capture and analysis.
pub mod sniffer;
// Re-export specific types from bandwidth to avoid conflicts
pub use bandwidth::{NetInterfaceStats, NetMonitorHandle, NetSnapshot};
/// Core types and data structures.
pub mod r#type;

/// DNS and network resolution services.
pub mod resolver;
pub use resolver::*;

// Re-export specific types to avoid conflicts
pub use aggregator::{Aggregator, NetAggregator};
pub use dns::{DnsHandle, IpTable};
pub use monitor::{start_monitor, MonitorHandle};
// Use r#type module as the canonical source for shared network types
pub use r#type::{
    display_connection_string, display_ip_or_host, BandwidthClass, BandwidthStats, Connection,
    ConnectionInfo, InterfaceStats, LocalSocket, NetBandwidthClass, NetBandwidthStats,
    NetOverallBandwidthStatus, NetUsageStatus, OpenSockets, Protocol, RemoteSocket, Snapshot,
    Utilization,
};
// Export sniffer types
pub use sniffer::{Direction, Segment, Sniffer};
