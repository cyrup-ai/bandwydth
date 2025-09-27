// No-op terminal backend implementation for raw output mode.
// 
// The TUI backend normally switches stdout to raw byte mode, which interferes
// with simple text output. This backend implements the ratatui Backend trait
// but performs no actual terminal operations, allowing clean text output
// in raw mode.
//
// This follows the Null Object pattern - providing a valid interface
// implementation that performs no operations instead of requiring
// conditional logic throughout the application.

use std::io;

use ratatui::{
    backend::{Backend, WindowSize},
    buffer::Cell,
    layout::{Position, Size},
};

/// A no-op terminal backend for raw output mode.
///
/// This backend implements the ratatui Backend trait but performs no actual
/// terminal operations, allowing the application to run in raw output mode
/// without interfering with stdout.
pub struct RawTerminalBackend {}

impl Backend for RawTerminalBackend {
    type Error = io::Error;

    fn clear(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn clear_region(&mut self, _clear_type: ratatui::backend::ClearType) -> io::Result<()> {
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::new(0, 0))
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _position: P) -> io::Result<()> {
        Ok(())
    }

    fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(0, 0))
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: Size::default(),
            pixels: Size::default(),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
