//! Tickable wrapper implementations for UI components
//!
//! This module provides wrapper types that implement the Tickable trait
//! for various UI components to enable opportunistic event-driven ticking.

use crate::concurrent::tickable::Tickable;
use std::time::Duration;

/// Wrapper for making any type Tickable with custom timing
pub struct TickableWrapper<T> {
    /// The wrapped component
    pub inner: T,
    /// The tick interval for this wrapper
    interval: Duration,
}

impl<T> TickableWrapper<T> {
    /// Create a new tickable wrapper with the specified interval
    pub fn new(inner: T, interval: Duration) -> Self {
        Self { inner, interval }
    }

    /// Get a reference to the inner component
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner component
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Tickable for TickableWrapper<T>
where
    T: Tickable,
{
    fn tick(&mut self) {
        self.inner.tick();
    }

    fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    fn frame_interval(&self) -> Duration {
        self.interval
    }
}
