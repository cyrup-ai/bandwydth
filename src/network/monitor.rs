use std::time::Duration;

use crate::{
    concurrent::{OpportunisticDebouncedTicker, Tickable},
    network::{aggregator::NetAggregator, bandwidth::read_snapshot, r#type::NetBandwidthStats},
};

/// Tickable bandwidth monitor that implements the opportunistic event pattern
pub struct TickableBandwidthMonitor<F>
where
    F: FnMut(NetBandwidthStats<10>) + Send + 'static,
{
    aggregator: NetAggregator<10>,
    callback: F,
    period: Duration,
    active: bool,
}

impl<F> TickableBandwidthMonitor<F>
where
    F: FnMut(NetBandwidthStats<10>) + Send + 'static,
{
    /// Create a new tickable bandwidth monitor
    pub fn new(period_secs: u64, callback: F) -> Self {
        let period = Duration::from_secs(period_secs);
        let aggregator = NetAggregator::<10>::new(period, 100.0, 0.5);

        Self {
            aggregator,
            callback,
            period,
            active: true,
        }
    }

    /// Enable or disable monitoring
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl<F> Tickable for TickableBandwidthMonitor<F>
where
    F: FnMut(NetBandwidthStats<10>) + Send + 'static,
{
    fn tick(&mut self) {
        if !self.active {
            return;
        }

        match read_snapshot() {
            Ok(snapshot) => {
                if let Some(stats) = self.aggregator.update(snapshot) {
                    (self.callback)(stats);
                }
            }
            Err(e) => eprintln!("monitor error: {e}"),
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn frame_interval(&self) -> Duration {
        self.period
    }
}

/// Handle for controlling a running bandwidth monitor using opportunistic ticking
pub struct MonitorHandle<F>
where
    F: FnMut(NetBandwidthStats<10>) + Send + 'static,
{
    ticker: OpportunisticDebouncedTicker<TickableBandwidthMonitor<F>>,
}

impl<F> MonitorHandle<F>
where
    F: FnMut(NetBandwidthStats<10>) + Send + 'static,
{
    /// Stop the bandwidth monitoring
    pub fn stop(mut self) {
        self.ticker.target_mut().set_active(false);
    }

    /// Check if a tick should happen on this event
    pub fn on_any_event(&mut self) -> bool {
        self.ticker.on_any_event()
    }

    /// Schedule a fallback tick
    pub fn schedule_fallback(&mut self) {
        self.ticker.schedule_fallback();
    }

    /// Try to poll for fallback ticks (for sync contexts)
    pub fn try_poll_fallback(&mut self) -> bool {
        self.ticker.try_poll_fallback()
    }
}

/// Start a bandwidth monitoring thread that polls network statistics.
///
/// This function spawns a background thread that periodically reads network
/// interface statistics and calculates bandwidth. The provided callback is
/// called with bandwidth statistics on each polling interval.
///
/// # Arguments
///
/// * `period_secs` - Polling interval in seconds
/// * `on_tick` - Callback function called with bandwidth statistics
///
/// # Returns
///
/// A `MonitorHandle` that can be used to stop the monitoring thread.
///
/// # Errors
///
/// Returns an I/O error if the monitoring thread cannot be started.
pub fn start_monitor<F>(period_secs: u64, on_tick: F) -> std::io::Result<MonitorHandle<F>>
where
    F: FnMut(NetBandwidthStats<10>) + Send + 'static,
{
    let tickable_monitor = TickableBandwidthMonitor::new(period_secs, on_tick);
    let ticker = OpportunisticDebouncedTicker::new(tickable_monitor);

    Ok(MonitorHandle { ticker })
}
