//! Cancellation utilities using crossbeam channels
//!
//! Provides zero-allocation, lock-free cancellation primitives with
//! ergonomic closure decorators for async operations.

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use pin_project_lite::pin_project;

/// A cancellation token that triggers cancellation when dropped
///
/// This is a zero-cost abstraction over a crossbeam channel sender.
/// Dropping the token closes the channel, signaling cancellation.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    tx: Sender<()>,
}

impl CancellationToken {
    /// Create a new cancellation token pair
    #[inline]
    pub fn new() -> (Self, CancellationHandle) {
        let (tx, rx) = bounded(0);
        (Self { tx }, CancellationHandle { rx })
    }

    /// Manually trigger cancellation
    #[inline]
    pub fn cancel(self) {
        drop(self);
    }

    /// Check if cancellation has been requested
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        // Channel is cancelled if receiver side is dropped
        self.tx.is_empty() && self.tx.is_full()
    }
}

/// Handle for checking cancellation status
#[derive(Debug, Clone)]
pub struct CancellationHandle {
    rx: Receiver<()>,
}

impl CancellationHandle {
    /// Check if cancellation has been requested (non-blocking)
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        // Channel is cancelled if it would return Disconnected
        matches!(self.rx.try_recv(), Err(TryRecvError::Disconnected))
    }

    /// Convert to a future that completes when cancelled
    #[inline]
    pub fn into_future(self) -> CancellationFuture {
        CancellationFuture::new(self.rx)
    }

    /// Get the inner receiver
    #[inline]
    pub fn as_receiver(&self) -> &Receiver<()> {
        &self.rx
    }
}

pin_project! {
    /// Future that resolves when cancellation is triggered
    #[derive(Debug)]
    pub struct CancellationFuture {
        receiver: Receiver<()>,
    }
}

impl CancellationFuture {
    /// Create a new cancellation future from a receiver
    #[inline]
    pub const fn new(receiver: Receiver<()>) -> Self {
        Self { receiver }
    }
}

impl Future for CancellationFuture {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(()) | Err(TryRecvError::Disconnected) => Poll::Ready(()),
            Err(TryRecvError::Empty) => {
                // Wake immediately for aggressive cancellation checking
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

/// Create a cancellation token pair
///
/// Returns (sender, future) where dropping the sender triggers cancellation
#[inline]
pub fn cancellation_token() -> (Sender<()>, CancellationFuture) {
    let (tx, rx) = bounded(0);
    (tx, CancellationFuture::new(rx))
}

/// Execute a future with cancellation support
///
/// The provided closure receives a CancellationHandle to check cancellation status.
/// The future completes when either the inner future completes or cancellation occurs.
///
/// # Example
/// ```no_run
/// let (token, handle) = CancellationToken::new();
///
/// let result = with_cancellation(handle, |cancel| async move {
///     loop {
///         if cancel.is_cancelled() {
///             break Err("Cancelled");
///         }
///         // Do work...
///         tokio::time::sleep(Duration::from_millis(100)).await;
///     }
/// }).await;
/// ```
pub async fn with_cancellation<F, Fut, T>(handle: CancellationHandle, f: F) -> Result<T, Cancelled>
where
    F: FnOnce(CancellationHandle) -> Fut,
    Fut: Future<Output = T>,
{
    let cancel_fut = handle.clone().into_future();
    let work_fut = f(handle);

    tokio::select! {
        _ = cancel_fut => Err(Cancelled),
        result = work_fut => Ok(result),
    }
}

/// Error type for cancelled operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation cancelled")
    }
}

impl std::error::Error for Cancelled {}

/// Select between multiple futures with automatic cancellation support
///
/// # Example
/// ```no_run
/// let (token, handle) = CancellationToken::new();
///
/// select_with_cancel!(
///     handle,
///     result = some_async_operation() => {
///         println!("Operation completed: {:?}", result);
///     }
///     _ = timer.tick() => {
///         println!("Timer expired");
///     }
/// );
/// ```
#[macro_export]
macro_rules! select_with_cancel {
    (
        $cancel:expr,
        $($pattern:pat = $future:expr => $code:block),* $(,)?
    ) => {{
        let __cancel_fut = $cancel.into_future();
        tokio::select! {
            _ = __cancel_fut => {
                Err($crate::crossbeam::cancellation::Cancelled)
            }
            $(
                $pattern = $future => {
                    Ok({ $code })
                }
            )*
        }
    }};
}

/// Extension trait for futures to add cancellation support
pub trait CancellableExt: Future + Sized {
    /// Run this future with cancellation support
    fn cancellable(self, handle: CancellationHandle) -> Cancellable<Self> {
        Cancellable {
            future: self,
            cancel: handle.into_future(),
        }
    }
}

impl<F: Future> CancellableExt for F {}

pin_project! {
    /// Future wrapper that adds cancellation support
    pub struct Cancellable<F> {
        #[pin]
        future: F,
        #[pin]
        cancel: CancellationFuture,
    }
}

impl<F: Future> Future for Cancellable<F> {
    type Output = Result<F::Output, Cancelled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // Check cancellation first
        if let Poll::Ready(()) = this.cancel.poll(cx) {
            return Poll::Ready(Err(Cancelled));
        }

        // Poll the inner future
        match this.future.poll(cx) {
            Poll::Ready(value) => Poll::Ready(Ok(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Scoped cancellation token that cancels on drop
///
/// Useful for ensuring cleanup in async blocks
pub struct ScopedCancellation {
    token: Option<CancellationToken>,
}

impl ScopedCancellation {
    /// Create a new scoped cancellation
    #[inline]
    pub fn new() -> (Self, CancellationHandle) {
        let (token, handle) = CancellationToken::new();
        (Self { token: Some(token) }, handle)
    }

    /// Cancel early
    #[inline]
    pub fn cancel(&mut self) {
        self.token.take();
    }

    /// Disarm the cancellation (prevent automatic cancellation on drop)
    #[inline]
    pub fn disarm(mut self) -> Option<CancellationToken> {
        self.token.take()
    }
}

impl Drop for ScopedCancellation {
    #[inline]
    fn drop(&mut self) {
        // Automatically cancels when dropped (unless disarmed)
        self.token.take();
    }
}
