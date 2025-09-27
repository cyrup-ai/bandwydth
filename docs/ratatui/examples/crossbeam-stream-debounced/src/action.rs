use serde::{Deserialize, Serialize};
use strum::Display;

use crate::data::FetchResult;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Quit,
    Render,
    Error(String),
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    FetchResult(FetchResult),
}
