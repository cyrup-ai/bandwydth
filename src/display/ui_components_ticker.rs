//! Unified ticker for UI components
//!
//! This module provides a single tickable that manages all UI components
//! based on the current display mode.

use crate::{
    concurrent::tickable::Tickable,
    display::{ui::Ui, DisplayMode},
};
use ratatui::backend::Backend;
use std::{ptr::NonNull, time::Duration};

/// Wrapper that makes UI tickable based on current display mode
pub struct UiComponentsTicker<B: Backend + 'static> {
    ui_ptr: Option<NonNull<Ui<B>>>,
}

impl<B: Backend + 'static> UiComponentsTicker<B> {
    /// Create a new ticker for UI components
    ///
    /// # Safety
    /// 
    /// The caller must ensure the UI reference remains valid for the entire lifetime
    /// of this UiComponentsTicker instance. The UI object must not be moved or 
    /// deallocated while this ticker exists.
    pub fn new(ui: &mut Ui<B>) -> Self {
        Self { ui_ptr: Some(NonNull::from(ui)) }
    }
    
    /// Create an inactive ticker with no UI reference
    ///
    /// This ticker will be inactive and all operations will be no-ops until
    /// a UI reference is set using set_ui().
    pub fn new_inactive() -> Self {
        Self { ui_ptr: None }
    }
    
    /// Set the UI reference for this ticker
    ///
    /// # Safety
    /// 
    /// The caller must ensure the UI reference remains valid for the entire lifetime
    /// of this UiComponentsTicker instance.
    pub fn set_ui(&mut self, ui: &mut Ui<B>) {
        self.ui_ptr = Some(NonNull::from(ui));
    }
    
    /// Remove the UI reference, making this ticker inactive
    pub fn clear_ui(&mut self) {
        self.ui_ptr = None;
    }
}

unsafe impl<B: Backend + 'static> Send for UiComponentsTicker<B> {}

impl<B: Backend + 'static> Tickable for UiComponentsTicker<B> {
    fn tick(&mut self) {
        if let Some(mut ui_ptr) = self.ui_ptr {
            unsafe {
                // Safe because NonNull guarantees the pointer is not null
                // and lifetime is managed externally
                let ui = ui_ptr.as_mut();
                ui.tick();
            }
        }
        // No-op if ui_ptr is None
    }

    fn is_active(&self) -> bool {
        if let Some(ui_ptr) = self.ui_ptr {
            unsafe {
                // Safe because NonNull guarantees the pointer is not null
                let ui = ui_ptr.as_ref();
                // Check if current mode's component is active
                match ui.current_display_mode() {
                    DisplayMode::Overview => Tickable::is_active(ui.bandwidth_graph_state()),
                    DisplayMode::BandwidthStats => {
                        Tickable::is_active(ui.bandwidth_stats_component())
                    }
                    DisplayMode::AdvancedMonitor => {
                        Tickable::is_active(ui.advanced_monitor_component())
                    }
                    DisplayMode::InterfaceDebug => {
                        Tickable::is_active(ui.interface_debug_component())
                    }
                }
            }
        } else {
            // Inactive if no UI reference
            false
        }
    }

    fn frame_interval(&self) -> Duration {
        if let Some(ui_ptr) = self.ui_ptr {
            unsafe {
                // Safe because NonNull guarantees the pointer is not null
                let ui = ui_ptr.as_ref();
                // Return interval based on current display mode
                match ui.current_display_mode() {
                    DisplayMode::Overview => Duration::from_millis(16), // 60fps
                    DisplayMode::BandwidthStats => Duration::from_millis(500),
                    DisplayMode::AdvancedMonitor => Duration::from_millis(500),
                    DisplayMode::InterfaceDebug => Duration::from_millis(1000),
                }
            }
        } else {
            // Default interval when inactive
            Duration::from_millis(250)
        }
    }
}
