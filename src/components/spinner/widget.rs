use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

/// A stateless widget that renders a single spinner frame
///
/// This widget is completely stateless and allocation-free.
/// It simply renders the provided frame text with the given style.
#[derive(Debug, Clone, Copy)]
pub struct SpinnerWidget {
    /// The frame text to render
    frame: &'static str,
    /// Style to apply to the frame
    style: Style,
}

impl SpinnerWidget {
    /// Create a new spinner widget
    #[inline(always)]
    pub const fn new(frame: &'static str, style: Style) -> Self {
        Self { frame, style }
    }

    /// Create a widget with default style
    #[inline(always)]
    pub const fn with_frame(frame: &'static str) -> Self {
        Self {
            frame,
            style: Style::new(),
        }
    }

    /// Set the style
    #[inline(always)]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Get the frame text
    #[inline(always)]
    pub const fn frame(&self) -> &'static str {
        self.frame
    }

    /// Calculate the width needed to render this spinner
    #[inline]
    pub fn width(&self) -> u16 {
        // Use unicode width for accurate rendering
        unicode_width::UnicodeWidthStr::width(self.frame) as u16
    }
}

impl Widget for SpinnerWidget {
    #[inline]
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Early return if area is too small
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Calculate the actual width of the spinner text
        let spinner_width = self.width();

        // Don't render if the spinner is wider than the area
        if spinner_width > area.width {
            return;
        }

        // Center the spinner horizontally in the area
        let x_offset = (area.width.saturating_sub(spinner_width)) / 2;
        let x = area.x.saturating_add(x_offset);

        // Render in the vertical center
        let y = area.y.saturating_add(area.height / 2);

        // Set the string with the configured style
        buf.set_string(x, y, self.frame, self.style);
    }
}

impl From<&'static str> for SpinnerWidget {
    #[inline]
    fn from(frame: &'static str) -> Self {
        Self::with_frame(frame)
    }
}

impl From<(&'static str, Style)> for SpinnerWidget {
    #[inline]
    fn from((frame, style): (&'static str, Style)) -> Self {
        Self::new(frame, style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_spinner_widget_creation() {
        let widget = SpinnerWidget::new("⠋", Style::default().fg(Color::Cyan));
        assert_eq!(widget.frame(), "⠋");
        assert_eq!(widget.width(), 1);
    }

    #[test]
    fn test_spinner_widget_from_str() {
        let widget: SpinnerWidget = "⠙".into();
        assert_eq!(widget.frame(), "⠙");
    }

    #[test]
    fn test_spinner_widget_width() {
        // Single character
        assert_eq!(SpinnerWidget::with_frame("⠋").width(), 1);

        // Multi-character
        assert_eq!(SpinnerWidget::with_frame("=>").width(), 2);
        assert_eq!(SpinnerWidget::with_frame("...").width(), 3);

        // Unicode
        assert_eq!(SpinnerWidget::with_frame("🌍").width(), 2); // Earth emoji is wide
    }

    #[test]
    fn test_spinner_widget_render() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 3));
        let widget = SpinnerWidget::new("X", Style::default().fg(Color::Red));

        widget.render(Rect::new(0, 0, 5, 3), &mut buf);

        // Check that the spinner is centered
        let cell = &buf[(2, 1)]; // Center position
        assert_eq!(cell.symbol(), "X");
        assert_eq!(cell.fg, Color::Red);
    }

    #[test]
    fn test_spinner_widget_render_small_area() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        let widget = SpinnerWidget::with_frame("ABC"); // Too wide

        // Should not panic, just not render
        widget.render(Rect::new(0, 0, 1, 1), &mut buf);

        // Buffer should be unchanged
        assert_eq!(buf[(0, 0)].symbol(), " ");
    }
}
