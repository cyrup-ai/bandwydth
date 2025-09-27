use std::collections::VecDeque;
use std::time::Instant;

/// Rich bandwidth data point with metadata for visual effects
#[derive(Clone, Debug)]
pub struct BandwidthPoint {
    /// Bandwidth speed in MB/s
    pub speed_mbps: f64,
    /// Download bytes for this measurement
    pub download_bytes: u64,
    /// Download rate in MB/s
    pub download_rate: f64,
    /// Effect intensity (0.0-1.0) for animation strength
    pub intensity: f64,
    /// When this data point was recorded
    pub timestamp: Instant,
    /// Whether this represents a peak value
    pub is_peak: bool,
}

impl BandwidthPoint {
    /// Create a new bandwidth point from bandwidth stats and optional utilization
    pub fn from_stats(
        stats: &crate::network::BandwidthStats,
        utilization: Option<&crate::network::Utilization>,
    ) -> Self {
        let speed_mbps = stats.current_speed;
        let intensity = (speed_mbps / 100.0).min(1.0); // Normalize to 0-1, cap at 100MB/s

        // Extract real download metrics from utilization data
        let (download_bytes, download_rate) = if let Some(util) = utilization {
            let total_download_bytes: u128 = util
                .connections
                .values()
                .map(|conn| conn.total_bytes_downloaded)
                .sum();

            let total_recv_bytes: u64 = util.connections.values().map(|conn| conn.recv_bytes).sum();

            (
                total_download_bytes as u64,
                total_recv_bytes as f64 / (1024.0 * 1024.0),
            ) // Convert to MB
        } else {
            // Fallback: use interface-level speed as download rate estimate
            (0, speed_mbps)
        };

        Self {
            speed_mbps,
            download_bytes,
            download_rate,
            intensity,
            timestamp: Instant::now(),
            is_peak: false, // Will be determined when adding to history
        }
    }

    /// Get the bandwidth classification level
    pub fn bandwidth_level(&self) -> BandwidthLevel {
        match self.speed_mbps {
            s if s < 10.0 => BandwidthLevel::Low,
            s if s < 50.0 => BandwidthLevel::Medium,
            s if s < 100.0 => BandwidthLevel::High,
            _ => BandwidthLevel::Critical,
        }
    }

    /// Get sparkline character for this data point (0-8 scale)
    pub fn sparkline_char(&self) -> char {
        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let index = ((self.speed_mbps / 12.5).min(7.0) as usize).min(7);
        chars[index]
    }
}

/// Bandwidth classification levels for color mapping
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BandwidthLevel {
    /// Low bandwidth (0-10 MB/s) - Blue
    Low,
    /// Medium bandwidth (10-50 MB/s) - Cyan  
    Medium,
    /// High bandwidth (50-100 MB/s) - Yellow
    High,
    /// Critical bandwidth (100+ MB/s) - Red
    Critical,
}

impl BandwidthLevel {
    /// Get base hue for this bandwidth level (0-360 degrees)
    pub fn base_hue(&self) -> f32 {
        match self {
            BandwidthLevel::Low => 240.0,    // Deep Blue
            BandwidthLevel::Medium => 180.0, // Cyan
            BandwidthLevel::High => 60.0,    // Yellow
            BandwidthLevel::Critical => 0.0, // Red
        }
    }

    /// Get saturation percentage for this level
    pub fn saturation(&self) -> f32 {
        match self {
            BandwidthLevel::Low => 70.0,
            BandwidthLevel::Medium => 80.0,
            BandwidthLevel::High => 90.0,
            BandwidthLevel::Critical => 100.0,
        }
    }

    /// Get base lightness percentage for this level
    pub fn lightness(&self) -> f32 {
        match self {
            BandwidthLevel::Low => 60.0,
            BandwidthLevel::Medium => 65.0,
            BandwidthLevel::High => 70.0,
            BandwidthLevel::Critical => 75.0,
        }
    }
}

/// Manages bandwidth data history with smoothing and peak detection
#[derive(Debug)]
pub struct BandwidthDataManager {
    /// Rolling history of bandwidth points (max 60 for smooth curves)
    history: VecDeque<BandwidthPoint>,
    /// Maximum number of data points to keep
    max_points: usize,
    /// Last recorded peak value
    session_peak: f64,
    /// Positions of peak markers
    peak_positions: Vec<usize>,
}

impl BandwidthDataManager {
    /// Create a new data manager
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(60),
            max_points: 60,
            session_peak: 0.0,
            peak_positions: Vec::new(),
        }
    }

    /// Add a new bandwidth data point
    pub fn add_point(&mut self, mut point: BandwidthPoint) {
        // Check if this is a new peak
        if point.speed_mbps > self.session_peak {
            self.session_peak = point.speed_mbps;
            point.is_peak = true;
            self.peak_positions.push(self.history.len());
        }

        // Add to history
        self.history.push_back(point);

        // Maintain max size
        if self.history.len() > self.max_points {
            self.history.pop_front();
            // Adjust peak positions
            self.peak_positions.iter_mut().for_each(|pos| {
                if *pos > 0 {
                    *pos -= 1;
                }
            });
            self.peak_positions.retain(|&pos| pos > 0);
        }
    }

    /// Get the current data history
    pub fn history(&self) -> &VecDeque<BandwidthPoint> {
        &self.history
    }

    /// Get current bandwidth level based on latest point
    pub fn current_level(&self) -> Option<BandwidthLevel> {
        self.history.back().map(|point| point.bandwidth_level())
    }

    /// Get session peak speed
    pub fn session_peak(&self) -> f64 {
        self.session_peak
    }

    /// Get peak marker positions
    pub fn peak_positions(&self) -> &[usize] {
        &self.peak_positions
    }

    /// Get sparkline string representation
    pub fn sparkline_string(&self) -> String {
        self.history
            .iter()
            .map(|point| point.sparkline_char())
            .collect()
    }

    /// Get the latest bandwidth point
    pub fn latest_point(&self) -> Option<&BandwidthPoint> {
        self.history.back()
    }

    /// Check if bandwidth level has changed significantly
    pub fn level_changed(&self) -> bool {
        if self.history.len() < 2 {
            return false;
        }

        let current = match self.history.back() {
            Some(point) => point.bandwidth_level(),
            None => return false,
        };

        let previous = match self.history.get(self.history.len() - 2) {
            Some(point) => point.bandwidth_level(),
            None => return false,
        };

        current != previous
    }
}

impl Default for BandwidthDataManager {
    fn default() -> Self {
        Self::new()
    }
}
