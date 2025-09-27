use crate::{
    action::{Action, ActionResult, SpinnerId, SpinnerSpeed},
    components::{SpinnerController, TextInput, TickableSpinners},
    concurrent::OpportunisticDebouncedTicker,
    SpinnerPreset,
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::time::Duration;

/// Main application state
pub struct App {
    /// Channel for sending actions
    action_tx: Sender<Action>,
    /// Channel for receiving actions
    action_rx: Receiver<Action>,
    /// Opportunistic ticker that manages spinner animations
    spinner_ticker: OpportunisticDebouncedTicker<TickableSpinners>,
    /// Text input widget for demonstrating event-driven updates
    text_input: TextInput,
    /// Whether the app should quit
    should_quit: bool,
    /// Whether a render is needed
    needs_render: bool,
    /// Currently selected spinner for keyboard control
    selected_spinner: Option<SpinnerId>,
    /// Application mode
    mode: AppMode,
    /// Help text visibility
    show_help: bool,
    /// Performance metrics
    metrics: PerformanceMetrics,
}

/// Application modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    /// Normal viewing mode
    Normal,
    /// Spinner selection mode
    Selection,
    /// Speed adjustment mode
    Speed,
}

/// Performance metrics tracking
#[derive(Debug, Default)]
struct PerformanceMetrics {
    /// Total frames rendered
    frames_rendered: u64,
    /// Total events processed
    events_processed: u64,
    /// Batch tick count
    batch_ticks: u64,
    /// Single tick count
    single_ticks: u64,
}

impl App {
    /// Create a new application instance
    pub fn new(action_tx: Sender<Action>, action_rx: Receiver<Action>) -> ActionResult<Self> {
        let mut spinner_controller = SpinnerController::new();

        // Create demo spinners
        let demo_spinners = vec![
            (SpinnerId::new(0), SpinnerPreset::Dots),
            (SpinnerId::new(1), SpinnerPreset::Line),
            (SpinnerId::new(2), SpinnerPreset::Arc),
            (SpinnerId::new(3), SpinnerPreset::BouncingBar),
            (SpinnerId::new(4), SpinnerPreset::CircleQuarters),
            (SpinnerId::new(5), SpinnerPreset::Toggle),
        ];

        for (id, preset) in demo_spinners {
            spinner_controller.create(id, preset);
        }

        // Create tickable spinners adapter
        let tickable_spinners = TickableSpinners::new(spinner_controller);

        // Create the opportunistic ticker
        let spinner_ticker = OpportunisticDebouncedTicker::new(tickable_spinners);

        // Create text input with placeholder text
        let text_input = TextInput::new()
            .with_content("Type here to see opportunistic ticking in action!")
            .with_style(Style::default().fg(Color::White));

        Ok(Self {
            action_tx,
            action_rx,
            spinner_ticker,
            text_input,
            should_quit: false,
            needs_render: true,
            selected_spinner: Some(SpinnerId::new(0)),
            mode: AppMode::Normal,
            show_help: true,
            metrics: PerformanceMetrics::default(),
        })
    }

    /// Initialize the application
    pub fn init(&mut self) -> ActionResult<()> {
        // Group some spinners to tick together for demo
        self.spinner_ticker
            .target_mut()
            .controller_mut()
            .create_group(
                "fast_group",
                vec![SpinnerId::new(0), SpinnerId::new(2), SpinnerId::new(4)],
            );

        self.spinner_ticker
            .target_mut()
            .controller_mut()
            .create_group(
                "slow_group",
                vec![SpinnerId::new(1), SpinnerId::new(3), SpinnerId::new(5)],
            );

        // Focus the text input
        self.text_input.set_focused(true);

        Ok(())
    }

    /// Process all pending events
    #[inline]
    pub fn process_events(&mut self) -> ActionResult<bool> {
        let mut processed = 0;
        const MAX_BATCH: usize = 100;

        // Poll the ticker's fallback stream
        if self.spinner_ticker.try_poll_fallback() {
            self.needs_render = true;
        }

        // Process events in batches to prevent starvation
        while processed < MAX_BATCH {
            match self.action_rx.try_recv() {
                Ok(action) => {
                    self.handle_action(action)?;
                    processed += 1;
                    self.metrics.events_processed += 1;
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

    /// Handle a single action
    #[inline]
    fn handle_action(&mut self, action: Action) -> ActionResult<()> {
        // First, give the ticker a chance to advance on ANY event
        if self.spinner_ticker.on_any_event() {
            self.needs_render = true;
        }

        match action {
            Action::Quit => self.should_quit = true,
            Action::Render => self.needs_render = true,
            Action::Resize(_, _) => self.needs_render = true,
            Action::Key(key) => {
                // Let text input handle the key first if focused
                if self.text_input.is_focused() && self.text_input.handle_key_event(key) {
                    self.needs_render = true;
                } else {
                    self.handle_key(key)?;
                }
            }
            Action::Error(msg) => {
                eprintln!("Error: {}", msg);
                self.needs_render = true;
            }
            Action::SpinnerPause(id) => {
                self.spinner_ticker.target_mut().controller_mut().pause(id);
                self.needs_render = true;
            }
            Action::SpinnerResume(id) => {
                self.spinner_ticker.target_mut().controller_mut().resume(id);
                self.needs_render = true;
            }
            Action::SpinnerReset(id) => {
                self.spinner_ticker.target_mut().controller_mut().reset(id);
                self.needs_render = true;
            }
            Action::SpinnerReverse(id) => {
                self.spinner_ticker
                    .target_mut()
                    .controller_mut()
                    .reverse(id);
                self.needs_render = true;
            }
            _ => {} // Other actions not needed for demo
        }

        Ok(())
    }

    /// Handle keyboard input
    #[inline]
    fn handle_key(&mut self, key: KeyEvent) -> ActionResult<()> {
        // Global keys work in any mode
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.action_tx.send(Action::Quit)?;
                return Ok(());
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.action_tx.send(Action::Quit)?;
                return Ok(());
            }
            KeyCode::Char('h') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
                self.needs_render = true;
                return Ok(());
            }
            KeyCode::Tab => {
                // Toggle focus between text input and spinner controls
                self.text_input.set_focused(!self.text_input.is_focused());
                self.needs_render = true;
                return Ok(());
            }
            _ => {}
        }

        // Mode-specific key handling
        match self.mode {
            AppMode::Normal => self.handle_normal_keys(key)?,
            AppMode::Selection => self.handle_selection_keys(key)?,
            AppMode::Speed => self.handle_speed_keys(key)?,
        }

        Ok(())
    }

    /// Handle keys in normal mode
    #[inline]
    fn handle_normal_keys(&mut self, key: KeyEvent) -> ActionResult<()> {
        match key.code {
            KeyCode::Char('s') => {
                self.mode = AppMode::Selection;
                self.needs_render = true;
            }
            KeyCode::Char(' ') => {
                // Toggle selected spinner pause/resume
                if let Some(id) = self.selected_spinner {
                    let controller = self.spinner_ticker.target_mut().controller_mut();
                    if controller.is_paused(id) {
                        controller.resume(id);
                    } else {
                        controller.pause(id);
                    }
                    self.needs_render = true;
                }
            }
            KeyCode::Char('r') => {
                // Reset selected spinner
                if let Some(id) = self.selected_spinner {
                    self.action_tx.send(Action::SpinnerReset(id))?;
                }
            }
            KeyCode::Char('R') => {
                // Reverse selected spinner
                if let Some(id) = self.selected_spinner {
                    self.action_tx.send(Action::SpinnerReverse(id))?;
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.mode = AppMode::Speed;
                self.needs_render = true;
            }
            KeyCode::Char('m') => {
                // Reset metrics
                self.spinner_ticker.reset_metrics();
                self.metrics = PerformanceMetrics::default();
                self.needs_render = true;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle keys in selection mode
    #[inline]
    fn handle_selection_keys(&mut self, key: KeyEvent) -> ActionResult<()> {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let id = c.to_digit(10).unwrap() as u8;
                self.selected_spinner = Some(SpinnerId::new(id));
                self.mode = AppMode::Normal;
                self.needs_render = true;
            }
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.needs_render = true;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle keys in speed adjustment mode
    #[inline]
    fn handle_speed_keys(&mut self, key: KeyEvent) -> ActionResult<()> {
        if let Some(id) = self.selected_spinner {
            match key.code {
                KeyCode::Char('1') => {
                    self.action_tx
                        .send(Action::SpinnerSpeed(id, SpinnerSpeed::half()))?;
                    self.mode = AppMode::Normal;
                    self.needs_render = true;
                }
                KeyCode::Char('2') => {
                    self.action_tx
                        .send(Action::SpinnerSpeed(id, SpinnerSpeed::normal()))?;
                    self.mode = AppMode::Normal;
                    self.needs_render = true;
                }
                KeyCode::Char('3') => {
                    self.action_tx
                        .send(Action::SpinnerSpeed(id, SpinnerSpeed::double()))?;
                    self.mode = AppMode::Normal;
                    self.needs_render = true;
                }
                KeyCode::Esc => {
                    self.mode = AppMode::Normal;
                    self.needs_render = true;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Check if a render is needed
    #[inline(always)]
    pub const fn should_render(&self) -> bool {
        self.needs_render
    }

    /// Mark that rendering has been completed
    #[inline]
    pub fn rendered(&mut self) {
        self.needs_render = false;
        self.metrics.frames_rendered += 1;
    }

    /// Render the application
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Spinners
                Constraint::Length(3), // Text input
                Constraint::Length(3), // Status
                Constraint::Length(6), // Metrics
            ])
            .split(area);

        // Schedule fallback tick for next frame
        self.spinner_ticker.schedule_fallback();

        // Render title
        self.render_title(frame, chunks[0]);

        // Render spinners
        self.render_spinners(frame, chunks[1]);

        // Render text input
        frame.render_widget(self.text_input.clone(), chunks[2]);

        // Render status bar
        self.render_status(frame, chunks[3]);

        // Render metrics
        self.render_metrics(frame, chunks[4]);

        // Render help overlay if visible
        if self.show_help {
            self.render_help(frame, area);
        }
    }

    /// Render the title bar
    #[inline]
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled("Zero", Style::default().fg(Color::Cyan).bold()),
            Span::raw("shot "),
            Span::styled("Spinner", Style::default().fg(Color::Green).bold()),
            Span::raw(" - Event-Driven Animation Demo"),
        ])
        .centered();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        frame.render_widget(Paragraph::new(title).block(block), area);
    }

    /// Render all spinners
    fn render_spinners(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Spinners ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate spinner layout
        let controller = self.spinner_ticker.target().controller();
        let spinner_count = controller.spinner_count();
        if spinner_count == 0 {
            return;
        }

        let rows = ((spinner_count as f32).sqrt().ceil() as u16).max(1);
        let cols = ((spinner_count + rows as usize - 1) / rows as usize) as u16;

        let row_constraints: Vec<_> = (0..rows)
            .map(|_| Constraint::Ratio(1, rows as u32))
            .collect();
        let col_constraints: Vec<_> = (0..cols)
            .map(|_| Constraint::Ratio(1, cols as u32))
            .collect();

        let row_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(&row_constraints)
            .split(inner);

        // Render each spinner
        let mut spinner_idx = 0;
        for row_chunk in row_chunks.iter() {
            let col_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(&col_constraints)
                .split(*row_chunk);

            for col_chunk in col_chunks.iter() {
                if spinner_idx >= spinner_count {
                    break;
                }

                let id = SpinnerId::new(spinner_idx as u8);
                self.render_single_spinner(frame, *col_chunk, id);
                spinner_idx += 1;
            }
        }
    }

    /// Render a single spinner
    fn render_single_spinner(&mut self, frame: &mut Frame, area: Rect, id: SpinnerId) {
        let controller = self.spinner_ticker.target().controller();
        let is_selected = self.selected_spinner == Some(id);
        let is_paused = controller.is_paused(id);

        let border_color = match (is_selected, is_paused) {
            (true, false) => Color::Green,
            (true, true) => Color::Yellow,
            (false, true) => Color::DarkGray,
            (false, false) => Color::Gray,
        };

        let title = format!(" {} [{}] ", id.0, controller.preset_name(id));

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render the spinner widget
        if let Some(widget) = controller.widget(id) {
            let spinner_area = centered_spinner_area(inner);
            frame.render_widget(widget, spinner_area);
        }

        // Show additional info for selected spinner
        if is_selected && inner.height > 3 {
            let info = format!(
                "Frame: {} | Speed: {:.1}x{}",
                controller.current_frame(id),
                controller.speed(id).0,
                if is_paused { " [PAUSED]" } else { "" }
            );

            let info_widget = Paragraph::new(info)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

            let info_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(1),
                width: inner.width,
                height: 1,
            };

            frame.render_widget(info_widget, info_area);
        }
    }

    /// Render the status bar
    #[inline]
    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let mode_text = match self.mode {
            AppMode::Normal => "NORMAL",
            AppMode::Selection => "SELECT SPINNER (0-9)",
            AppMode::Speed => "SET SPEED (1=0.5x, 2=1x, 3=2x)",
        };

        let status = Line::from(vec![
            Span::raw("Mode: "),
            Span::styled(mode_text, Style::default().fg(Color::Cyan).bold()),
            Span::raw(" | "),
            Span::raw("Tab: Toggle Focus | Help: F1/h | Quit: q/Esc"),
        ])
        .centered();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        frame.render_widget(Paragraph::new(status).block(block), area);
    }

    /// Render performance metrics
    #[inline]
    fn render_metrics(&self, frame: &mut Frame, area: Rect) {
        let ticker_metrics = self.spinner_ticker.metrics();
        let controller = self.spinner_ticker.target().controller();

        // Create a visual bar for opportunistic ratio
        let ratio = ticker_metrics.opportunistic_ratio();
        let bar_width = 10;
        let filled = (ratio * bar_width as f32) as usize;
        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(bar_width - filled));

        let metrics_text = vec![
            Line::from(format!(
                "Frames: {} | Events: {} | Opportunistic: {:.1}% {}",
                self.metrics.frames_rendered,
                self.metrics.events_processed,
                ratio * 100.0,
                bar
            )),
            Line::from(format!(
                "Ticker - Opportunistic: {} | Fallback: {} | Cancelled: {} | Early: {}",
                ticker_metrics.opportunistic_ticks,
                ticker_metrics.fallback_ticks,
                ticker_metrics.cancelled_fallbacks,
                ticker_metrics.early_ticks
            )),
            Line::from(format!(
                "Active Spinners: {} | Groups: {} | Skipped (inactive): {}",
                controller.spinner_count(),
                controller.group_count(),
                ticker_metrics.skipped_inactive
            )),
        ];

        let block = Block::default()
            .title(" Metrics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        frame.render_widget(
            Paragraph::new(metrics_text)
                .block(block)
                .style(Style::default().fg(Color::DarkGray)),
            area,
        );
    }

    /// Render help overlay
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_area = centered_rect(60, 60, area);

        // Clear background
        frame.render_widget(Clear, help_area);

        let help_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Keyboard Controls",
                Style::default().fg(Color::Cyan).bold(),
            )),
            Line::from(""),
            Line::from("h/F1     - Toggle this help"),
            Line::from("q/Esc    - Quit application"),
            Line::from("Tab      - Toggle focus (text input / spinners)"),
            Line::from(""),
            Line::from(Span::styled(
                "Spinner Control",
                Style::default().fg(Color::Green).bold(),
            )),
            Line::from(""),
            Line::from("s        - Enter selection mode"),
            Line::from("Space    - Pause/Resume selected"),
            Line::from("r        - Reset selected"),
            Line::from("R        - Reverse selected"),
            Line::from("+/=      - Adjust speed"),
            Line::from(""),
            Line::from(Span::styled(
                "Group Control",
                Style::default().fg(Color::Yellow).bold(),
            )),
            Line::from(""),
            Line::from("m        - Reset metrics"),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to close",
                Style::default().fg(Color::DarkGray).italic(),
            )),
        ];

        let help_widget = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .style(Style::default().bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        frame.render_widget(help_widget, help_area);
    }
}

/// Helper to create a centered area for spinner display
#[inline(always)]
fn centered_spinner_area(area: Rect) -> Rect {
    let spinner_width = 3; // Most spinners are 1-3 chars wide
    let spinner_height = 1;

    let x = area.x + area.width.saturating_sub(spinner_width) / 2;
    let y = area.y + area.height.saturating_sub(spinner_height) / 2;

    Rect {
        x,
        y,
        width: spinner_width.min(area.width),
        height: spinner_height.min(area.height),
    }
}

/// Helper to create a centered rect with percentage dimensions
#[inline(always)]
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width.saturating_mul(percent_x) / 100;
    let height = area.height.saturating_mul(percent_y) / 100;

    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width,
        height,
    }
}
