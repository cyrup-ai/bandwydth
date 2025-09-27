//! Application display UI module with components and layout management.

/// Display components for UI elements.
pub mod components;
/// Display modes for tab navigation.
pub mod modes;
mod raw_terminal_backend;
/// Tickable wrappers for UI components
pub mod tickable_wrapper;
/// User interface implementation for bandwidth monitoring.
pub mod ui;
/// Unified ticker for UI components based on display mode
pub mod ui_components_ticker;
/// Safe UI components wrapper for opportunistic ticking
pub mod ui_components_tickable;
mod ui_state;

pub use components::*;
pub use modes::*;
pub use raw_terminal_backend::*;
pub use ui::*;
pub use ui_state::*;
