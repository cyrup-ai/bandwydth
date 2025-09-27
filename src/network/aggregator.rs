use std::collections::VecDeque;
use std::time::Duration;

use crate::network::bandwidth::NetSnapshot;
use crate::network::r#type::{
    BandwidthClass, BandwidthStats, NetBandwidthClass, NetBandwidthStats, NetUsageStatus, Snapshot,
};

/// Aggregates network interface statistics to calculate bandwidth over time.
pub struct Aggregator {
    last: Option<Snapshot>,
    history: VecDeque<f64>,
    max_history: usize,
    interval: Duration,
}

impl Aggregator {
    /// Create a new bandwidth aggregator.
    ///
    /// # Arguments
    ///
    /// * `interval` - Time between snapshots for bandwidth calculation
    /// * `max_history` - Maximum number of historical measurements to keep
    pub fn new(interval: Duration, max_history: usize) -> Self {
        Self {
            last: None,
            history: VecDeque::with_capacity(max_history),
            max_history,
            interval,
        }
    }

    /// Update with a new network statistics snapshot and calculate bandwidth.
    ///
    /// Returns `None` on the first call (needs two snapshots to calculate bandwidth).
    ///
    /// # Arguments
    ///
    /// * `snapshot` - Current network interface statistics
    ///
    /// # Returns
    ///
    /// `Some(BandwidthStats)` if bandwidth could be calculated, `None` otherwise.
    pub fn update(&mut self, snapshot: Snapshot) -> Option<BandwidthStats> {
        let last = self.last.replace(snapshot.clone())?;

        let mut total_delta_bytes = 0u64;

        for (name, current) in &snapshot.interfaces {
            if name == "lo" {
                continue;
            }
            if let Some(prev) = last.interfaces.get(name) {
                let rx = current.received_bytes.saturating_sub(prev.received_bytes);
                let tx = current
                    .transmitted_bytes
                    .saturating_sub(prev.transmitted_bytes);
                total_delta_bytes += rx + tx;
            }
        }

        let seconds = self.interval.as_secs_f64();
        let mbps = (total_delta_bytes as f64 * 8.0) / (1024.0 * 1024.0 * seconds); // Megabits per second
        let mbps_data = (total_delta_bytes as f64) / (1024.0 * 1024.0 * seconds); // Megabytes per second

        self.history.push_back(mbps_data);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        let class = classify_bandwidth(mbps, &self.history, 5); // Use Mbps for classification

        Some(BandwidthStats {
            current_speed: mbps_data, // Store MB/s for display
            bandwidth_history: self.history.iter().copied().collect(),
            bandwidth_class: class,
        })
    }
}

fn classify_bandwidth(
    _mbps: f64,
    history: &std::collections::VecDeque<f64>,
    min_samples: usize,
) -> BandwidthClass {
    // Need sufficient samples for statistical significance (minimum 15 for robust analysis)
    let required_samples = 15.max(min_samples);
    if history.len() < required_samples {
        return BandwidthClass::Inconclusive;
    }

    // Get recent samples for sustained load analysis
    let recent_samples: Vec<f64> = history
        .iter()
        .rev()
        .take(required_samples)
        .copied()
        .collect();

    // Convert MB/s to Mbps for analysis
    let recent_mbps: Vec<f64> = recent_samples.iter().map(|x| x * 8.0).collect();

    // Define minimum threshold for "sustained usage" based on real-world performance research
    const SUSTAINED_USAGE_THRESHOLD_MBPS: f64 = 50.0; // 50 Mbps minimum for meaningful load testing

    // Count samples showing sustained usage
    let sustained_samples = recent_mbps
        .iter()
        .filter(|&&x| x >= SUSTAINED_USAGE_THRESHOLD_MBPS)
        .count();

    // Require 67% of samples to show sustained usage for "maxed out" detection
    let required_sustained = (required_samples * 2) / 3;
    if sustained_samples < required_sustained {
        return BandwidthClass::Inconclusive;
    }

    // Extract sustained usage values for statistical analysis
    let sustained_values: Vec<f64> = recent_mbps
        .iter()
        .filter(|&&x| x >= SUSTAINED_USAGE_THRESHOLD_MBPS)
        .copied()
        .collect();

    let mean_sustained = sustained_values.iter().sum::<f64>() / sustained_values.len() as f64;

    // Calculate coefficient of variation to detect "maxed out" behavior
    let variance = sustained_values
        .iter()
        .map(|x| (x - mean_sustained).powi(2))
        .sum::<f64>()
        / sustained_values.len() as f64;
    let std_dev = variance.sqrt();
    let coefficient_of_variation = std_dev / mean_sustained;

    // Only classify if coefficient of variation < 0.15 (low variability = sustained maxed out usage)
    if coefficient_of_variation > 0.15 {
        return BandwidthClass::Inconclusive;
    }

    // Classify based on real-world download performance thresholds
    match mean_sustained {
        x if x > 500.0 => BandwidthClass::Blazing, // >500 Mbps - Exceptional multi-gig performance
        x if x > 200.0 => BandwidthClass::Excellent, // 200-500 Mbps - Premium, maxes most services
        x if x > 100.0 => BandwidthClass::Good,    // 100-200 Mbps - Solid modern performance
        x if x > 50.0 => BandwidthClass::Fair,     // 50-100 Mbps - Adequate modern usage
        _ => BandwidthClass::Poor,                 // <50 Mbps - Below modern standards
    }
}

/// Enhanced aggregator for network snapshots into bandwidth statistics using statistical analysis.
pub struct NetAggregator<const N: usize> {
    last: Option<NetSnapshot>,
    history: [f64; N],
    history_index: usize,
    mean: f64,
    m2: f64, // For Welford's variance calculation
    count: usize,
    interval: Duration,
    high_threshold: f64, // Mbps
    cv_threshold: f64,
}

impl<const N: usize> NetAggregator<N> {
    /// Create a new aggregator with specified interval and thresholds.
    pub const fn new(interval: Duration, high_threshold: f64, cv_threshold: f64) -> Self {
        Self {
            last: None,
            history: [0.0; N],
            history_index: 0,
            mean: 0.0,
            m2: 0.0,
            count: 0,
            interval,
            high_threshold,
            cv_threshold,
        }
    }

    /// Update with a new snapshot and calculate bandwidth.
    pub fn update(&mut self, snapshot: NetSnapshot) -> Option<NetBandwidthStats<N>> {
        let last = self.last.replace(snapshot.clone())?;

        let mut total_delta_bytes = 0u64;
        for (name, current) in &snapshot.interfaces {
            if *name == "lo" {
                continue;
            }
            if let Some(prev) = last.interfaces.get(name) {
                let rx = current.received_bytes.saturating_sub(prev.received_bytes);
                let tx = current
                    .transmitted_bytes
                    .saturating_sub(prev.transmitted_bytes);
                total_delta_bytes += rx + tx;
            }
        }

        let seconds = self.interval.as_secs_f64();
        let mbps = (total_delta_bytes as f64 * 8.0) / (1024.0 * 1024.0 * seconds);

        // Welford's online algorithm for mean and variance
        self.count = self.count.saturating_add(1);
        let delta = mbps - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = mbps - self.mean;
        self.m2 += delta * delta2;

        // Update history
        self.history[self.history_index] = mbps;
        self.history_index = (self.history_index + 1) % N;

        let usage_status = if self.count >= N {
            let variance = self.m2 / N as f64;
            let sd = variance.sqrt();
            let cv = if self.mean > 0.0 {
                sd / self.mean
            } else {
                f64::INFINITY
            };
            if self.mean > self.high_threshold && cv < self.cv_threshold {
                NetUsageStatus::SteadyHigh
            } else {
                NetUsageStatus::Inconclusive
            }
        } else {
            NetUsageStatus::Inconclusive
        };

        Some(NetBandwidthStats {
            current_speed: mbps,
            bandwidth_history: self.history,
            bandwidth_class: Self::classify_bandwidth(mbps),
            usage_status,
        })
    }

    /// Classify bandwidth performance based on current speed
    #[inline]
    fn classify_bandwidth(mbps: f64) -> NetBandwidthClass {
        if mbps >= 1000.0 {
            NetBandwidthClass::Blazing // 1+ Gbps
        } else if mbps >= 100.0 {
            NetBandwidthClass::Good // 100+ Mbps
        } else if mbps >= 25.0 {
            NetBandwidthClass::Average // 25+ Mbps
        } else if mbps >= 1.0 {
            NetBandwidthClass::Poor // 1+ Mbps
        } else {
            NetBandwidthClass::Inconclusive // Below 1 Mbps or no data
        }
    }
}
