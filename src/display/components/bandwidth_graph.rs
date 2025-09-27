use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color as RatatuiColor, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};
use std::time::{Duration, Instant};
use tachyonfx::{Effect, Shader};

use crate::concurrent::tickable::Tickable;

use super::{
    bandwidth_data::{BandwidthDataManager, BandwidthLevel, BandwidthPoint},
    bandwidth_effects::{BandwidthEffects, EffectStack},
};
use crate::network::BandwidthStats;

/// State for the bandwidth graph widget
#[derive(Debug)]
pub struct BandwidthGraphState {
    /// Data manager for bandwidth history
    data_manager: BandwidthDataManager,
    /// TachyonFX effects stack
    effect_stack: EffectStack,
    /// Current effect composition
    current_effect: Option<Effect>,
    /// Last effect update time
    last_effect_update: Instant,
    /// Whether effects need regeneration
    effects_dirty: bool,
}

impl BandwidthGraphState {
    /// Create a new bandwidth graph state
    pub fn new() -> Self {
        Self {
            data_manager: BandwidthDataManager::new(),
            effect_stack: EffectStack::new(),
            current_effect: None,
            last_effect_update: Instant::now(),
            effects_dirty: true,
        }
    }

    /// Update with new bandwidth data (event-driven)
    pub fn update_bandwidth(
        &mut self,
        stats: &BandwidthStats,
        utilization: Option<&crate::network::Utilization>,
    ) {
        let point = BandwidthPoint::from_stats(stats, utilization);
        let level_changed = self.data_manager.level_changed();

        self.data_manager.add_point(point);

        // Mark effects as dirty if significant change occurred
        if level_changed {
            self.effects_dirty = true;
        }
    }

    /// Update effects with frame timing
    pub fn update_effects(&mut self, frame_duration: Duration) {
        // Regenerate effects if needed
        if self.effects_dirty {
            self.regenerate_effects();
            self.effects_dirty = false;
        }

        // Update effect stack
        self.effect_stack.update(frame_duration);
        self.last_effect_update = Instant::now();
    }

    /// Get current bandwidth level
    pub fn current_level(&self) -> Option<BandwidthLevel> {
        self.data_manager.current_level()
    }

    /// Get sparkline string for display
    pub fn sparkline_string(&self) -> String {
        self.data_manager.sparkline_string()
    }

    /// Get session peak
    pub fn session_peak(&self) -> f64 {
        self.data_manager.session_peak()
    }

    /// Get latest bandwidth point
    pub fn latest_point(&self) -> Option<&BandwidthPoint> {
        self.data_manager.latest_point()
    }

    /// Check if effects are running
    pub fn has_running_effects(&self) -> bool {
        self.current_effect.as_ref().is_some_and(|e| e.running())
    }

    /// Get current effect for rendering
    pub fn current_effect(&self) -> Option<&Effect> {
        self.current_effect.as_ref()
    }

    /// Regenerate effects based on current state
    fn regenerate_effects(&mut self) {
        if let Some(level) = self.current_level() {
            let intensity = self
                .latest_point()
                .map(|p| p.intensity as f32)
                .unwrap_or(0.0);

            let current_position = self.data_manager.history().len().saturating_sub(1);
            let peak_positions = self.data_manager.peak_positions();

            self.current_effect = Some(BandwidthEffects::create_effect_stack(
                level,
                intensity,
                current_position,
                peak_positions,
            ));
        }
    }

    /// Render the bandwidth graph component
    pub fn render(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        // Update effects with frame timing
        self.update_effects(Duration::from_millis(16)); // ~60 FPS

        // Create the bandwidth graph widget
        let widget = BandwidthGraphWidget::new()
            .title("BANDWIDTH ANALYSIS")
            .border_style(Style::default().fg(RatatuiColor::Cyan));

        // Render with state
        frame.render_stateful_widget(widget, area, self);
    }
}

impl Default for BandwidthGraphState {
    fn default() -> Self {
        Self::new()
    }
}

/// Modern stateful bandwidth graph widget with TachyonFX effects
#[derive(Debug)]
pub struct BandwidthGraphWidget {
    /// Widget title
    title: String,
    /// Widget border style
    border_style: Style,
    /// Whether to show borders
    show_borders: bool,
}

impl BandwidthGraphWidget {
    /// Create a new bandwidth graph widget
    pub fn new() -> Self {
        Self {
            title: "BANDWIDTH GRAPH".to_string(),
            border_style: Style::default().fg(RatatuiColor::Cyan),
            show_borders: true,
        }
    }

    /// Set the widget title
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }

    /// Set border style
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Control border visibility
    pub fn borders(mut self, show: bool) -> Self {
        self.show_borders = show;
        self
    }

    /// Render the peak glow markers row
    fn render_peak_markers(&self, state: &BandwidthGraphState, buf: &mut Buffer, area: Rect) {
        let history = state.data_manager.history();
        let peak_positions = state.data_manager.peak_positions();

        if history.is_empty() || area.width == 0 {
            return;
        }

        let mut markers = String::new();
        for i in 0..area.width as usize {
            let history_index = if i < history.len() {
                i
            } else {
                history.len() - 1
            };

            if peak_positions.contains(&history_index) {
                markers.push('●');
            } else if history
                .get(history_index)
                .is_some_and(|p| p.intensity > 0.7)
            {
                markers.push('○');
            } else {
                markers.push(' ');
            }
        }

        let peak_line = Line::from(vec![
            Span::styled("│ ", Style::default().fg(RatatuiColor::DarkGray)),
            Span::styled(markers, Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(" ← peak markers"),
        ]);

        let paragraph = Paragraph::new(peak_line);
        let marker_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        paragraph.render(marker_area, buf);
    }

    /// Render the main sparkline row
    fn render_sparkline(&self, state: &BandwidthGraphState, buf: &mut Buffer, area: Rect) {
        let sparkline = state.sparkline_string();
        let padded_sparkline = if sparkline.len() < area.width as usize {
            format!("{:width$}", sparkline, width = area.width as usize)
        } else {
            sparkline
        };

        let sparkline_line = Line::from(vec![
            Span::styled("│ ", Style::default().fg(RatatuiColor::DarkGray)),
            Span::styled(padded_sparkline, Style::default().fg(RatatuiColor::White)),
        ]);

        let paragraph = Paragraph::new(sparkline_line);
        let sparkline_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 1,
        };
        paragraph.render(sparkline_area, buf);
    }

    /// Render the flow baseline row
    fn render_flow_baseline(&self, _state: &BandwidthGraphState, buf: &mut Buffer, area: Rect) {
        let baseline_chars: String = (0..area.width.saturating_sub(20))
            .map(|i| if i % 2 == 0 { '▔' } else { '▁' })
            .collect();

        let baseline_line = Line::from(vec![
            Span::styled("│ ", Style::default().fg(RatatuiColor::DarkGray)),
            Span::styled(baseline_chars, Style::default().fg(RatatuiColor::DarkGray)),
        ]);

        let paragraph = Paragraph::new(baseline_line);
        let baseline_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width,
            height: 1,
        };
        paragraph.render(baseline_area, buf);
    }

    /// Render the scale reference row
    fn render_scale_reference(&self, state: &BandwidthGraphState, buf: &mut Buffer, area: Rect) {
        let peak = state.session_peak();
        let scale_points = if peak > 0.0 {
            vec![
                "0MB".to_string(),
                format!("{:.0}MB", peak * 0.25),
                format!("{:.0}MB", peak * 0.5),
                format!("{:.0}MB", peak * 0.75),
                format!("{:.0}MB", peak),
            ]
        } else {
            vec![
                "0MB".to_string(),
                "25MB".to_string(),
                "50MB".to_string(),
                "75MB".to_string(),
                "100MB".to_string(),
            ]
        };

        let scale_text = scale_points.join("   ");
        let scale_line = Line::from(vec![
            Span::styled("│ ", Style::default().fg(RatatuiColor::DarkGray)),
            Span::styled(scale_text, Style::default().fg(RatatuiColor::Gray)),
            Span::raw(" ← scale"),
        ]);

        let paragraph = Paragraph::new(scale_line);
        let scale_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: 1,
        };
        paragraph.render(scale_area, buf);
    }

    /// Render the header with current stats
    fn render_header(&self, state: &BandwidthGraphState, buf: &mut Buffer, area: Rect) {
        let current_speed = state.latest_point().map(|p| p.speed_mbps).unwrap_or(0.0);

        let _download_speed = state.latest_point().map(|p| p.download_rate).unwrap_or(0.0);

        let download_bytes = state.latest_point().map(|p| p.download_bytes).unwrap_or(0);

        let peak_speed = state.session_peak();

        let header_text = format!(
            "─ DOWNLOAD BANDWIDTH ─ {:.1} MB/s ─ {:.1} GB Total ─ Peak: {:.1} MB/s ",
            current_speed,
            download_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            peak_speed
        );

        let level_color = state
            .current_level()
            .map_or(RatatuiColor::Gray, |level| match level {
                BandwidthLevel::Low => RatatuiColor::Blue,
                BandwidthLevel::Medium => RatatuiColor::Cyan,
                BandwidthLevel::High => RatatuiColor::Yellow,
                BandwidthLevel::Critical => RatatuiColor::Red,
            });

        let header_len = header_text.len();
        let header_line = Line::from(vec![
            Span::styled("┌", Style::default().fg(RatatuiColor::DarkGray)),
            Span::styled(
                header_text,
                Style::default()
                    .fg(level_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "─".repeat((area.width as usize).saturating_sub(header_len + 1)),
                Style::default().fg(RatatuiColor::DarkGray),
            ),
            Span::styled("┐", Style::default().fg(RatatuiColor::DarkGray)),
        ]);

        let paragraph = Paragraph::new(header_line);
        let header_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        paragraph.render(header_area, buf);
    }

    /// Render the bottom border
    fn render_footer(&self, buf: &mut Buffer, area: Rect) {
        let footer_line = Line::from(vec![
            Span::styled("└", Style::default().fg(RatatuiColor::DarkGray)),
            Span::styled(
                "─".repeat((area.width as usize).saturating_sub(2)),
                Style::default().fg(RatatuiColor::DarkGray),
            ),
            Span::styled("┘", Style::default().fg(RatatuiColor::DarkGray)),
        ]);

        let paragraph = Paragraph::new(footer_line);
        let footer_area = Rect {
            x: area.x,
            y: area.y + 4,
            width: area.width,
            height: 1,
        };
        paragraph.render(footer_area, buf);
    }
}

impl Default for BandwidthGraphWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of StatefulWidget for modern ratatui patterns
impl StatefulWidget for BandwidthGraphWidget {
    type State = BandwidthGraphState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Ensure minimum size for proper display
        if area.height < 6 || area.width < 40 {
            return;
        }

        // Render the multi-layer bandwidth graph
        self.render_header(state, buf, area);
        self.render_peak_markers(state, buf, area);
        self.render_sparkline(state, buf, area);
        self.render_flow_baseline(state, buf, area);
        self.render_scale_reference(state, buf, area);
        self.render_footer(buf, area);
    }
}

/// Implementation of StatefulWidget for references
impl StatefulWidget for &BandwidthGraphWidget {
    type State = BandwidthGraphState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Ensure minimum size for proper display
        if area.height < 6 || area.width < 40 {
            return;
        }

        // Render the multi-layer bandwidth graph
        self.render_header(state, buf, area);
        self.render_peak_markers(state, buf, area);
        self.render_sparkline(state, buf, area);
        self.render_flow_baseline(state, buf, area);
        self.render_scale_reference(state, buf, area);
        self.render_footer(buf, area);
    }
}

/// Extension trait for rendering with TachyonFX effects
pub trait BandwidthGraphEffectRenderer {
    /// Render the widget with TachyonFX effects
    fn render_with_effects(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut BandwidthGraphState,
        frame_duration: Duration,
    );
}

impl BandwidthGraphEffectRenderer for BandwidthGraphWidget {
    fn render_with_effects(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut BandwidthGraphState,
        frame_duration: Duration,
    ) {
        // First render the widget normally
        StatefulWidget::render(self, area, buf, state);

        // Update and apply TachyonFX effects
        state.update_effects(frame_duration);

        if let Some(effect) = state.current_effect() {
            if effect.running() {
                // Apply effects to the sparkline area (row 2)
                let _sparkline_area = Rect {
                    x: area.x + 2,                        // Skip "│ " prefix
                    y: area.y + 2,                        // Sparkline row
                    width: area.width.saturating_sub(20), // Leave space for suffix
                    height: 1,
                };

                // Note: This requires implementing EffectRenderer on Buffer
                // For now, we'll comment this out until proper integration
                // effect.process(frame_duration, buf, sparkline_area);
            }
        }
    }
}

impl BandwidthGraphState {
    /// Tick the bandwidth graph for smooth animations
    pub fn tick(&mut self) {
        // Update effects timing for smooth animations
        let now = Instant::now();
        let frame_duration = now.duration_since(self.last_effect_update);
        self.last_effect_update = now;

        // Update effects
        self.update_effects(frame_duration);
    }

    /// Check if the graph is actively animating
    pub fn is_active(&self) -> bool {
        // Always active if we have data or effects running
        !self.data_manager.history().is_empty()
            || self.current_effect.as_ref().is_some_and(|e| e.running())
    }
}

impl Tickable for BandwidthGraphState {
    fn tick(&mut self) {
        // Update effects timing for smooth animations
        let now = Instant::now();
        let frame_duration = now.duration_since(self.last_effect_update);
        self.last_effect_update = now;

        // Update effects for smooth graph animations
        self.update_effects(frame_duration);
    }

    fn is_active(&self) -> bool {
        // Always active if we have data or effects running
        !self.data_manager.history().is_empty()
            || self.current_effect.as_ref().is_some_and(|e| e.running())
    }

    fn frame_interval(&self) -> Duration {
        // Update animations every 16ms for 60fps smooth animations
        Duration::from_millis(16)
    }
}
