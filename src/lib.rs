// ---
// src/lib.rs
// ---
#![deny(missing_docs)]
//! # NetMon – A tick‑driven network / system monitoring library
//!
//! This library provides network bandwidth monitoring capabilities with configurable
//! polling intervals and callback-based event handling.
//!
//! ## Quick Start
//!
//! ```no_run
//! use netmon::{MonitorConfig, start_monitor};
//!
//! // Build a config – here we want bandwidth stats each second
//! let cfg = MonitorConfig::default();
//!
//! // Start the monitor and feed a closure with live `BandwidthStats`
//! let handle = start_monitor(cfg, |stats| {
//!     println!("Download+Upload: {:.2} MB/s ({:?})",
//!              stats.current_speed, stats.bandwidth_class);
//! }).expect("monitor failed to start");
//!
//! // … later, shut everything down gracefully
//! handle.stop();
//! ```

/// Action types for event handling.
pub mod action;
/// Application state and event processing.
pub mod app;
/// CLI argument parsing and options.
pub mod cli;
/// UI components including spinners.
pub mod components;
/// Concurrent utilities for zero-allocation async operations.
pub mod concurrent;
/// Configuration module for bandwidth monitoring.
pub mod config;
/// Data types for application state.
pub mod data;
/// Display and UI components.
pub mod display;
/// Event handling for terminal input.
pub mod event;
/// Network monitoring modules including bandwidth calculation.
pub mod network;
/// OS-specific functionality.
pub mod os;
/// Privilege management utilities.
pub mod privileges;
/// Debug utilities for diagnosing network interface issues.
pub mod debug_interfaces;
/// Spinner presets and frames.
pub mod spinners;
/// Theme module for UI styling.
pub mod theme;
/// Terminal UI wrapper.
pub mod tui;

pub use config::*;
pub use network::*;
pub use theme::Theme;

// Re-export commonly used types
pub use network::r#type::OpenSockets;

// Re-export bandwidth graph components for external use
pub use display::components::{
    BandwidthDataManager, BandwidthGraphEffectRenderer, BandwidthGraphState, BandwidthGraphWidget,
    BandwidthLevel, BandwidthPoint,
};

/// Start a bandwidth monitor with the given configuration and callback.
///
/// This function spawns a background thread that continuously monitors network
/// bandwidth usage according to the provided configuration. The callback function
/// is invoked at each polling interval with fresh bandwidth statistics.
///
/// # Arguments
///
/// * `cfg` - Monitor configuration specifying interface, polling interval, etc.
/// * `callback` - Function called with bandwidth statistics on each polling interval.
///   Must be `Send + 'static` as it will be moved to the monitoring thread.
///
/// # Returns
///
/// A `MonitorHandle` that can be used to stop the monitoring thread gracefully.
///
/// # Errors
///
/// Returns an `std::io::Error` if:
/// - The specified network interface cannot be found or accessed
/// - Insufficient privileges to read network statistics
/// - System resource allocation fails
///
/// # Example
///
/// ```no_run
/// use netmon::{MonitorConfig, start_monitor};
///
/// let cfg = MonitorConfig::default();
/// let handle = start_monitor(cfg, |stats| {
///     println!("Speed: {:.2} MB/s", stats.current_speed);
/// }).expect("Failed to start monitor");
///
/// // Later...
/// handle.stop();
/// ```
pub fn start_monitor<F>(cfg: MonitorConfig, callback: F) -> std::io::Result<MonitorHandle<F>>
where
    F: FnMut(NetBandwidthStats<10>) + Send + 'static,
{
    network::monitor::start_monitor(cfg.period_secs, callback)
}
