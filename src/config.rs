// ---
// src/config.rs
// ---
#![allow(clippy::module_name_repetitions)]

/// Builder‑style configuration for [`start_monitor`](crate::start_monitor).
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Network interface to sniff (`None` → first active, up interface).
    pub interface: Option<String>,
    /// Whether DNS names should be resolved.
    pub resolve_dns: bool,
    /// Optional custom DNS server (IPv4).
    pub dns_server: Option<std::net::Ipv4Addr>,
    /// Fallback to pure system statistics if raw sockets are unavailable
    /// even after attempting privilege escalation.
    pub allow_system_fallback: bool,
    /// Tick period (seconds) fed into the closure.
    pub period_secs: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interface: None,
            resolve_dns: true,
            dns_server: None,
            allow_system_fallback: true,
            period_secs: 1,
        }
    }
}
