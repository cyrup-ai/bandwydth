use std::time::{Duration, Instant};

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};
use tachyonfx::{Duration as TachyonDuration, Effect, Shader};
use unicode_width::UnicodeWidthStr;

use crate::display::{DisplayBandwidth, StatusEffects, UIState};

/// Calculate the elapsed time taking into account pause state.
pub fn elapsed_time(last_start_time: Instant, cumulative_time: Duration, paused: bool) -> Duration {
    if paused {
        cumulative_time
    } else {
        cumulative_time + last_start_time.elapsed()
    }
}

fn format_duration(d: Duration) -> String {
    let s = d.as_secs();
    let days = match s / 86400 {
        0 => "".to_string(),
        1 => "1 day, ".to_string(),
        n => format!("{n} days, "),
    };
    format!(
        "{days}{:02}:{:02}:{:02}",
        (s / 3600) % 24,
        (s / 60) % 60,
        s % 60,
    )
}

/// Header details display component showing bandwidth and timing information.
pub struct HeaderDetails<'a> {
    /// Reference to the current UI state.
    pub state: &'a UIState,
    /// Total elapsed monitoring time.
    pub elapsed_time: Duration,
    /// Whether monitoring is currently paused.
    pub paused: bool,
    /// Effect for the status indicator gradient.
    pub status_effect: Option<Effect>,
}

impl HeaderDetails<'_> {
    /// Create a new HeaderDetails with status effect
    pub fn new(state: &UIState, elapsed_time: Duration, paused: bool) -> HeaderDetails<'_> {
        let status_effect = Some(StatusEffects::create_class_effect(&state.bandwidth_class));
        HeaderDetails {
            state,
            elapsed_time,
            paused,
            status_effect,
        }
    }

    /// Render the header details to the given frame area.
    pub fn render(&mut self, frame: &mut Frame, rect: Rect, frame_duration: Duration) {
        let bandwidth = self.bandwidth_string();
        let color = if self.paused {
            Color::Yellow
        } else {
            Color::Green
        };

        // do not render time in tests, otherwise the output becomes non-deterministic
        // see: https://github.com/imsnif/bandwhich/issues/303
        if cfg!(not(test)) && self.state.cumulative_mode {
            let elapsed_time = format_duration(self.elapsed_time);
            // only render if there is enough width
            if bandwidth.width() + 1 + elapsed_time.width() <= rect.width as usize {
                self.render_elapsed_time(frame, rect, &elapsed_time, color);
            }
        }

        self.render_bandwidth(frame, rect, &bandwidth, color);

        // Apply status effect to the rendered content
        if let Some(ref mut effect) = self.status_effect {
            if !effect.done() {
                let tachyon_duration =
                    TachyonDuration::from_millis(frame_duration.as_millis() as u32);
                effect.process(tachyon_duration, frame.buffer_mut(), rect);
            }
        }
    }

    fn render_bandwidth(&self, frame: &mut Frame, rect: Rect, bandwidth: &str, color: Color) {
        let bandwidth_text = Span::styled(
            bandwidth,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        );

        let paragraph = Paragraph::new(bandwidth_text).alignment(Alignment::Left);
        frame.render_widget(paragraph, rect);
    }

    fn bandwidth_string(&self) -> String {
        let intrf = self.state.interface_name.as_deref().unwrap_or("all");
        let t = if self.state.cumulative_mode {
            "Data"
        } else {
            "Rate"
        };
        let unit_family = self.state.unit_family;
        let up = DisplayBandwidth {
            bandwidth: self.state.total_bytes_uploaded as f64,
            unit_family,
        };
        let down = DisplayBandwidth {
            bandwidth: self.state.total_bytes_downloaded as f64,
            unit_family,
        };
        let class = StatusEffects::get_class_text(&self.state.bandwidth_class);
        let speed = format!("{:.1} MB/s", self.state.current_speed);
        let paused = if self.paused { " [PAUSED]" } else { "" };
        format!("IF: {intrf} | {class} ({speed}) | {t} (Up/Down): {up}/{down}{paused}")
    }

    fn render_elapsed_time(&self, frame: &mut Frame, rect: Rect, elapsed_time: &str, color: Color) {
        let elapsed_time_text = Span::styled(
            elapsed_time,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        );
        let paragraph = Paragraph::new(elapsed_time_text).alignment(Alignment::Right);
        frame.render_widget(paragraph, rect);
    }
}
