//! Display modes for the bandwidth monitoring TUI.
//!
//! This module defines different visualization modes that users can switch between
//! using tab navigation in the main interface.

use std::fmt;

/// Different display modes available in the TUI interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Comprehensive bandwidth overview with real-time analysis
    Overview,
    /// Detailed per-interface statistics and debugging information
    InterfaceDebug,
    /// Advanced monitoring with historical trends and DNS resolution
    AdvancedMonitor,
    /// Statistical analysis with performance recommendations
    BandwidthStats,
}

impl DisplayMode {
    /// Get all available display modes in order
    #[inline(always)]
    pub const fn all() -> &'static [DisplayMode] {
        &[
            DisplayMode::Overview,
            DisplayMode::InterfaceDebug,
            DisplayMode::AdvancedMonitor,
            DisplayMode::BandwidthStats,
        ]
    }

    /// Get the display name for this mode (shown in tab bar)
    #[inline(always)]
    pub const fn display_name(self) -> &'static str {
        match self {
            DisplayMode::Overview => "Overview",
            DisplayMode::InterfaceDebug => "Interface Debug",
            DisplayMode::AdvancedMonitor => "Advanced",
            DisplayMode::BandwidthStats => "Stats",
        }
    }

    /// Get a short description of what this mode shows
    #[inline(always)]
    pub const fn description(self) -> &'static str {
        match self {
            DisplayMode::Overview => "Comprehensive bandwidth monitoring with real-time analysis",
            DisplayMode::InterfaceDebug => "Detailed per-interface network statistics",
            DisplayMode::AdvancedMonitor => "Historical trends with DNS resolution",
            DisplayMode::BandwidthStats => "Statistical analysis and performance insights",
        }
    }

    /// Get the next mode in the sequence (for tab navigation)
    #[inline(always)]
    pub const fn next(self) -> Self {
        match self {
            DisplayMode::Overview => DisplayMode::InterfaceDebug,
            DisplayMode::InterfaceDebug => DisplayMode::AdvancedMonitor,
            DisplayMode::AdvancedMonitor => DisplayMode::BandwidthStats,
            DisplayMode::BandwidthStats => DisplayMode::Overview,
        }
    }

    /// Get the previous mode in the sequence (for shift+tab navigation)
    #[inline(always)]
    pub const fn previous(self) -> Self {
        match self {
            DisplayMode::Overview => DisplayMode::BandwidthStats,
            DisplayMode::InterfaceDebug => DisplayMode::Overview,
            DisplayMode::AdvancedMonitor => DisplayMode::InterfaceDebug,
            DisplayMode::BandwidthStats => DisplayMode::AdvancedMonitor,
        }
    }

    /// Check if this mode requires DNS resolution
    #[inline(always)]
    pub const fn requires_dns(self) -> bool {
        match self {
            DisplayMode::Overview => true, // Full featured overview includes DNS
            DisplayMode::InterfaceDebug => false,
            DisplayMode::AdvancedMonitor => true,
            DisplayMode::BandwidthStats => false,
        }
    }

    /// Check if this mode requires historical data tracking
    #[inline(always)]
    pub const fn requires_history(self) -> bool {
        match self {
            DisplayMode::Overview => true, // Comprehensive history tracking
            DisplayMode::InterfaceDebug => false,
            DisplayMode::AdvancedMonitor => true,
            DisplayMode::BandwidthStats => true,
        }
    }
}

impl Default for DisplayMode {
    #[inline(always)]
    fn default() -> Self {
        DisplayMode::Overview
    }
}

impl fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
