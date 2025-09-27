//! Opportunistic debounced ticker for smooth event-driven animations
//!
//! This module provides a sophisticated animation driver that "surfs" on
//! the application's natural event flow, using scheduled timers for consistency.

use super::{debounced, Debounced, Tickable, TickerMetrics};
use crossbeam_channel::{bounded, Receiver, Sender};
use futures_util::StreamExt;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// Default early window percentage (20% = can tick at 80% of interval)
const DEFAULT_EARLY_WINDOW_PERCENT: u8 = 20;

/// A ticker that opportunistically advances animations on any event
///
/// This ticker implements a sliding window approach where animations can
/// advance up to 20% early when any event occurs, reducing the need for
/// dedicated timer events while maintaining smooth playback.
///
/// # Example
///
/// ```ignore
/// let spinner_group = TickableSpinners::new(controller);
/// let mut ticker = OpportunisticDebouncedTicker::new(spinner_group);
///
/// // In event handler - try to tick on any event
/// if ticker.on_any_event() {
///     needs_render = true;
/// }
///
/// // After render - schedule timer
/// ticker.schedule_timer();
///
/// // In event loop - poll for timer
/// if ticker.poll_timer(&mut cx).is_ready() {
///     needs_render = true;
/// }
/// ```
pub struct OpportunisticDebouncedTicker<T: Tickable> {
    /// The animatable target
    target: T,
    /// When we last ticked
    last_tick: Instant,
    /// When we ideally want to tick next
    next_ideal_tick: Instant,
    /// How early we can tick (as percentage of interval)
    early_window_percent: u8,
    /// Channel for scheduling timer ticks
    timer_tx: Sender<()>,
    /// Receiver for draining cancelled timers
    timer_rx: Receiver<()>,
    /// Debounced stream for animation timing
    timer_stream: Pin<Box<Debounced<()>>>,
    /// Performance metrics
    metrics: TickerMetrics,
}

impl<T: Tickable> OpportunisticDebouncedTicker<T> {
    /// Create a new ticker with default 20% early window
    #[inline]
    pub fn new(target: T) -> Self {
        Self::with_early_window(target, DEFAULT_EARLY_WINDOW_PERCENT)
    }

    /// Create a ticker with custom early window percentage
    pub fn with_early_window(target: T, early_window_percent: u8) -> Self {
        let early_window_percent = early_window_percent.min(50); // Cap at 50%
        let frame_interval = target.frame_interval();

        // Create private channel for this ticker
        let (timer_tx, timer_rx) = bounded(1);

        // Create debounced stream with frame interval
        let timer_stream = Box::pin(debounced(timer_rx.clone(), frame_interval));

        let now = Instant::now();

        Self {
            target,
            last_tick: now,
            next_ideal_tick: now + frame_interval,
            early_window_percent,
            timer_tx,
            timer_rx,
            timer_stream,
            metrics: TickerMetrics::default(),
        }
    }

    /// Check if it's time to tick based on sliding window
    #[inline]
    fn should_tick(&self, now: Instant) -> bool {
        if !self.target.is_active() {
            return false;
        }

        let frame_interval = self.target.frame_interval();
        let early_window = frame_interval * self.early_window_percent as u32 / 100;
        let early_threshold = self
            .next_ideal_tick
            .checked_sub(early_window)
            .unwrap_or(self.next_ideal_tick);

        now >= early_threshold
    }

    /// Called on every event - returns true if a tick occurred
    pub fn on_any_event(&mut self) -> bool {
        let now = Instant::now();

        if self.should_tick(now) {
            // Advance the animation
            self.target.tick();

            // Update timing
            let frame_interval = self.target.frame_interval();
            self.last_tick = now;
            self.next_ideal_tick = now + frame_interval;

            // Update metrics
            self.metrics.opportunistic_ticks += 1;
            if let Some(early_bound) = self.next_ideal_tick.checked_sub(frame_interval / 10) {
                if now < early_bound {
                    self.metrics.early_ticks += 1;
                }
            }

            // Drain any pending timer events to prevent double-tick
            while self.timer_rx.try_recv().is_ok() {
                self.metrics.cancelled_timers += 1;
            }

            true
        } else {
            false
        }
    }

    /// Schedule a timer tick via the debounced stream
    #[inline]
    pub fn schedule_timer(&mut self) {
        if self.target.is_active() {
            // Send to our private channel - debounced stream handles timing
            let _ = self.timer_tx.try_send(());
        } else {
            self.metrics.skipped_inactive += 1;
        }
    }

    /// Poll the timer stream - returns true if a tick occurred
    pub fn poll_timer(&mut self, cx: &mut Context<'_>) -> Poll<bool> {
        // Check if target is still active
        if !self.target.is_active() {
            self.metrics.skipped_inactive += 1;
            return Poll::Ready(false);
        }

        // Poll the debounced stream
        match self.timer_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(())) => {
                // Timer fired - tick now
                self.target.tick();

                // Update timing
                let now = Instant::now();
                let frame_interval = self.target.frame_interval();
                self.last_tick = now;
                self.next_ideal_tick = now + frame_interval;

                // Update metrics
                self.metrics.timer_ticks += 1;

                Poll::Ready(true)
            }
            Poll::Ready(None) => {
                // Stream ended (shouldn't happen with our bounded channel)
                Poll::Ready(false)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    /// Try to poll without a proper context (for sync contexts)
    #[inline]
    pub fn try_poll_timer(&mut self) -> bool {
        // Create a no-op waker
        let waker = futures_util::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        matches!(self.poll_timer(&mut cx), Poll::Ready(true))
    }

    /// Get access to the target
    #[inline]
    pub fn target(&self) -> &T {
        &self.target
    }

    /// Get mutable access to the target
    #[inline]
    pub fn target_mut(&mut self) -> &mut T {
        &mut self.target
    }

    /// Get current performance metrics
    #[inline]
    pub fn metrics(&self) -> TickerMetrics {
        self.metrics
    }

    /// Reset metrics
    #[inline]
    pub fn reset_metrics(&mut self) {
        self.metrics = TickerMetrics::default();
    }

    /// Get the early window percentage
    #[inline]
    pub const fn early_window_percent(&self) -> u8 {
        self.early_window_percent
    }

    /// Set a new early window percentage
    #[inline]
    pub fn set_early_window_percent(&mut self, percent: u8) {
        self.early_window_percent = percent.min(50);
    }

    /// Force a tick regardless of timing
    pub fn force_tick(&mut self) {
        self.target.tick();
        let now = Instant::now();
        self.last_tick = now;
        self.next_ideal_tick = now + self.target.frame_interval();
    }

    /// Check if the target is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.target.is_active()
    }

    /// Get time until next ideal tick
    #[inline]
    pub fn time_until_next_tick(&self) -> Duration {
        self.next_ideal_tick
            .saturating_duration_since(Instant::now())
    }

    /// Schedule a fallback tick (alias for schedule_timer)
    #[inline]
    pub fn schedule_fallback(&mut self) {
        self.schedule_timer()
    }

    /// Try to poll for fallback ticks (alias for try_poll_timer)
    #[inline]
    pub fn try_poll_fallback(&mut self) -> bool {
        self.try_poll_timer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestTickable {
        ticks: Arc<Mutex<u32>>,
        active: bool,
        interval: Duration,
    }

    impl Tickable for TestTickable {
        fn tick(&mut self) {
            *self.ticks.lock().unwrap() += 1;
        }

        fn is_active(&self) -> bool {
            self.active
        }

        fn frame_interval(&self) -> Duration {
            self.interval
        }
    }

    #[test]
    fn test_opportunistic_ticking() {
        let ticks = Arc::new(Mutex::new(0));
        let tickable = TestTickable {
            ticks: ticks.clone(),
            active: true,
            interval: Duration::from_millis(100),
        };

        let mut ticker = OpportunisticDebouncedTicker::new(tickable);

        // First event should tick immediately
        assert!(ticker.on_any_event());
        assert_eq!(*ticks.lock().unwrap(), 1);

        // Immediate event shouldn't tick (too early)
        assert!(!ticker.on_any_event());
        assert_eq!(*ticks.lock().unwrap(), 1);

        // Event at 80% of interval should tick (80ms for 100ms interval)
        std::thread::sleep(Duration::from_millis(80));
        assert!(ticker.on_any_event());
        assert_eq!(*ticks.lock().unwrap(), 2);
    }

    #[test]
    fn test_metrics_tracking() {
        let tickable = TestTickable {
            ticks: Arc::new(Mutex::new(0)),
            active: true,
            interval: Duration::from_millis(50),
        };

        let mut ticker = OpportunisticDebouncedTicker::new(tickable);

        // Opportunistic tick
        ticker.on_any_event();
        assert_eq!(ticker.metrics().opportunistic_ticks, 1);
        assert_eq!(ticker.metrics().timer_ticks, 0);

        // Schedule and drain
        ticker.schedule_timer();
        std::thread::sleep(Duration::from_millis(45)); // 90% of interval
        ticker.on_any_event();

        // Should have cancelled the timer
        assert_eq!(ticker.metrics().cancelled_timers, 1);
    }
}
