//! Event-driven components for no-tick rendering
//!
//! This module provides UI components that operate on events
//! rather than time-based ticks, ensuring zero allocations and optimal performance.

pub mod spinner;
pub mod text_input;

// Re-export commonly used types
pub use spinner::{SpinnerController, SpinnerState, SpinnerWidget, TickableSpinners};
pub use text_input::TextInput;
