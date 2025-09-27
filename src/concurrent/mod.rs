//! Concurrent utilities for zero-allocation async operations
//!
//! This module provides high-performance concurrent primitives built
//! exclusively on crossbeam for lock-free, zero-allocation operations:
//!
//! - Async I/O bridge for blocking file descriptors
//! - Debounced event streams with hybrid leading/trailing edge
//! - Cancellation primitives with ergonomic closure decorators
//! - All operations are lock-free and allocation-free after initialization

mod cancellation;
mod debounced;
mod executor;
mod opportunistic_ticker;
mod threaded;
pub mod tickable;
pub mod ticker_state;

// Debounced stream exports
pub use debounced::{debounced, Debounced};

// Cancellation exports
pub use cancellation::{
    cancellation_token, with_cancellation, Cancellable, CancellableExt, CancellationFuture,
    CancellationHandle, CancellationToken, Cancelled, ScopedCancellation,
};

// Executor exports
pub use executor::{spawn_executor, ExecutorHandle, Task, TaskSubmitError};

// Threaded I/O exports
pub use threaded::{split_async_io, ThreadedAsyncReader, ThreadedAsyncWriter, ThreadedRawFd};

// Tickable and opportunistic ticker exports
pub use opportunistic_ticker::OpportunisticDebouncedTicker;
pub use tickable::{BatchTickable, Tickable, TickerMetrics};
pub use ticker_state::{
    OpportunisticDebouncedTicker as OpportunisticDebouncedTickerImpl, OpportunisticTicker,
};

// Re-export commonly used crossbeam types for convenience
pub use crossbeam_channel::{
    after, bounded, never, tick, unbounded, Receiver, RecvError, RecvTimeoutError, Select,
    SelectedOperation, SendError, SendTimeoutError, Sender, TryRecvError, TrySendError,
};

pub use crossbeam_queue::{ArrayQueue, SegQueue};

// Re-export the select macro for convenience
pub use crossbeam_channel::select;

/// Prelude for convenient imports
///
/// # Example
/// ```
/// use ratagpu_example::concurrent::prelude::*;
/// ```
pub mod prelude {
    pub use super::{
        // Constructors
        bounded,
        cancellation_token,
        debounced,
        spawn_executor,
        split_async_io,
        unbounded,
        // Utility functions
        with_cancellation,
        // Extension traits
        CancellableExt,
        // Core types
        CancellationHandle,
        CancellationToken,
        Cancelled,
        // Executor
        ExecutorHandle,
        // Tickable components
        OpportunisticDebouncedTicker,
        Task,
        TaskSubmitError,
        ThreadedAsyncReader,
        ThreadedAsyncWriter,
        ThreadedRawFd,
        Tickable,
        TickerMetrics,
    };

    // Re-export the select macro
    pub use crossbeam_channel::select;
}
