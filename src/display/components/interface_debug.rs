//! Interface debugging component for detailed per-interface network statistics.
//!
//! This component provides a detailed view of all network interfaces with
//! real-time byte counts, helping diagnose connectivity and performance issues.

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use std::collections::HashMap;
use std::time::Duration;

use crate::{concurrent::tickable::Tickable, network::Snapshot};

/// Interface debug component state
pub struct InterfaceDebugComponent {
    /// Current network interface snapshot
    current_snapshot: Option<Snapshot>,
    /// Interface activity indicators
    active_interfaces: HashMap<String, bool>,
}

impl InterfaceDebugComponent {
    /// Create a new interface debug component
    #[inline]
    pub fn new() -> Self {
        Self {
            current_snapshot: None,
            active_interfaces: HashMap::with_capacity(32),
        }
    }

    /// Update with a new network snapshot
    #[inline]
    pub fn update(&mut self, snapshot: Snapshot) {
        // Mark interfaces as active if they have data
        for (name, interface) in &snapshot.interfaces {
            let has_activity = interface.received_bytes > 0 || interface.transmitted_bytes > 0;
            self.active_interfaces.insert(name.clone(), has_activity);
        }

        self.current_snapshot = Some(snapshot);
    }

    /// Render the interface debug view
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let main_block = Block::default()
            .title(" Network Interface Debug ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        let inner_area = main_block.inner(area);
        frame.render_widget(main_block, area);

        // Create layout for header and table
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Table
            ])
            .split(inner_area);

        // Render header with summary
        self.render_header(frame, chunks[0]);

        // Render interface table
        self.render_interface_table(frame, chunks[1]);
    }

    /// Render the header with interface summary
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let content = if let Some(ref snapshot) = self.current_snapshot {
            let total_interfaces = snapshot.interfaces.len();
            let active_count = self
                .active_interfaces
                .values()
                .filter(|&&active| active)
                .count();
            let total_rx = snapshot
                .interfaces
                .values()
                .map(|interface| interface.received_bytes)
                .sum::<u64>();
            let total_tx = snapshot
                .interfaces
                .values()
                .map(|interface| interface.transmitted_bytes)
                .sum::<u64>();

            vec![
                Line::from(vec![
                    Span::styled("Found ", Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{total_interfaces}"),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" network interfaces (", Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{active_count}"),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" active)", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Total: ", Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{} bytes RX", format_bytes(total_rx)),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} bytes TX", format_bytes(total_tx)),
                        Style::default().fg(Color::Red),
                    ),
                ]),
            ]
        } else {
            vec![Line::from(vec![Span::styled(
                "Loading network interfaces...",
                Style::default().fg(Color::Yellow),
            )])]
        };

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, area);
    }

    /// Render the interface statistics table
    fn render_interface_table(&self, frame: &mut Frame, area: Rect) {
        if let Some(ref snapshot) = self.current_snapshot {
            let header = Row::new(vec![
                Cell::from("Interface").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Received").style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Transmitted")
                    .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Cell::from("Total").style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Status").style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);

            let mut interfaces: Vec<_> = snapshot.interfaces.iter().collect();
            // Sort by total activity (most active first)
            interfaces.sort_by(|a, b| {
                let total_a = a.1.received_bytes + a.1.transmitted_bytes;
                let total_b = b.1.received_bytes + b.1.transmitted_bytes;
                total_b.cmp(&total_a)
            });

            let rows: Vec<Row> = interfaces
                .into_iter()
                .map(|(name, interface)| {
                    let is_active = self.active_interfaces.get(name).copied().unwrap_or(false);
                    let total_bytes = interface.received_bytes + interface.transmitted_bytes;

                    let status_style = if is_active {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let status_text = if is_active { "Active" } else { "Inactive" };

                    Row::new(vec![
                        Cell::from(name.as_str()).style(if is_active {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        }),
                        Cell::from(format_bytes(interface.received_bytes))
                            .style(Style::default().fg(Color::Blue)),
                        Cell::from(format_bytes(interface.transmitted_bytes))
                            .style(Style::default().fg(Color::Red)),
                        Cell::from(format_bytes(total_bytes)).style(if total_bytes > 0 {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        }),
                        Cell::from(status_text).style(status_style),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(16), // Interface name
                    Constraint::Length(15), // Received
                    Constraint::Length(15), // Transmitted
                    Constraint::Length(15), // Total
                    Constraint::Length(10), // Status
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)),
            );

            frame.render_widget(table, area);
        } else {
            let loading = Paragraph::new("Loading interface data...")
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Gray)),
                );
            frame.render_widget(loading, area);
        }
    }
}

impl Default for InterfaceDebugComponent {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Tickable for InterfaceDebugComponent {
    fn tick(&mut self) {
        // Update interface activity tracking
        // This helps smooth out the active/inactive status display
        if let Some(ref snapshot) = self.current_snapshot {
            // Check for recent activity changes
            let mut activity_changed = false;

            for (name, interface) in &snapshot.interfaces {
                let has_activity = interface.received_bytes > 0 || interface.transmitted_bytes > 0;
                let was_active = self.active_interfaces.get(name).copied().unwrap_or(false);

                if has_activity != was_active {
                    activity_changed = true;
                    self.active_interfaces.insert(name.clone(), has_activity);
                }
            }

            // If activity changed, the component needs to re-render
            if activity_changed {
                // Activity changes will trigger re-render through the update system
            }
        }
    }

    fn is_active(&self) -> bool {
        // Component is active if it has interface data to display
        self.current_snapshot.is_some()
    }

    fn frame_interval(&self) -> Duration {
        // Update interface activity status every 1000ms as requested
        Duration::from_millis(1000)
    }
}

/// Format bytes for human-readable display
#[inline]
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
