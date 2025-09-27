//! Zero-allocation spinner presets for terminal UI applications.
//!
//! This crate provides compile-time spinner frame data with feature-gated
//! presets. Enable only the spinners you need to minimize binary size.
//!
//! # Features
//!
//! - Individual spinner features: `dots`, `line`, `arc`, etc.
//! - Feature groups: `all` (all spinners), `dots_all`, `toggle_all`
//!
//! # Usage
//!
//! ## Stateless Spinner
//!
//! ```ignore
//! use zeroshot_spinner::{Spinner, SpinnerPreset};
//! use std::time::Instant;
//!
//! let start = Instant::now();
//! let preset = SpinnerPreset::Dots;
//! let frames = preset.frames();
//! let interval = preset.interval();
//!
//! // In your render loop:
//! let elapsed = start.elapsed();
//! let frame_index = (elapsed.as_millis() / interval.as_millis()) as usize % frames.len();
//! let spinner = Spinner::new(preset, frame_index);
//! ```
//!
//! ## Stateful Spinner
//!
//! ```ignore
//! use zeroshot_spinner::{SpinnerState, SpinnerPreset};
//!
//! let mut spinner = SpinnerState::new(SpinnerPreset::Dots);
//!
//! // In your render loop - the spinner automatically animates
//! terminal.draw(|f| {
//!     f.render_widget(&spinner, area);
//! })?;
//! ```
//!
//! # Cargo.toml Example
//!
//! ```toml
//! [dependencies]
//! opportunistic_event_ticker = { version = "0.1", features = ["dots", "line"] }
//! ```

// Internal modules
pub mod action;
pub mod app;
pub mod components;
pub mod concurrent;
pub mod event;
pub mod spinners;
pub mod tui;

// Re-export commonly used types at the crate root
pub use components::spinner::{SpinnerController, SpinnerState, SpinnerWidget};
pub use concurrent::*;
pub use spinners::{SpinnerFrames, SpinnerPreset};
