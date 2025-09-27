//! Bandwidth statistics component for detailed network performance analysis.
//!
//! This component provides comprehensive statistical analysis of network usage,
//! including performance insights, trend analysis, and optimization recommendations.

use crossbeam_channel::Sender;
use std::collections::BTreeMap;
use std::time::Duration;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    action::Action,
    concurrent::tickable::Tickable,
    display::UIState,
    network::{BandwidthClass, BandwidthStats},
};

/// Component for displaying comprehensive bandwidth statistics and analysis
#[derive(Debug)]
pub struct BandwidthStatsComponent {
    /// Historical bandwidth samples for trend analysis
    bandwidth_history: Vec<f64>,
    /// Maximum recorded bandwidth
    peak_bandwidth: f64,
    /// Average bandwidth over time
    average_bandwidth: f64,
    /// Current efficiency rating (0.0 - 1.0)
    efficiency_score: f32,
    /// Top bandwidth consumers
    top_consumers: Vec<ConsumerInfo>,
    /// Performance insights
    insights: Vec<Insight>,
    /// Channel sender for triggering UI updates
    action_sender: Option<Sender<Action>>,
}

/// Information about a bandwidth consumer
#[derive(Debug, Clone)]
struct ConsumerInfo {
    /// Process or connection name
    name: String,
    /// Total bytes transferred
    total_bytes: u128,
    /// Percentage of total bandwidth
    percentage: f32,
    /// Upload/download ratio
    ratio: f32,
}

/// Performance insight or recommendation
#[derive(Debug, Clone)]
struct Insight {
    /// Severity level
    severity: InsightSeverity,
    /// Insight message
    message: String,
}

/// Severity levels for insights
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsightSeverity {
    Info,
    Suggestion,
    Warning,
}

impl BandwidthStatsComponent {
    /// Create a new bandwidth statistics component
    #[inline]
    pub fn new() -> Self {
        Self {
            bandwidth_history: Vec::new(),
            peak_bandwidth: 0.0,
            average_bandwidth: 0.0,
            efficiency_score: 1.0,
            top_consumers: Vec::new(),
            insights: Vec::new(),
            action_sender: None,
        }
    }

    /// Set the action sender for triggering UI updates
    pub fn set_action_sender(&mut self, sender: Sender<Action>) {
        self.action_sender = Some(sender);
        self.start_update_timer();
    }

    /// Start internal timer to send update events every 500ms
    fn start_update_timer(&self) {
        if let Some(sender) = &self.action_sender {
            let sender_clone = sender.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(500));
                loop {
                    interval.tick().await;
                    if sender_clone.send(Action::StatsUpdate).is_err() {
                        // Channel closed, stop timer
                        break;
                    }
                }
            });
        }
    }

    /// Update component with latest bandwidth statistics
    #[inline]
    pub fn update_stats(&mut self, stats: &BandwidthStats, ui_state: &UIState) {
        // Update bandwidth history (keep last 60 samples for 1-minute window)
        self.bandwidth_history.push(stats.current_speed);
        if self.bandwidth_history.len() > 60 {
            self.bandwidth_history.remove(0);
        }

        // Update peak bandwidth
        if stats.current_speed > self.peak_bandwidth {
            self.peak_bandwidth = stats.current_speed;
        }

        // Calculate average bandwidth
        if !self.bandwidth_history.is_empty() {
            let sum: f64 = self.bandwidth_history.iter().sum();
            self.average_bandwidth = sum / self.bandwidth_history.len() as f64;
        }

        // Calculate efficiency score based on utilization patterns
        self.calculate_efficiency_score(stats);

        // Update top consumers
        self.update_top_consumers(ui_state);

        // Generate insights
        self.generate_insights(stats, ui_state);
    }

    /// Update internal display state and request render
    pub fn update_display_state(&mut self) {
        // Update insights based on current patterns
        if !self.bandwidth_history.is_empty() {
            self.update_time_based_insights();
        } else {
            // Show waiting message when no data is available
            self.insights.clear();
            self.insights.push(Insight {
                severity: InsightSeverity::Info,
                message: "📊 Waiting for network data...".to_string(),
            });
        }

        // Recalculate efficiency score based on current bandwidth patterns
        if !self.bandwidth_history.is_empty() {
            let variance = self.calculate_variance();
            let consistency_score = 1.0 / (1.0 + variance.sqrt() / 100.0);

            // Update efficiency score smoothly
            let current_utilization = if self.average_bandwidth > 0.0 {
                (self.bandwidth_history.last().copied().unwrap_or(0.0) / self.average_bandwidth)
                    .min(1.0) as f32
            } else {
                1.0
            };

            // Weighted efficiency calculation
            let new_efficiency =
                (consistency_score * 0.6 + current_utilization * 0.4).clamp(0.0, 1.0);

            // Smooth efficiency updates to prevent jarring changes
            self.efficiency_score = self.efficiency_score * 0.8 + new_efficiency * 0.2;
        }
    }

    /// Calculate network efficiency score
    #[inline]
    fn calculate_efficiency_score(&mut self, stats: &BandwidthStats) {
        // Efficiency based on:
        // 1. Bandwidth utilization consistency (less spiky = more efficient)
        // 2. Current speed vs average (steady usage = more efficient)
        // 3. Bandwidth class stability

        let variance = self.calculate_variance();
        let consistency_score = 1.0 / (1.0 + variance.sqrt() / 100.0);

        let utilization_score = if self.average_bandwidth > 0.0 {
            (stats.current_speed / self.average_bandwidth).min(1.0) as f32
        } else {
            1.0
        };

        // Weight the scores
        self.efficiency_score = (consistency_score * 0.6 + utilization_score * 0.4).clamp(0.0, 1.0);
    }

    /// Calculate variance of bandwidth history
    #[inline]
    fn calculate_variance(&self) -> f32 {
        if self.bandwidth_history.len() < 2 {
            return 0.0;
        }

        let mean = self.average_bandwidth;
        let variance: f64 = self
            .bandwidth_history
            .iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.bandwidth_history.len() as f64;

        variance as f32
    }

    /// Update top bandwidth consumers
    #[inline]
    fn update_top_consumers(&mut self, ui_state: &UIState) {
        // Clear previous consumers
        self.top_consumers.clear();

        // Aggregate by process
        let mut process_bytes: BTreeMap<String, (u128, u128)> = BTreeMap::new();

        for (process_info, network_data) in &ui_state.processes {
            let name = process_info.name.clone();
            let entry = process_bytes.entry(name).or_insert((0, 0));
            entry.0 += network_data.total_bytes_uploaded;
            entry.1 += network_data.total_bytes_downloaded;
        }

        // Calculate total bytes
        let total_bytes: u128 = process_bytes.values().map(|(up, down)| up + down).sum();

        // Convert to ConsumerInfo and sort by total bytes
        let mut consumers: Vec<ConsumerInfo> = process_bytes
            .into_iter()
            .map(|(name, (uploaded, downloaded))| {
                let total = uploaded + downloaded;
                let percentage = if total_bytes > 0 {
                    (total as f64 / total_bytes as f64 * 100.0) as f32
                } else {
                    0.0
                };
                let ratio = if downloaded > 0 {
                    uploaded as f32 / downloaded as f32
                } else {
                    1.0
                };

                ConsumerInfo {
                    name,
                    total_bytes: total,
                    percentage,
                    ratio,
                }
            })
            .collect();

        // Sort by total bytes (descending) and keep top 5
        consumers.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        consumers.truncate(5);

        self.top_consumers = consumers;
    }

    /// Generate performance insights
    #[inline]
    fn generate_insights(&mut self, stats: &BandwidthStats, ui_state: &UIState) {
        self.insights.clear();

        // Bandwidth class insight
        match stats.bandwidth_class {
            BandwidthClass::Excellent => {
                self.insights.push(Insight {
                    severity: InsightSeverity::Info,
                    message: "🌟 Network performance is outstanding!".to_string(),
                });
            }
            BandwidthClass::Blazing => {
                self.insights.push(Insight {
                    severity: InsightSeverity::Info,
                    message: "🚀 Network performance is excellent!".to_string(),
                });
            }
            BandwidthClass::Good => {
                self.insights.push(Insight {
                    severity: InsightSeverity::Info,
                    message: "✓ Network performance is good".to_string(),
                });
            }
            BandwidthClass::Fair => {
                self.insights.push(Insight {
                    severity: InsightSeverity::Suggestion,
                    message: "⚡ Network performance could be improved".to_string(),
                });
            }
            BandwidthClass::Poor => {
                self.insights.push(Insight {
                    severity: InsightSeverity::Warning,
                    message: "⚠️ Network performance is poor".to_string(),
                });
            }
            BandwidthClass::Inconclusive => {
                self.insights.push(Insight {
                    severity: InsightSeverity::Info,
                    message: "📊 Gathering more data for analysis...".to_string(),
                });
            }
        }

        // Efficiency insight
        if self.efficiency_score < 0.5 {
            self.insights.push(Insight {
                severity: InsightSeverity::Warning,
                message: "Network usage is highly irregular - consider load balancing".to_string(),
            });
        } else if self.efficiency_score > 0.8 {
            self.insights.push(Insight {
                severity: InsightSeverity::Info,
                message: "Network utilization is stable and efficient".to_string(),
            });
        }

        // Top consumer insight
        if let Some(top_consumer) = self.top_consumers.first() {
            if top_consumer.percentage > 50.0 {
                self.insights.push(Insight {
                    severity: InsightSeverity::Suggestion,
                    message: format!(
                        "{} is using {:.1}% of bandwidth",
                        top_consumer.name, top_consumer.percentage
                    ),
                });
            }
        }

        // Connection count insight
        let connection_count = ui_state.connections.len();
        if connection_count > 100 {
            self.insights.push(Insight {
                severity: InsightSeverity::Warning,
                message: format!("High number of connections: {}", connection_count),
            });
        }

        // Upload/download ratio insight
        let total_uploaded: u128 = ui_state
            .processes
            .iter()
            .map(|(_, data)| data.total_bytes_uploaded)
            .sum();
        let total_downloaded: u128 = ui_state
            .processes
            .iter()
            .map(|(_, data)| data.total_bytes_downloaded)
            .sum();

        if total_downloaded > 0 {
            let ratio = total_uploaded as f64 / total_downloaded as f64;
            if ratio > 2.0 {
                self.insights.push(Insight {
                    severity: InsightSeverity::Info,
                    message: "Upload-heavy usage detected (server/seeding activity)".to_string(),
                });
            } else if ratio < 0.1 {
                self.insights.push(Insight {
                    severity: InsightSeverity::Info,
                    message: "Download-heavy usage detected".to_string(),
                });
            }
        }
    }

    /// Render the component
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Split area into sections
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(8),  // Overview stats
                Constraint::Length(10), // Top consumers
                Constraint::Min(8),     // Insights
                Constraint::Length(6),  // Efficiency gauge
            ])
            .split(area);

        // Render title
        self.render_title(frame, chunks[0]);

        // Render overview statistics
        self.render_overview(frame, chunks[1]);

        // Render top consumers
        self.render_top_consumers(frame, chunks[2]);

        // Render insights
        self.render_insights(frame, chunks[3]);

        // Render efficiency gauge
        self.render_efficiency_gauge(frame, chunks[4]);
    }

    /// Render component title
    #[inline]
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("📊 Bandwidth Statistics & Analysis")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)),
            );

        frame.render_widget(title, area);
    }

    /// Render overview statistics
    #[inline]
    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        let current_speed = self.bandwidth_history.last().copied().unwrap_or(0.0);

        let stats_text = vec![
            Line::from(vec![
                Span::styled("Current Speed: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.2} MB/s", current_speed),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Average Speed: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.2} MB/s", self.average_bandwidth),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("Peak Speed: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.2} MB/s", self.peak_bandwidth),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(vec![
                Span::styled("Samples: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}/60", self.bandwidth_history.len()),
                    Style::default().fg(Color::Blue),
                ),
            ]),
        ];

        let overview = Paragraph::new(stats_text).block(
            Block::default()
                .title(" Overview ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(overview, area);
    }

    /// Render top bandwidth consumers
    #[inline]
    fn render_top_consumers(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .top_consumers
            .iter()
            .enumerate()
            .map(|(i, consumer)| {
                let color = match i {
                    0 => Color::Red,
                    1 => Color::Yellow,
                    2 => Color::Green,
                    _ => Color::Gray,
                };

                let content = vec![
                    Span::styled(
                        format!("{:2}. ", i + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{:<16}", truncate_string(&consumer.name, 16)),
                        Style::default().fg(color),
                    ),
                    Span::styled(
                        format!("{:>8}", format_bytes(consumer.total_bytes)),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!(" ({:>4.1}%)", consumer.percentage),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!(" {:>4.1}:1", consumer.ratio),
                        Style::default().fg(Color::Cyan),
                    ),
                ];

                ListItem::new(Line::from(content))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Top Consumers ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(list, area);
    }

    /// Render performance insights
    #[inline]
    fn render_insights(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .insights
            .iter()
            .map(|insight| {
                let (symbol, color) = match insight.severity {
                    InsightSeverity::Info => ("ℹ ", Color::Blue),
                    InsightSeverity::Suggestion => ("💡", Color::Yellow),
                    InsightSeverity::Warning => ("⚠️ ", Color::Red),
                };

                let content = vec![
                    Span::styled(symbol, Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(&insight.message, Style::default().fg(Color::White)),
                ];

                ListItem::new(Line::from(content))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Insights & Recommendations ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(list, area);
    }

    /// Render efficiency gauge
    #[inline]
    fn render_efficiency_gauge(&self, frame: &mut Frame, area: Rect) {
        let color = if self.efficiency_score > 0.8 {
            Color::Green
        } else if self.efficiency_score > 0.5 {
            Color::Yellow
        } else {
            Color::Red
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(" Network Efficiency ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .gauge_style(Style::default().fg(color))
            .percent((self.efficiency_score * 100.0) as u16)
            .label(format!("{:.0}%", self.efficiency_score * 100.0));

        frame.render_widget(gauge, area);
    }
}

/// Format bytes into human-readable string
#[inline]
fn format_bytes(bytes: u128) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

/// Truncate string to fit within given width
#[inline]
fn truncate_string(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 3 {
        format!("{}...", &s[..max_width - 3])
    } else {
        s[..max_width].to_string()
    }
}

impl BandwidthStatsComponent {
    /// Update insights based on time-based patterns
    fn update_time_based_insights(&mut self) {
        // Clear old insights
        self.insights.clear();

        // Only generate insights if we have sufficient data
        if self.bandwidth_history.len() < 3 {
            return;
        }

        // Analyze recent trend
        let recent_samples = self
            .bandwidth_history
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect::<Vec<_>>();
        let trend_increasing = recent_samples.windows(2).all(|w| w[0] <= w[1]);
        let trend_decreasing = recent_samples.windows(2).all(|w| w[0] >= w[1]);

        if trend_increasing {
            self.insights.push(Insight {
                severity: InsightSeverity::Info,
                message: "📈 Bandwidth usage is trending upward".to_string(),
            });
        } else if trend_decreasing {
            self.insights.push(Insight {
                severity: InsightSeverity::Info,
                message: "📉 Bandwidth usage is trending downward".to_string(),
            });
        }

        // Efficiency analysis
        if self.efficiency_score < 0.4 {
            self.insights.push(Insight {
                severity: InsightSeverity::Warning,
                message: "⚠️ Network efficiency is poor - consider optimizing usage patterns"
                    .to_string(),
            });
        } else if self.efficiency_score > 0.8 {
            self.insights.push(Insight {
                severity: InsightSeverity::Info,
                message: "✨ Network usage is highly efficient".to_string(),
            });
        }

        // Peak usage analysis
        let current_speed = self.bandwidth_history.last().copied().unwrap_or(0.0);
        if current_speed > self.peak_bandwidth * 0.9 {
            self.insights.push(Insight {
                severity: InsightSeverity::Suggestion,
                message: "🚀 Approaching peak bandwidth usage".to_string(),
            });
        }
    }
}

impl Default for BandwidthStatsComponent {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Tickable for BandwidthStatsComponent {
    fn tick(&mut self) {
        // Update display state and refresh insights
        self.update_display_state();
    }

    fn is_active(&self) -> bool {
        // Always active since it provides ongoing statistical analysis
        true
    }

    fn frame_interval(&self) -> Duration {
        // Update every 500ms for statistics refresh
        Duration::from_millis(500)
    }
}
