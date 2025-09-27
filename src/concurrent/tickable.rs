//! Trait for frame-based animations in event-driven systems
//!
//! This module provides the core trait for integrating animations
//! with the opportunistic debounced ticker system.

use std::time::Duration;

/// Trait for anything that can be animated frame-by-frame
///
/// Implement this trait to make your component work with the
/// `OpportunisticDebouncedTicker`, enabling smooth animation
/// that rides on the application's natural event flow.
///
/// # Example
///
/// ```ignore
/// struct ScrollingText {
///     text: String,
///     position: usize,
///     paused: bool,
/// }
///
/// impl Tickable for ScrollingText {
///     fn tick(&mut self) {
///         if !self.paused {
///             self.position = (self.position + 1) % self.text.len();
///         }
///     }
///
///     fn is_active(&self) -> bool {
///         !self.paused && !self.text.is_empty()
///     }
///
///     fn frame_interval(&self) -> Duration {
///         Duration::from_millis(100) // Scroll speed
///     }
/// }
/// ```
pub trait Tickable: Send + 'static {
    /// Advance the animation by one frame
    ///
    /// This method is called when it's time to update the animation.
    /// It should be fast and non-blocking.
    fn tick(&mut self);

    /// Check if the animation should be running
    ///
    /// Return `false` when the animation is paused, has no content,
    /// or otherwise shouldn't consume resources. The ticker will
    /// skip scheduling when this returns `false`.
    fn is_active(&self) -> bool;

    /// Get the desired interval between frames
    ///
    /// This determines the animation speed. The ticker will try to
    /// maintain this interval, using the sliding window technique
    /// to ensure smooth playback.
    fn frame_interval(&self) -> Duration;
}

/// Optional trait for tickables that support batch operations
pub trait BatchTickable: Tickable {
    /// Tick multiple items at once
    ///
    /// This can be more efficient than individual ticks for
    /// components managing multiple animations.
    fn tick_batch(&mut self, count: usize) {
        for _ in 0..count {
            self.tick();
        }
    }
}

/// Metrics for performance analysis
#[derive(Debug, Default, Clone, Copy)]
pub struct TickerMetrics {
    /// Ticks triggered by opportunistic event processing
    pub opportunistic_ticks: u64,
    /// Ticks triggered by the timer
    pub timer_ticks: u64,
    /// Ticks that happened earlier than ideal due to sliding window
    pub early_ticks: u64,
    /// Timer events that were cancelled (drained) after opportunistic ticks
    pub cancelled_timers: u64,
    /// Times we skipped ticking because target wasn't active
    pub skipped_inactive: u64,
}

impl TickerMetrics {
    /// Calculate the opportunistic efficiency (0.0 to 1.0)
    ///
    /// Higher values mean more ticks are happening via natural events
    /// rather than dedicated timers.
    #[inline]
    pub fn opportunistic_ratio(&self) -> f32 {
        let total = self.opportunistic_ticks + self.timer_ticks;
        if total == 0 {
            0.0
        } else {
            self.opportunistic_ticks as f32 / total as f32
        }
    }

    /// Get total tick count
    #[inline]
    pub const fn total_ticks(&self) -> u64 {
        self.opportunistic_ticks + self.timer_ticks
    }
}
