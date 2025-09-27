use crate::action::Action;
use crossbeam_channel::Sender;
use crossterm::{
    cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::Backend, Frame, Terminal};
use std::io;

/// Terminal UI wrapper that manages terminal state and rendering
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    action_tx: Sender<Action>,
}

impl<B: Backend> Tui<B> {
    /// Create a new TUI instance
    #[inline]
    pub fn new(terminal: Terminal<B>, action_tx: Sender<Action>) -> io::Result<Self> {
        Ok(Self {
            terminal,
            action_tx,
        })
    }

    /// Enter terminal raw mode and alternate screen
    pub fn enter(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;

        // Clear the terminal
        self.terminal
            .clear()
            .map_err(|e| io::Error::other(e.to_string()))?;

        // Send initial resize event
        let size = self
            .terminal
            .size()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let _ = self
            .action_tx
            .try_send(Action::Resize(size.width, size.height));

        Ok(())
    }

    /// Exit terminal raw mode and alternate screen
    pub fn exit(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)?;
        Ok(())
    }

    /// Draw the UI with the provided rendering function
    #[inline]
    pub fn draw<F>(&mut self, render_fn: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal
            .draw(render_fn)
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }

    /// Get the current terminal size
    #[inline]
    pub fn size(&self) -> io::Result<ratatui::layout::Rect> {
        self.terminal
            .size()
            .map(|size| ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: size.width,
                height: size.height,
            })
            .map_err(|e| io::Error::other(e.to_string()))
    }

    /// Force a terminal resize event
    #[inline]
    pub fn resize(&mut self) -> io::Result<()> {
        self.terminal
            .autoresize()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let size = self
            .terminal
            .size()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let _ = self
            .action_tx
            .try_send(Action::Resize(size.width, size.height));
        Ok(())
    }

    /// Get a reference to the underlying terminal
    #[inline(always)]
    pub const fn terminal(&self) -> &Terminal<B> {
        &self.terminal
    }

    /// Get a mutable reference to the underlying terminal
    #[inline(always)]
    pub fn terminal_mut(&mut self) -> &mut Terminal<B> {
        &mut self.terminal
    }

    /// Hide the cursor
    #[inline]
    pub fn hide_cursor(&mut self) -> io::Result<()> {
        self.terminal
            .hide_cursor()
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }

    /// Show the cursor at a specific position
    #[inline]
    pub fn show_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.terminal
            .set_cursor_position((x, y))
            .map_err(|e| io::Error::other(e.to_string()))?;
        self.terminal
            .show_cursor()
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }

    /// Clear the terminal screen
    #[inline]
    pub fn clear(&mut self) -> io::Result<()> {
        self.terminal
            .clear()
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }
}

/// Ensure proper cleanup on drop
impl<B: Backend> Drop for Tui<B> {
    fn drop(&mut self) {
        // Best effort cleanup - ignore errors
        let _ = self.exit();
    }
}

/// Helper to create a centered rect
#[inline(always)]
pub const fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_width = area.width.saturating_mul(percent_x) / 100;
    let popup_height = area.height.saturating_mul(percent_y) / 100;

    let popup_x = area
        .x
        .saturating_add(area.width.saturating_sub(popup_width) / 2);
    let popup_y = area
        .y
        .saturating_add(area.height.saturating_sub(popup_height) / 2);

    ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    }
}

/// Helper to create a rect with margins
#[inline(always)]
pub const fn margin_rect(margin: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let margin_2 = margin.saturating_mul(2);

    ratatui::layout::Rect {
        x: area.x.saturating_add(margin),
        y: area.y.saturating_add(margin),
        width: area.width.saturating_sub(margin_2),
        height: area.height.saturating_sub(margin_2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 25);
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 50);
    }

    #[test]
    fn test_margin_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let with_margin = margin_rect(10, area);

        assert_eq!(with_margin.x, 10);
        assert_eq!(with_margin.y, 10);
        assert_eq!(with_margin.width, 80);
        assert_eq!(with_margin.height, 80);
    }

    #[test]
    fn test_edge_cases() {
        let small_area = Rect::new(0, 0, 5, 5);

        // Test with large margins
        let with_margin = margin_rect(10, small_area);
        assert_eq!(with_margin.width, 0);
        assert_eq!(with_margin.height, 0);

        // Test with 100% centered rect
        let centered = centered_rect(100, 100, small_area);
        assert_eq!(centered.width, 5);
        assert_eq!(centered.height, 5);
    }
}
