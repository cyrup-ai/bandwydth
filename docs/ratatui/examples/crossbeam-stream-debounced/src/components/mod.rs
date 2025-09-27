mod pull_request;

pub use pull_request::PullRequests;

pub trait Component {
    fn init(&mut self) -> Result<(), color_eyre::Report>;
    fn handle_events(
        &mut self,
        event: Option<crate::tui::Event>,
    ) -> Result<Option<crate::action::Action>, color_eyre::Report>;
    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<Option<crate::action::Action>, color_eyre::Report>;
    fn update(
        &mut self,
        action: crate::action::Action,
    ) -> Result<Option<crate::action::Action>, color_eyre::Report>;
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> Result<(), color_eyre::Report>;
}
