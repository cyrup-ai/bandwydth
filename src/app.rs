use crate::{
    action::{Action, ActionResult},
    display::ui::Ui,
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::Backend, Frame};

/// Main application state container with event-driven architecture
///
/// This struct orchestrates the event processing loop, UI rendering,
/// and state management for the bandwidth monitoring application.
pub struct App<B: Backend + 'static> {
    action_tx: Sender<Action>,
    action_rx: Receiver<Action>,
    ui: Ui<B>,
    should_quit: bool,
    needs_render: bool,
}

impl<B: Backend + 'static> App<B> {
    /// Create a new application instance with event channels and UI
    pub fn new(
        action_tx: Sender<Action>,
        action_rx: Receiver<Action>,
        ui: Ui<B>,
    ) -> ActionResult<Self> {
        Ok(Self {
            action_tx,
            action_rx,
            ui,
            should_quit: false,
            needs_render: true,
        })
    }

    /// Initialize the application state and perform any setup tasks
    pub fn init(&mut self) -> ActionResult<()> {
        Ok(())
    }

    /// Process all pending events from the action channel
    ///
    /// Returns true if the application should quit, false otherwise.
    /// Processes up to MAX_BATCH events per call to prevent blocking.
    // EXACT copy from opportunistic-event-ticker/src/app.rs lines 149-163
    pub fn process_events(&mut self) -> ActionResult<bool> {
        let mut processed = 0;
        const MAX_BATCH: usize = 100;

        while processed < MAX_BATCH {
            match self.action_rx.try_recv() {
                Ok(action) => {
                    // Opportunistic ticking on ANY event - this is the key to the pattern!
                    if self.ui.on_any_event() {
                        self.needs_render = true;
                    }

                    self.handle_action(action)?;
                    processed += 1;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.should_quit = true;
                    break;
                }
            }
        }

        Ok(self.should_quit)
    }

    fn handle_action(&mut self, action: Action) -> ActionResult<()> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Tick => {
                // Tick events are now just one of many opportunities to tick
                // The opportunistic ticker already tried to tick above
                self.needs_render = true;
            }
            Action::Render => self.needs_render = true,
            Action::Resize(_, _) => self.needs_render = true,
            Action::UpdateUI => self.needs_render = true,
            Action::Key(key) => {
                self.handle_key(key)?;
            }
            Action::Error(msg) => {
                log::error!("Error: {}", msg);
                self.needs_render = true;
            }
            Action::DnsUpdate(dns_table) => {
                self.ui.update_dns_cache(dns_table);
                self.needs_render = true;
            }
            _ => {} // Ignore mouse and spinner events
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> ActionResult<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.action_tx.send(Action::Quit)?;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.action_tx.send(Action::Quit)?;
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.ui.previous_tab();
                } else {
                    self.ui.next_tab();
                }
                self.needs_render = true;
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if the UI needs to be rendered
    #[inline(always)]
    pub const fn should_render(&self) -> bool {
        self.needs_render
    }

    /// Mark that the UI has been rendered and clear the render flag
    #[inline]
    pub fn rendered(&mut self) {
        self.needs_render = false;
    }

    /// Render the UI to the given frame
    pub fn render(&mut self, frame: &mut Frame) {
        self.ui.render(frame);

        // After rendering, schedule timer ticks to ensure animations continue
        // even if no events occur - this is the second key part of the pattern!
        self.ui.schedule_timer();
    }
}
