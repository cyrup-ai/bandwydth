//! Adapter to make SpinnerController work with OpportunisticDebouncedTicker
//!
//! This module provides the glue between the pure spinner logic and
//! the opportunistic timing system.

use crate::{
    action::SpinnerId, components::spinner::SpinnerController, concurrent::tickable::Tickable,
};
use std::time::Duration;

/// Tick mode determines which spinners advance on each tick
#[derive(Debug, Clone, PartialEq)]
pub enum TickMode {
    /// Tick all active spinners
    All,
    /// Tick spinners in a specific group
    Group(String),
    /// Tick a specific spinner
    Individual(SpinnerId),
    /// Tick multiple specific spinners
    Multiple(Vec<SpinnerId>),
}

/// Adapter that makes SpinnerController work with OpportunisticDebouncedTicker
pub struct TickableSpinners {
    /// The underlying spinner controller
    controller: SpinnerController,
    /// Which spinners to tick
    mode: TickMode,
    /// Frame interval for animation
    frame_interval: Duration,
    /// Whether ticking is enabled
    enabled: bool,
}

impl TickableSpinners {
    /// Create a new tickable spinner group that ticks all spinners
    #[inline]
    pub fn new(controller: SpinnerController) -> Self {
        Self {
            controller,
            mode: TickMode::All,
            frame_interval: Duration::from_millis(80), // ~12.5 FPS
            enabled: true,
        }
    }

    /// Create with a specific tick mode
    #[inline]
    pub fn with_mode(mut self, mode: TickMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the frame interval
    #[inline]
    pub fn with_frame_interval(mut self, interval: Duration) -> Self {
        self.frame_interval = interval;
        self
    }

    /// Get the controller
    #[inline]
    pub fn controller(&self) -> &SpinnerController {
        &self.controller
    }

    /// Get mutable access to the controller
    #[inline]
    pub fn controller_mut(&mut self) -> &mut SpinnerController {
        &mut self.controller
    }

    /// Set the tick mode
    #[inline]
    pub fn set_mode(&mut self, mode: TickMode) {
        self.mode = mode;
    }

    /// Enable or disable ticking
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get the current tick mode
    #[inline]
    pub fn mode(&self) -> &TickMode {
        &self.mode
    }

    /// Check which spinners would be ticked
    pub fn affected_spinners(&self) -> Vec<SpinnerId> {
        match &self.mode {
            TickMode::All => self.controller.all_spinner_ids(),
            TickMode::Group(name) => self.controller.get_group_spinners(name),
            TickMode::Individual(id) => vec![*id],
            TickMode::Multiple(ids) => ids.clone(),
        }
    }

    /// Count of active spinners that would be ticked
    pub fn active_count(&self) -> usize {
        self.affected_spinners()
            .into_iter()
            .filter(|&id| !self.controller.is_paused(id))
            .count()
    }
}

impl Tickable for TickableSpinners {
    fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        match &self.mode {
            TickMode::All => {
                // Get all non-paused spinners
                let ids: Vec<_> = self
                    .controller
                    .all_spinner_ids()
                    .into_iter()
                    .filter(|&id| !self.controller.is_paused(id))
                    .collect();

                if !ids.is_empty() {
                    self.controller.tick_batch(&ids);
                }
            }
            TickMode::Group(name) => {
                // Tick the named group (controller handles paused state)
                self.controller.tick_group(name);
            }
            TickMode::Individual(id) => {
                // Tick single spinner if not paused
                if !self.controller.is_paused(*id) {
                    self.controller.tick(*id);
                }
            }
            TickMode::Multiple(ids) => {
                // Tick multiple specific spinners
                let active_ids: Vec<_> = ids
                    .iter()
                    .copied()
                    .filter(|&id| !self.controller.is_paused(id))
                    .collect();

                if !active_ids.is_empty() {
                    self.controller.tick_batch(&active_ids);
                }
            }
        }
    }

    fn is_active(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Check if any affected spinners are active (not paused)
        match &self.mode {
            TickMode::All => {
                // Any spinner exists and is not paused
                self.controller.spinner_count() > 0 && self.active_count() > 0
            }
            TickMode::Group(name) => {
                // Group has active spinners
                let spinners = self.controller.get_group_spinners(name);
                !spinners.is_empty()
                    && spinners
                        .into_iter()
                        .any(|id| !self.controller.is_paused(id))
            }
            TickMode::Individual(id) => {
                // Specific spinner exists and is not paused
                !self.controller.is_paused(*id)
            }
            TickMode::Multiple(ids) => {
                // At least one spinner is active
                ids.iter().any(|&id| !self.controller.is_paused(id))
            }
        }
    }

    fn frame_interval(&self) -> Duration {
        self.frame_interval
    }
}

/// Builder for creating configured TickableSpinners
pub struct TickableSpinnersBuilder {
    mode: TickMode,
    frame_interval: Duration,
    enabled: bool,
}

impl Default for TickableSpinnersBuilder {
    fn default() -> Self {
        Self {
            mode: TickMode::All,
            frame_interval: Duration::from_millis(80),
            enabled: true,
        }
    }
}

impl TickableSpinnersBuilder {
    /// Create a new builder
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tick mode to all spinners
    #[inline]
    pub fn all(mut self) -> Self {
        self.mode = TickMode::All;
        self
    }

    /// Set tick mode to a specific group
    #[inline]
    pub fn group(mut self, name: impl Into<String>) -> Self {
        self.mode = TickMode::Group(name.into());
        self
    }

    /// Set tick mode to a single spinner
    #[inline]
    pub fn individual(mut self, id: SpinnerId) -> Self {
        self.mode = TickMode::Individual(id);
        self
    }

    /// Set tick mode to multiple specific spinners
    #[inline]
    pub fn multiple(mut self, ids: Vec<SpinnerId>) -> Self {
        self.mode = TickMode::Multiple(ids);
        self
    }

    /// Set frame interval
    #[inline]
    pub fn frame_interval(mut self, interval: Duration) -> Self {
        self.frame_interval = interval;
        self
    }

    /// Set frame rate in FPS
    #[inline]
    pub fn fps(self, fps: u32) -> Self {
        let interval = Duration::from_millis(1000 / fps as u64);
        self.frame_interval(interval)
    }

    /// Enable or disable
    #[inline]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Build the TickableSpinners
    #[inline]
    pub fn build(self, controller: SpinnerController) -> TickableSpinners {
        TickableSpinners {
            controller,
            mode: self.mode,
            frame_interval: self.frame_interval,
            enabled: self.enabled,
        }
    }
}
