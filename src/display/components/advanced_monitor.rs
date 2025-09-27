//! Advanced monitoring component with historical trends and DNS resolution.
//!
//! This component provides detailed bandwidth monitoring with trend analysis,
//! peak tracking, and integrated DNS hostname resolution for network connections.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::time::Duration;

use crate::{
    concurrent::tickable::Tickable,
    network::{BandwidthClass, BandwidthStats, Snapshot},
};

/// Historical bandwidth measurement with metadata
#[derive(Debug, Clone)]
pub struct BandwidthMeasurement {
    /// Speed in MB/s
    pub speed: f64,
    /// Timestamp when measurement was taken
    pub timestamp: std::time::SystemTime,
    /// Number of active connections at this time
    pub connection_count: usize,
    /// Total bytes transferred at this measurement
    pub total_bytes: u64,
}

/// Advanced monitoring component state
pub struct AdvancedMonitorComponent {
    /// Historical bandwidth measurements (last 60 seconds)
    bandwidth_history: VecDeque<BandwidthMeasurement>,
    /// DNS hostname cache
    hostname_cache: HashMap<IpAddr, String>,
    /// Peak bandwidth this session
    peak_bandwidth: f64,
    /// Total measurement count
    measurement_count: u32,
    /// Session start time
    session_start: std::time::SystemTime,
    /// Current bandwidth classification
    current_class: BandwidthClass,
    /// Maximum history length (samples to keep)
    max_history: usize,
}

impl AdvancedMonitorComponent {
    /// Create a new advanced monitor component
    #[inline]
    pub fn new() -> Self {
        Self {
            bandwidth_history: VecDeque::with_capacity(60),
            hostname_cache: HashMap::with_capacity(256),
            peak_bandwidth: 0.0,
            measurement_count: 0,
            session_start: std::time::SystemTime::now(),
            current_class: BandwidthClass::Inconclusive,
            max_history: 60, // Keep 60 seconds of data
        }
    }

    /// Update with new bandwidth statistics
    #[inline]
    pub fn update_stats(&mut self, stats: &BandwidthStats) {
        self.measurement_count += 1;

        // Update peak bandwidth
        if stats.current_speed > self.peak_bandwidth {
            self.peak_bandwidth = stats.current_speed;
        }

        // Update current classification
        self.current_class = stats.bandwidth_class.clone();

        // Add new measurement to history
        let measurement = BandwidthMeasurement {
            speed: stats.current_speed,
            timestamp: std::time::SystemTime::now(),
            connection_count: 0, // Will be updated with network snapshot
            total_bytes: 0,      // Will be updated with network snapshot
        };

        self.bandwidth_history.push_back(measurement);

        // Trim history to max length
        while self.bandwidth_history.len() > self.max_history {
            self.bandwidth_history.pop_front();
        }
    }

    /// Update with new network snapshot
    #[inline]
    pub fn update_snapshot(&mut self, snapshot: &Snapshot) {
        // Update the latest measurement with connection info
        if let Some(latest) = self.bandwidth_history.back_mut() {
            latest.connection_count = snapshot.interfaces.len();
            latest.total_bytes = snapshot
                .interfaces
                .values()
                .map(|interface| interface.received_bytes + interface.transmitted_bytes)
                .sum();
        }

        // DNS resolution is handled by the main UI update_dns_cache method
    }

    /// Update hostname cache with DNS resolution results
    #[inline]
    pub fn update_hostname(&mut self, ip: IpAddr, hostname: String) {
        self.hostname_cache.insert(ip, hostname);
    }

    /// Render the advanced monitor view
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let main_block = Block::default()
            .title(" Advanced Bandwidth Monitor ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        let inner_area = main_block.inner(area);
        frame.render_widget(main_block, area);

        // Create layout: header, trend graph, statistics, DNS resolution
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header with current stats
                Constraint::Length(8), // Trend graph
                Constraint::Length(6), // Session statistics
                Constraint::Min(1),    // DNS/connection info
            ])
            .split(inner_area);

        // Render each section
        self.render_header(frame, chunks[0]);
        self.render_trend_graph(frame, chunks[1]);
        self.render_session_stats(frame, chunks[2]);
        self.render_dns_info(frame, chunks[3]);
    }

    /// Render the current bandwidth header
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let current_speed = self
            .bandwidth_history
            .back()
            .map(|m| m.speed)
            .unwrap_or(0.0);

        let class_color = match self.current_class {
            BandwidthClass::Poor => Color::Red,
            BandwidthClass::Fair => Color::Yellow,
            BandwidthClass::Good => Color::Blue,
            BandwidthClass::Excellent => Color::Green,
            BandwidthClass::Blazing => Color::Magenta,
            BandwidthClass::Inconclusive => Color::DarkGray,
        };

        let trend_indicator = self.get_trend_indicator();
        let (trend_text, trend_color) = match trend_indicator {
            TrendDirection::Increasing => ("↑ Increasing", Color::Green),
            TrendDirection::Decreasing => ("↓ Decreasing", Color::Red),
            TrendDirection::Stable => ("→ Stable", Color::Yellow),
        };

        let content = vec![
            Line::from(vec![
                Span::styled("Current Speed: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.2} MB/s", current_speed),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" | Quality: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:?}", self.current_class),
                    Style::default()
                        .fg(class_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Trend: ", Style::default().fg(Color::White)),
                Span::styled(
                    trend_text,
                    Style::default()
                        .fg(trend_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" | Measurements: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("#{}", self.measurement_count),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Real-time Stats"),
        );
        frame.render_widget(paragraph, area);
    }

    /// Render the bandwidth trend graph
    fn render_trend_graph(&self, frame: &mut Frame, area: Rect) {
        if self.bandwidth_history.is_empty() {
            let loading = Paragraph::new("Collecting bandwidth data...")
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Bandwidth Trend"),
                );
            frame.render_widget(loading, area);
            return;
        }

        // Convert bandwidth history to sparkline data
        let data: Vec<u64> = self
            .bandwidth_history
            .iter()
            .map(|m| (m.speed * 100.0) as u64) // Scale for sparkline
            .collect();

        let max_value = data.iter().max().copied().unwrap_or(1);

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Bandwidth History (60s)"),
            )
            .data(&data)
            .max(max_value)
            .style(Style::default().fg(Color::Green));

        frame.render_widget(sparkline, area);
    }

    /// Render session statistics
    fn render_session_stats(&self, frame: &mut Frame, area: Rect) {
        let session_duration = self.session_start.elapsed().unwrap_or_default().as_secs();

        let avg_speed = if !self.bandwidth_history.is_empty() {
            self.bandwidth_history.iter().map(|m| m.speed).sum::<f64>()
                / self.bandwidth_history.len() as f64
        } else {
            0.0
        };

        let content = vec![
            Line::from(vec![
                Span::styled("Session Duration: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}m {}s", session_duration / 60, session_duration % 60),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(" | Peak Speed: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.2} MB/s", self.peak_bandwidth),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Average Speed: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.2} MB/s", avg_speed),
                    Style::default().fg(Color::Blue),
                ),
                Span::styled(" | Samples: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", self.bandwidth_history.len()),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ];

        // Create a simple gauge for peak vs current speed
        let current_speed = self
            .bandwidth_history
            .back()
            .map(|m| m.speed)
            .unwrap_or(0.0);

        let gauge_ratio = if self.peak_bandwidth > 0.0 {
            (current_speed / self.peak_bandwidth).min(1.0)
        } else {
            0.0
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Stats text
                Constraint::Length(1), // Gauge
            ])
            .split(area);

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Session Statistics"),
        );
        frame.render_widget(paragraph, layout[0]);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Current vs Peak"),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(gauge_ratio);
        frame.render_widget(gauge, layout[1]);
    }

    /// Render DNS resolution information
    fn render_dns_info(&self, frame: &mut Frame, area: Rect) {
        let mut content = vec![Line::from(vec![
            Span::styled("DNS Cache: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{} hostnames resolved", self.hostname_cache.len()),
                Style::default().fg(Color::Cyan),
            ),
        ])];

        // Show some recent DNS resolutions
        let mut dns_entries: Vec<_> = self.hostname_cache.iter().collect();
        dns_entries.sort_by_key(|(ip, _)| ip.to_string());

        for (ip, hostname) in dns_entries.iter().take(5) {
            content.push(Line::from(vec![
                Span::styled(format!("  {} → ", ip), Style::default().fg(Color::Blue)),
                Span::styled(hostname.as_str(), Style::default().fg(Color::Green)),
            ]));
        }

        if dns_entries.len() > 5 {
            content.push(Line::from(vec![Span::styled(
                format!("  ... and {} more", dns_entries.len() - 5),
                Style::default().fg(Color::DarkGray),
            )]));
        }

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .title("DNS Resolution"),
        );
        frame.render_widget(paragraph, area);
    }

    /// Calculate trend direction from recent measurements
    fn get_trend_indicator(&self) -> TrendDirection {
        if self.bandwidth_history.len() < 3 {
            return TrendDirection::Stable;
        }

        let recent: Vec<f64> = self
            .bandwidth_history
            .iter()
            .rev()
            .take(3)
            .map(|m| m.speed)
            .collect();

        let diff_threshold = 0.1; // MB/s threshold for trend detection

        if recent[0] > recent[2] + diff_threshold {
            TrendDirection::Increasing
        } else if recent[0] < recent[2] - diff_threshold {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
}

impl Default for AdvancedMonitorComponent {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Tickable for AdvancedMonitorComponent {
    fn tick(&mut self) {
        // Update bandwidth trend indicators based on current history
        if self.bandwidth_history.len() > 1 {
            // Calculate moving averages for trend analysis
            let recent_window = self
                .bandwidth_history
                .iter()
                .rev()
                .take(5)
                .map(|m| m.speed)
                .collect::<Vec<f64>>();

            if recent_window.len() >= 2 {
                let current_avg = recent_window.iter().sum::<f64>() / recent_window.len() as f64;
                let older_avg = if self.bandwidth_history.len() >= 10 {
                    self.bandwidth_history
                        .iter()
                        .rev()
                        .skip(5)
                        .take(5)
                        .map(|m| m.speed)
                        .sum::<f64>()
                        / 5.0
                } else {
                    current_avg
                };

                // Update trend analysis with smoothed calculations
                let trend_strength = (current_avg - older_avg).abs();
                if trend_strength > 0.5 {
                    // Significant trend detected - this will be used by render methods
                    // No need to store trend state as get_trend_indicator() calculates it dynamically
                }
            }
        }

        // Update session duration for real-time display
        // Session start time is already tracked, duration is calculated in render_session_stats

        // DNS cache timing updates are handled by the main UI update_dns_cache method
        // We don't need to modify DNS cache timing here as it's managed externally
    }

    fn is_active(&self) -> bool {
        // Component is active if it has bandwidth history, DNS cache entries, or measurements
        !self.bandwidth_history.is_empty()
            || !self.hostname_cache.is_empty()
            || self.measurement_count > 0
    }

    fn frame_interval(&self) -> Duration {
        // Update trend analysis every 500ms as requested by user
        Duration::from_millis(500)
    }
}

/// Bandwidth trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}
