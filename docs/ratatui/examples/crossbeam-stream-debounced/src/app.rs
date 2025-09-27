use color_eyre::Result;
use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
};

use crate::{
    action::Action,
    components::{Component, PullRequests},
    concurrent::{spawn_executor, ExecutorHandle},
    Event, Tui,
};

pub struct App {
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    action_tx: Sender<Action>,
    action_rx: Receiver<Action>,
    executor: ExecutorHandle,
}

impl App {
    pub async fn new() -> Result<Self> {
        // Use bounded channel for backpressure control
        let (action_tx, action_rx) = bounded(256);

        // Spawn the executor with the action channel
        let executor = spawn_executor(action_tx.clone());

        // Create components
        let components: Vec<Box<dyn Component>> =
            vec![Box::new(PullRequests::new(executor.clone()))];

        Ok(Self {
            components,
            should_quit: false,
            action_tx,
            action_rx,
            executor,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new(self.event_tx())?;
        tui.enter()?;

        // Initialize all components
        for component in self.components.iter_mut() {
            component.init()?;
        }

        // Initial render
        self.render(&mut tui)?;

        // Main event loop - process events until quit
        while !self.should_quit {
            self.process_events(&mut tui)?;
        }

        tui.exit()?;
        Ok(())
    }

    #[inline]
    fn process_events(&mut self, tui: &mut Tui) -> Result<()> {
        let mut needs_render = false;

        // Process all pending actions without blocking
        loop {
            match self.action_rx.try_recv() {
                Ok(action) => {
                    needs_render |= self.handle_action(action)?;
                }
                Err(TryRecvError::Empty) => {
                    // No more actions, break
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    // Channel disconnected, quit
                    self.should_quit = true;
                    break;
                }
            }
        }

        // Render if any action requested it
        if needs_render {
            self.render(tui)?;
        }

        // Small sleep to prevent CPU spinning
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS

        Ok(())
    }

    #[inline]
    fn handle_action(&mut self, action: Action) -> Result<bool> {
        let mut needs_render = false;

        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Render => {
                needs_render = true;
            }
            Action::Resize(_width, _height) => {
                // Terminal resize is handled by TUI automatically
                // Just trigger a re-render
                needs_render = true;
            }
            Action::Key(key) => {
                if self.handle_key_event(key)? {
                    needs_render = true;
                }
            }
            Action::Mouse(_) => {
                // Mouse events can be handled by components
                for component in self.components.iter_mut() {
                    if let Some(event) = Event::from_action(&action) {
                        if let Some(new_action) = component.handle_events(Some(event))? {
                            self.action_tx.send(new_action)?;
                        }
                    }
                }
            }
            Action::FetchResult(ref _result) => {
                // Process fetch results through all components
                for component in self.components.iter_mut() {
                    if let Some(new_action) = component.update(action.clone())? {
                        self.action_tx.send(new_action)?;
                    }
                }
                needs_render = true;
            }
            Action::Error(ref msg) => {
                // Log error and trigger render to show it
                eprintln!("Error: {}", msg);
                needs_render = true;
            }
        }

        Ok(needs_render)
    }

    #[inline]
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        // Only handle key press events
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        // Check for global quit keys
        if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('d')))
        {
            self.action_tx.send(Action::Quit)?;
            return Ok(false);
        }

        // Let components handle the key
        let mut needs_render = false;
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_key_event(key)? {
                self.action_tx.send(action)?;
                needs_render = true;
            }
        }

        Ok(needs_render)
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            // Create layout
            let vertical = Layout::vertical([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(1), // Status bar
            ]);
            let [title_area, content_area, status_area] = vertical.areas(frame.area());

            // Render title
            let title = Line::from("Ratatui Essence")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .centered();
            let title_block = ratatui::widgets::Block::bordered()
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(title, title_block.inner(title_area));
            frame.render_widget(title_block, title_area);

            // Render components
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, content_area) {
                    // Draw error message if component fails
                    let error_msg = format!("Component render error: {}", err);
                    let error_widget = ratatui::widgets::Paragraph::new(error_msg)
                        .style(Style::default().fg(Color::Red))
                        .wrap(ratatui::widgets::Wrap { trim: true });
                    frame.render_widget(error_widget, content_area);
                }
            }

            // Render status bar
            let status = Line::from("Press 'q' to quit | 'r' to refresh")
                .style(Style::default().fg(Color::DarkGray))
                .centered();
            frame.render_widget(status, status_area);
        })?;

        Ok(())
    }

    #[inline]
    fn event_tx(&self) -> Sender<Action> {
        self.action_tx.clone()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Gracefully shutdown the executor
        self.executor.clone().shutdown();
    }
}
