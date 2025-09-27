use color_eyre::Result;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Cell, HighlightSpacing, Row, Table, TableState},
};
use std::time::Instant;

use super::Component;
use crate::action::Action;
use crate::concurrent::{ExecutorHandle, Task};
use crate::data::{FetchResult, LoadingState, PullRequest};

pub struct PullRequests {
    state: PullRequestState,
    executor: ExecutorHandle,
    last_tick: Instant,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_INTERVAL_MS: u128 = 80;

#[derive(Debug, Default)]
struct PullRequestState {
    pull_requests: Vec<PullRequest>,
    loading_state: LoadingState,
    table_state: TableState,
    spinner_index: usize,
}

impl PullRequests {
    #[inline]
    pub fn new(executor: ExecutorHandle) -> Self {
        Self {
            state: PullRequestState::default(),
            executor,
            last_tick: Instant::now(),
        }
    }

    /// Trigger a fetch of pull requests
    #[inline]
    fn fetch_pull_requests(&mut self) -> Result<()> {
        self.state.loading_state = LoadingState::Loading;

        // Submit task to executor - static strings for zero allocation
        let task = Task::FetchPullRequests {
            owner: "ratatui",
            repo: "ratatui",
        };

        self.executor
            .submit(task)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to submit task: {}", e))?;

        Ok(())
    }
}

impl Component for PullRequests {
    #[inline]
    fn init(&mut self) -> Result<()> {
        // Fetch pull requests on initialization
        self.fetch_pull_requests()?;
        // Trigger initial render for spinner
        Ok(())
    }

    #[inline]
    fn handle_events(&mut self, event: Option<crate::tui::Event>) -> Result<Option<Action>> {
        // Check if we need to trigger a render for spinner animation
        if matches!(self.state.loading_state, LoadingState::Loading) {
            let now = Instant::now();
            if now.duration_since(self.last_tick).as_millis() >= SPINNER_INTERVAL_MS {
                return Ok(Some(Action::Render));
            }
        }

        match event {
            Some(crate::tui::Event::Key(key)) => self.handle_key_event(key),
            _ => Ok(None),
        }
    }

    #[inline]
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.state.pull_requests.is_empty() {
                    self.state.table_state.scroll_down_by(1);
                    Ok(Some(Action::Render))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !self.state.pull_requests.is_empty() {
                    self.state.table_state.scroll_up_by(1);
                    Ok(Some(Action::Render))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('r') => {
                // Refresh pull requests
                self.fetch_pull_requests()?;
                Ok(Some(Action::Render))
            }
            KeyCode::Char('g') => {
                // Go to top
                if !self.state.pull_requests.is_empty() {
                    self.state.table_state.select_first();
                    Ok(Some(Action::Render))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('G') => {
                // Go to bottom
                if !self.state.pull_requests.is_empty() {
                    self.state.table_state.select_last();
                    Ok(Some(Action::Render))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    #[inline]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FetchResult(FetchResult::PullRequestsOk(prs)) => {
                self.state.pull_requests = prs;
                self.state.loading_state = LoadingState::Loaded;

                // Select first item if we have any
                if !self.state.pull_requests.is_empty()
                    && self.state.table_state.selected().is_none()
                {
                    self.state.table_state.select(Some(0));
                }

                Ok(Some(Action::Render))
            }
            Action::FetchResult(FetchResult::PullRequestsErr(err)) => {
                self.state.loading_state = LoadingState::Error(err);
                Ok(Some(Action::Render))
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Update spinner animation
        if matches!(self.state.loading_state, LoadingState::Loading) {
            let now = Instant::now();
            if now.duration_since(self.last_tick).as_millis() >= SPINNER_INTERVAL_MS {
                self.state.spinner_index = (self.state.spinner_index + 1) % SPINNER_FRAMES.len();
                self.last_tick = now;
            }
        }

        let title = match &self.state.loading_state {
            LoadingState::Idle => " Pull Requests ".to_string(),
            LoadingState::Loading => {
                format!(
                    " Pull Requests {} Loading... ",
                    SPINNER_FRAMES[self.state.spinner_index]
                )
            }
            LoadingState::Loaded => format!(" Pull Requests [{}] ", self.state.pull_requests.len()),
            LoadingState::Error(e) => format!(" Pull Requests [Error: {}] ", e),
        };

        let block = Block::bordered()
            .title(title)
            .title_bottom(" j/k: scroll | r: refresh | g/G: top/bottom | q: quit ");

        if self.state.pull_requests.is_empty() {
            let message = match &self.state.loading_state {
                LoadingState::Loading => {
                    let spinner = SPINNER_FRAMES[self.state.spinner_index];
                    Box::leak(format!("{} Loading pull requests...", spinner).into_boxed_str())
                }
                LoadingState::Error(e) => e.as_str(),
                _ => "No pull requests found",
            };

            let paragraph = ratatui::widgets::Paragraph::new(message)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(paragraph, area);
        } else {
            let rows = self.state.pull_requests.iter().map(|pr| {
                Row::new(vec![
                    Cell::from(pr.id.as_str()),
                    Cell::from(pr.title.as_str()),
                    Cell::from(pr.url.as_str()),
                ])
            });

            let widths = [
                Constraint::Length(6), // ID
                Constraint::Fill(1),   // Title (takes remaining space)
                Constraint::Max(50),   // URL (max 50 chars)
            ];

            let table = Table::new(rows, widths)
                .block(block)
                .header(
                    Row::new(vec!["ID", "Title", "URL"])
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .bottom_margin(1),
                )
                .highlight_spacing(HighlightSpacing::Always)
                .highlight_symbol(">> ")
                .row_highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray),
                );

            frame.render_stateful_widget(table, area, &mut self.state.table_state);
        }

        Ok(())
    }
}
