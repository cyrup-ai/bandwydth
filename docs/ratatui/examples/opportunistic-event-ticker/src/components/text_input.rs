//! Simple text input widget for demonstrating event-driven animations
//!
//! This widget provides basic text editing functionality to generate
//! keyboard events that drive the opportunistic ticker.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

/// A simple single-line text input widget
#[derive(Debug, Clone)]
pub struct TextInput {
    /// The text content
    content: String,
    /// Current cursor position (byte index)
    cursor_position: usize,
    /// Visual cursor position (character count)
    cursor_visual: usize,
    /// Widget style
    style: Style,
    /// Cursor style
    cursor_style: Style,
    /// Whether the input is focused
    focused: bool,
    /// Maximum content length
    max_length: usize,
}

impl TextInput {
    /// Create a new text input
    #[inline]
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            cursor_visual: 0,
            style: Style::default(),
            cursor_style: Style::default().bg(Color::White).fg(Color::Black),
            focused: false,
            max_length: 1024,
        }
    }

    /// Set the initial content
    #[inline]
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self.cursor_position = self.content.len();
        self.cursor_visual = self.content.chars().count();
        self
    }

    /// Set the style
    #[inline]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the cursor style
    #[inline]
    pub fn with_cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    /// Set focus state
    #[inline]
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if focused
    #[inline]
    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    /// Get the content
    #[inline]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Clear the content
    #[inline]
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
        self.cursor_visual = 0;
    }

    /// Handle keyboard input
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if !self.focused {
            return false;
        }

        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.content.len() < self.max_length {
                    // Insert character at cursor position
                    self.content.insert(self.cursor_position, c);
                    self.cursor_position += c.len_utf8();
                    self.cursor_visual += 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    // Find the previous character boundary
                    let mut new_pos = self.cursor_position.saturating_sub(1);
                    while !self.content.is_char_boundary(new_pos) && new_pos > 0 {
                        new_pos -= 1;
                    }
                    self.content.remove(new_pos);
                    self.cursor_position = new_pos;
                    self.cursor_visual = self.cursor_visual.saturating_sub(1);
                    true
                } else {
                    false
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.content.len() {
                    self.content.remove(self.cursor_position);
                    true
                } else {
                    false
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    // Move to previous character boundary
                    let mut new_pos = self.cursor_position.saturating_sub(1);
                    while !self.content.is_char_boundary(new_pos) && new_pos > 0 {
                        new_pos -= 1;
                    }
                    self.cursor_position = new_pos;
                    self.cursor_visual = self.cursor_visual.saturating_sub(1);
                    true
                } else {
                    false
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.content.len() {
                    // Move to next character boundary
                    let mut new_pos = self.cursor_position + 1;
                    while !self.content.is_char_boundary(new_pos) && new_pos < self.content.len() {
                        new_pos += 1;
                    }
                    self.cursor_position = new_pos.min(self.content.len());
                    self.cursor_visual = (self.cursor_visual + 1).min(self.content.chars().count());
                    true
                } else {
                    false
                }
            }
            KeyCode::Home => {
                if self.cursor_position != 0 {
                    self.cursor_position = 0;
                    self.cursor_visual = 0;
                    true
                } else {
                    false
                }
            }
            KeyCode::End => {
                let end = self.content.len();
                if self.cursor_position != end {
                    self.cursor_position = end;
                    self.cursor_visual = self.content.chars().count();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Render the text input to a single line
    fn render_line(&self, area: Rect, buf: &mut Buffer) {
        // Clear the area
        for x in area.left()..area.right() {
            buf.get_mut(x, area.y).set_symbol(" ").set_style(self.style);
        }

        // Calculate visible content based on area width
        let available_width = area.width as usize;
        let content_chars: Vec<char> = self.content.chars().collect();

        // Simple scrolling logic - keep cursor visible
        let start_char = if self.cursor_visual >= available_width {
            self.cursor_visual.saturating_sub(available_width - 1)
        } else {
            0
        };

        let visible_chars: String = content_chars
            .iter()
            .skip(start_char)
            .take(available_width)
            .collect();

        // Render the visible text
        buf.set_string(area.x, area.y, &visible_chars, self.style);

        // Render cursor if focused
        if self.focused {
            let cursor_screen_pos = self.cursor_visual.saturating_sub(start_char);
            if cursor_screen_pos < available_width {
                let cursor_x = area.x + cursor_screen_pos as u16;

                // Get the character at cursor position or use space
                let cursor_char = if self.cursor_visual < content_chars.len() {
                    content_chars[self.cursor_visual]
                } else {
                    ' '
                };

                buf.get_mut(cursor_x, area.y)
                    .set_symbol(&cursor_char.to_string())
                    .set_style(self.cursor_style);
            }
        }
    }
}

impl Widget for TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a block with borders
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .title(if self.focused {
                " Type Here "
            } else {
                " Text Input "
            });

        let inner = block.inner(area);
        block.render(area, buf);

        // Render the text content in the inner area
        if inner.height > 0 && inner.width > 0 {
            self.render_line(inner, buf);
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_basic() {
        let mut input = TextInput::new();
        input.set_focused(true);

        // Type some text
        assert!(input.handle_key_event(KeyEvent::from(KeyCode::Char('H'))));
        assert!(input.handle_key_event(KeyEvent::from(KeyCode::Char('i'))));
        assert_eq!(input.content(), "Hi");

        // Backspace
        assert!(input.handle_key_event(KeyEvent::from(KeyCode::Backspace)));
        assert_eq!(input.content(), "H");

        // Navigate
        assert!(input.handle_key_event(KeyEvent::from(KeyCode::Home)));
        assert_eq!(input.cursor_position, 0);

        assert!(input.handle_key_event(KeyEvent::from(KeyCode::End)));
        assert_eq!(input.cursor_position, 1);
    }

    #[test]
    fn test_text_input_unicode() {
        let mut input = TextInput::new();
        input.set_focused(true);

        // Type emoji
        assert!(input.handle_key_event(KeyEvent::from(KeyCode::Char('🎯'))));
        assert_eq!(input.content(), "🎯");

        // Cursor should handle multi-byte chars correctly
        assert!(input.handle_key_event(KeyEvent::from(KeyCode::Left)));
        assert_eq!(input.cursor_position, 0);
    }
}
