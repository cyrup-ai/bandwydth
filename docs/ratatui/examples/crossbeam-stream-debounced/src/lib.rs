pub mod action;
pub mod app;
pub mod components;
pub mod concurrent;
pub mod data;
pub mod tui;

pub use action::Action;
pub use app::App;
pub use components::{Component, PullRequests};
pub use data::{FetchResult, LoadingState, PullRequest};
pub use tui::{Event, Tui};

// Re-export concurrent utilities
pub use concurrent::{
    // Cancellation
    cancellation_token,
    // Debounced streams
    debounced,
    with_cancellation,
    CancellationHandle,
    CancellationToken,
    Debounced,
    ThreadedAsyncReader,
    ThreadedAsyncWriter,
    ThreadedRawFd,
};
