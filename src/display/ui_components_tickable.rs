//! Safe UI components wrapper for opportunistic ticking
//!
//! This module provides a memory-safe alternative to the UiComponentsTicker
//! that avoids unsafe self-references by owning the UI components.

use crate::{
    concurrent::tickable::Tickable,
    display::{
        components::{
            AdvancedMonitorComponent, BandwidthGraphState, BandwidthStatsComponent,
            InterfaceDebugComponent,
        },
        DisplayMode,
    },
};

use std::time::Duration;

/// Safe wrapper that owns UI components for opportunistic ticking
pub struct UiComponentsTickable {
    /// Bandwidth graph component
    bandwidth_graph_state: BandwidthGraphState,
    /// Bandwidth stats component
    bandwidth_stats_component: BandwidthStatsComponent,
    /// Advanced monitor component
    advanced_monitor_component: AdvancedMonitorComponent,
    /// Interface debug component
    interface_debug_component: InterfaceDebugComponent,
    /// Current display mode
    current_mode: DisplayMode,
}

impl UiComponentsTickable {
    /// Create a new UI components tickable
    pub fn new() -> Self {
        Self {
            bandwidth_graph_state: BandwidthGraphState::new(),
            bandwidth_stats_component: BandwidthStatsComponent::new(),
            advanced_monitor_component: AdvancedMonitorComponent::new(),
            interface_debug_component: InterfaceDebugComponent::new(),
            current_mode: DisplayMode::Overview,
        }
    }

    /// Set the current display mode
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.current_mode = mode;
    }

    /// Get bandwidth graph state
    pub fn bandwidth_graph_state(&self) -> &BandwidthGraphState {
        &self.bandwidth_graph_state
    }

    /// Get mutable bandwidth graph state
    pub fn bandwidth_graph_state_mut(&mut self) -> &mut BandwidthGraphState {
        &mut self.bandwidth_graph_state
    }

    /// Get bandwidth stats component
    pub fn bandwidth_stats_component(&self) -> &BandwidthStatsComponent {
        &self.bandwidth_stats_component
    }

    /// Get mutable bandwidth stats component
    pub fn bandwidth_stats_component_mut(&mut self) -> &mut BandwidthStatsComponent {
        &mut self.bandwidth_stats_component
    }

    /// Get advanced monitor component
    pub fn advanced_monitor_component(&self) -> &AdvancedMonitorComponent {
        &self.advanced_monitor_component
    }

    /// Get mutable advanced monitor component
    pub fn advanced_monitor_component_mut(&mut self) -> &mut AdvancedMonitorComponent {
        &mut self.advanced_monitor_component
    }

    /// Get interface debug component
    pub fn interface_debug_component(&self) -> &InterfaceDebugComponent {
        &self.interface_debug_component
    }

    /// Get mutable interface debug component
    pub fn interface_debug_component_mut(&mut self) -> &mut InterfaceDebugComponent {
        &mut self.interface_debug_component
    }
}

impl Default for UiComponentsTickable {
    fn default() -> Self {
        Self::new()
    }
}

impl Tickable for UiComponentsTickable {
    fn tick(&mut self) {
        // Tick the appropriate component based on current mode
        match self.current_mode {
            DisplayMode::Overview => self.bandwidth_graph_state.tick(),
            DisplayMode::BandwidthStats => self.bandwidth_stats_component.tick(),
            DisplayMode::AdvancedMonitor => self.advanced_monitor_component.tick(),
            DisplayMode::InterfaceDebug => self.interface_debug_component.tick(),
        }
    }

    fn is_active(&self) -> bool {
        // Check if current mode's component is active
        match self.current_mode {
            DisplayMode::Overview => self.bandwidth_graph_state.is_active(),
            DisplayMode::BandwidthStats => self.bandwidth_stats_component.is_active(),
            DisplayMode::AdvancedMonitor => self.advanced_monitor_component.is_active(),
            DisplayMode::InterfaceDebug => self.interface_debug_component.is_active(),
        }
    }

    fn frame_interval(&self) -> Duration {
        // Return interval based on current display mode
        match self.current_mode {
            DisplayMode::Overview => Duration::from_millis(16), // 60fps
            DisplayMode::BandwidthStats => Duration::from_millis(500),
            DisplayMode::AdvancedMonitor => Duration::from_millis(500),
            DisplayMode::InterfaceDebug => Duration::from_millis(1000),
        }
    }
}