//! Event-driven spinner components for no-tick rendering
//!
//! This module provides a complete spinner system that operates on events
//! rather than time-based ticks, ensuring zero allocations and optimal performance.

mod controller;
mod state;
mod tickable;
mod widget;

pub use controller::SpinnerController;
pub use state::{SpinnerDirection, SpinnerState};
pub use tickable::{TickMode, TickableSpinners, TickableSpinnersBuilder};
pub use widget::SpinnerWidget;

// Re-export commonly used types from the crate root
pub use crate::spinners::{SpinnerFrames, SpinnerPreset};
