//! Zero-allocation debounced event stream using crossbeam channels
//!
//! This module provides a high-performance debounced stream that:
//! - Emits isolated events immediately (speed lane)
//! - Debounces rapid events with trailing edge emission
//! - Zero allocations after initialization
//! - Lock-free operation using crossbeam's SegQueue

use crossbeam::queue::SegQueue;
use crossbeam_channel::Receiver;
use futures_timer::Delay;
use futures_util::{FutureExt, Stream};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pin_project! {
    /// A debounced event stream that emits isolated events immediately and rapid events
    /// after a specified delay of inactivity, yielding `T`.
    ///
    /// This stream processes events from a `crossbeam_channel::Receiver<T>`, implementing a
    /// hybrid leading/trailing debounce: isolated events (arriving after a pause longer
    /// than the delay duration) are emitted immediately ("speed lane"), while rapid
    /// successive events are debounced to emit only the most recent one after the delay.
    /// It achieves zero allocations beyond initial setup, requiring `T` to be `Unpin`.
    pub struct Debounced<T> {
        receiver: Receiver<T>,
        queue: SegQueue<T>,
        delay_duration: Duration,
        #[pin]
        delay: Delay,
        last_item: Option<T>,
        last_event_time: Option<Instant>,
    }
}

impl<T> Debounced<T> {
    /// Creates a new `Debounced` stream from a receiver and delay duration.
    ///
    /// # Arguments
    /// - `receiver`: The `crossbeam_channel::Receiver<T>` providing events.
    /// - `delay`: The duration to wait before emitting rapid events.
    ///
    /// # Performance
    /// - Zero allocations during operation (beyond initial `SegQueue` and `Delay` setup).
    /// - Lock-free `SegQueue` for high-performance event buffering.
    /// - Safe, checked operations with no `unsafe` code.
    #[inline]
    pub fn new(receiver: Receiver<T>, delay: Duration) -> Self {
        Debounced {
            receiver,
            queue: SegQueue::new(),
            delay_duration: delay,
            delay: Delay::new(Duration::ZERO),
            last_item: None,
            last_event_time: None,
        }
    }

    /// Get the number of events currently queued
    #[inline]
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }
}

impl<T> Stream for Debounced<T>
where
    T: Unpin + Send,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // Check if the delay has expired for a pending item
            if this.last_item.is_some() && this.delay.poll_unpin(cx).is_ready() {
                *this.last_event_time = None;
                let item = this
                    .last_item
                    .take()
                    .expect("last_item verified with is_some");
                return Poll::Ready(Some(item));
            }

            // Poll the receiver for new events
            match this.receiver.try_recv() {
                Ok(event) => {
                    this.queue.push(event);
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // Channel disconnected
                    if this.last_item.is_none() && this.queue.is_empty() {
                        return Poll::Ready(None); // Stream exhausted
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    // No new events available
                    if this.last_item.is_none() && this.queue.is_empty() {
                        // No events pending, wait for new data or timer expiration
                        return Poll::Pending;
                    }
                }
            }

            // Process the queue - keep only the latest event
            let now = Instant::now();
            let mut latest_event = None;

            // Drain all events, keeping only the last one
            while let Some(event) = this.queue.pop() {
                latest_event = Some(event);
            }

            if let Some(event) = latest_event {
                if let Some(last_time) = *this.last_event_time {
                    if now.duration_since(last_time) > *this.delay_duration {
                        // Speed lane: isolated event after delay period
                        *this.last_event_time = Some(now);
                        return Poll::Ready(Some(event));
                    }
                } else {
                    // First event: emit immediately
                    *this.last_event_time = Some(now);
                    return Poll::Ready(Some(event));
                }

                // Rapid event: store and reset delay
                *this.last_item = Some(event);
                *this.last_event_time = Some(now);
                this.delay.as_mut().reset(*this.delay_duration);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let pending = self.queue.len() + if self.last_item.is_some() { 1 } else { 0 };
        (pending, None)
    }
}

/// Creates a debounced stream from a crossbeam channel receiver, yielding `T`.
///
/// # Arguments
/// - `receiver`: The `crossbeam_channel::Receiver<T>` providing events.
/// - `delay`: The duration to wait before emitting rapid events.
///
/// # Example
/// ```no_run
/// use crossbeam_channel::unbounded;
/// use futures_util::pin_mut;
/// use futures_util::StreamExt;
/// use std::time::Duration;
///
/// let (tx, rx) = unbounded();
/// let debounced = debounced(rx, Duration::from_millis(50));
/// pin_mut!(debounced);
/// ```
#[inline]
pub fn debounced<T>(receiver: Receiver<T>, delay: Duration) -> Debounced<T>
where
    T: Unpin + Send,
{
    Debounced::new(receiver, delay)
}

impl<T> std::fmt::Debug for Debounced<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Debounced")
            .field("queue_len", &self.queue_len())
            .field("delay_duration", &self.delay_duration)
            .field("has_pending", &self.last_item.is_some())
            .finish()
    }
}
