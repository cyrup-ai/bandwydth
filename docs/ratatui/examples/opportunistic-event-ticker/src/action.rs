use crossterm::event::{KeyEvent, MouseEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum::Display;

/// Unique identifier for a spinner instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpinnerId(pub u8);

impl SpinnerId {
    #[inline(always)]
    pub const fn new(id: u8) -> Self {
        Self(id)
    }
}

/// Speed multiplier for spinner animation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpinnerSpeed(pub f32);

impl SpinnerSpeed {
    #[inline(always)]
    pub const fn normal() -> Self {
        Self(1.0)
    }

    #[inline(always)]
    pub const fn double() -> Self {
        Self(2.0)
    }

    #[inline(always)]
    pub const fn half() -> Self {
        Self(0.5)
    }
}

/// Actions that can be performed in the application
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    /// Quit the application
    Quit,

    /// Request a render update
    Render,

    /// Terminal resize event
    Resize(u16, u16),

    /// Keyboard input event
    Key(KeyEvent),

    /// Mouse input event
    Mouse(MouseEvent),

    /// Error occurred
    Error(String),

    /// Advance a specific spinner by one frame
    SpinnerTick(SpinnerId),

    /// Advance multiple spinners in a batch (more efficient)
    SpinnerTickBatch(Vec<SpinnerId>),

    /// Set spinner animation speed
    SpinnerSpeed(SpinnerId, SpinnerSpeed),

    /// Pause a spinner
    SpinnerPause(SpinnerId),

    /// Resume a spinner
    SpinnerResume(SpinnerId),

    /// Reset a spinner to frame 0
    SpinnerReset(SpinnerId),

    /// Reverse spinner direction
    SpinnerReverse(SpinnerId),

    /// Set spinner to a specific frame
    SpinnerSetFrame(SpinnerId, usize),

    /// Create a new spinner with a preset
    SpinnerCreate(SpinnerId, crate::SpinnerPreset),

    /// Remove a spinner
    SpinnerRemove(SpinnerId),

    /// Group spinners to tick together
    SpinnerGroup(Vec<SpinnerId>),

    /// Ungroup spinners
    SpinnerUngroup(Vec<SpinnerId>),

    /// Start spinner driver with tick interval
    SpinnerDriverStart(Duration),

    /// Stop spinner driver
    SpinnerDriverStop,

    /// Adjust driver tick rate
    SpinnerDriverSetRate(Duration),
}

impl Action {
    /// Check if this action requires a render update
    #[inline(always)]
    pub const fn requires_render(&self) -> bool {
        matches!(
            self,
            Action::Render
                | Action::Resize(_, _)
                | Action::SpinnerTick(_)
                | Action::SpinnerTickBatch(_)
                | Action::SpinnerSetFrame(_, _)
                | Action::SpinnerCreate(_, _)
                | Action::SpinnerRemove(_)
                | Action::SpinnerReset(_)
                | Action::Error(_)
        )
    }

    /// Check if this is a spinner-related action
    #[inline(always)]
    pub const fn is_spinner_action(&self) -> bool {
        matches!(
            self,
            Action::SpinnerTick(_)
                | Action::SpinnerTickBatch(_)
                | Action::SpinnerSpeed(_, _)
                | Action::SpinnerPause(_)
                | Action::SpinnerResume(_)
                | Action::SpinnerReset(_)
                | Action::SpinnerReverse(_)
                | Action::SpinnerSetFrame(_, _)
                | Action::SpinnerCreate(_, _)
                | Action::SpinnerRemove(_)
                | Action::SpinnerGroup(_)
                | Action::SpinnerUngroup(_)
                | Action::SpinnerDriverStart(_)
                | Action::SpinnerDriverStop
                | Action::SpinnerDriverSetRate(_)
        )
    }

    /// Get the spinner ID if this action targets a specific spinner
    #[inline(always)]
    pub const fn spinner_id(&self) -> Option<SpinnerId> {
        match self {
            Action::SpinnerTick(id)
            | Action::SpinnerSpeed(id, _)
            | Action::SpinnerPause(id)
            | Action::SpinnerResume(id)
            | Action::SpinnerReset(id)
            | Action::SpinnerReverse(id)
            | Action::SpinnerSetFrame(id, _)
            | Action::SpinnerCreate(id, _)
            | Action::SpinnerRemove(id) => Some(*id),
            _ => None,
        }
    }
}

/// Result type for action processing
pub type ActionResult<T = ()> = std::result::Result<T, ActionError>;

/// Errors that can occur during action processing
#[derive(Debug, Clone, PartialEq)]
pub enum ActionError {
    /// Channel send error
    ChannelSend(String),
    /// Channel receive error
    ChannelRecv(String),
    /// Spinner not found
    SpinnerNotFound(SpinnerId),
    /// Invalid frame index
    InvalidFrame(SpinnerId, usize),
    /// Invalid speed value
    InvalidSpeed(f32),
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionError::ChannelSend(msg) => write!(f, "Channel send error: {}", msg),
            ActionError::ChannelRecv(msg) => write!(f, "Channel receive error: {}", msg),
            ActionError::SpinnerNotFound(id) => write!(f, "Spinner {:?} not found", id),
            ActionError::InvalidFrame(id, frame) => {
                write!(f, "Invalid frame {} for spinner {:?}", frame, id)
            }
            ActionError::InvalidSpeed(speed) => write!(f, "Invalid speed: {}", speed),
        }
    }
}

impl std::error::Error for ActionError {}

impl From<crossbeam_channel::SendError<Action>> for ActionError {
    #[inline]
    fn from(err: crossbeam_channel::SendError<Action>) -> Self {
        ActionError::ChannelSend(err.to_string())
    }
}

impl From<crossbeam_channel::RecvError> for ActionError {
    #[inline]
    fn from(err: crossbeam_channel::RecvError) -> Self {
        ActionError::ChannelRecv(err.to_string())
    }
}
