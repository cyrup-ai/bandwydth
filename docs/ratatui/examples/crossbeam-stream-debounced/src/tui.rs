use color_eyre::Result;
use crossbeam_channel::{bounded, Sender as CrossbeamSender};
use crossterm::{
    cursor,
    event::{
        Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{pin_mut, FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use std::{
    io::{stdout, Stdout},
    ops::{Deref, DerefMut},
};
use tokio::task::JoinHandle;

use crate::{
    action::Action,
    concurrent::{cancellation_token, debounced},
};

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

impl Event {
    /// Convert an Action to an Event if applicable
    #[inline]
    pub fn from_action(action: &crate::action::Action) -> Option<Event> {
        match action {
            crate::action::Action::Key(key) => Some(Event::Key(*key)),
            _ => None,
        }
    }
}

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<Stdout>>,
    pub task: JoinHandle<()>,
    pub event_tx: CrossbeamSender<Action>,
    event_channel_tx: crossbeam_channel::Sender<CrosstermEvent>,
    cancel_tx: Option<crossbeam_channel::Sender<()>>,
}

impl Tui {
    /// Create a new TUI instance with zero-allocation event handling
    pub fn new(event_tx: CrossbeamSender<Action>) -> Result<Self> {
        // Use bounded channel with reasonable capacity for backpressure
        let (event_channel_tx, event_channel_rx) = bounded(256);

        let mut tui = Self {
            terminal: ratatui::Terminal::new(Backend::new(stdout()))?,
            task: tokio::spawn(async {}),
            event_tx,
            event_channel_tx,
            cancel_tx: None,
        };

        tui.start(event_channel_rx);
        Ok(tui)
    }

    /// Start the event loop with crossbeam-based cancellation
    fn start(&mut self, event_channel_rx: crossbeam_channel::Receiver<CrosstermEvent>) {
        self.cancel();

        let (cancel_tx, cancel_fut) = cancellation_token();
        self.cancel_tx = Some(cancel_tx);

        let event_tx = self.event_tx.clone();
        let event_channel_tx = self.event_channel_tx.clone();

        self.task = tokio::spawn(async move {
            let mut event_stream = EventStream::new();
            let debounced_stream = debounced(event_channel_rx, std::time::Duration::from_millis(2));
            pin_mut!(debounced_stream);
            pin_mut!(cancel_fut);

            loop {
                tokio::select! {
                    _ = &mut cancel_fut => break,
                    event = event_stream.next().fuse() => {
                        if let Some(Ok(event)) = event {
                            // Non-blocking send, drop event if channel full
                            let _ = event_channel_tx.try_send(event);
                        }
                    }
                    event = debounced_stream.next() => {
                        match event {
                            Some(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                                if is_quit_key(&key) {
                                    let _ = event_tx.send(Action::Quit);
                                } else {
                                    let _ = event_tx.send(Action::Key(key));
                                }
                            }
                            Some(CrosstermEvent::Mouse(mouse)) => {
                                let _ = event_tx.send(Action::Mouse(mouse));
                            }
                            Some(CrosstermEvent::Resize(x, y)) => {
                                let _ = event_tx.send(Action::Resize(x, y));
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }

    /// Enable raw mode and enter alternate screen
    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Suspend the TUI and restore terminal state
    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    /// Resume the TUI after suspension
    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Exit raw mode and restore terminal
    pub fn exit(&mut self) -> Result<()> {
        if crossterm::terminal::is_raw_mode_enabled()? {
            crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    /// Cancel the event loop by dropping the cancel sender
    fn cancel(&mut self) {
        self.cancel_tx = None; // Dropping the sender triggers cancellation
    }

    /// Get the next event from the queue (non-blocking)
    pub async fn next_event(&mut self) -> Option<Event> {
        // This is for compatibility - events are sent directly to event_tx
        None
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<Stdout>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.cancel();
        let _ = self.exit();
    }
}

/// Check if a key event represents a quit command
#[inline]
fn is_quit_key(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('d')))
}
