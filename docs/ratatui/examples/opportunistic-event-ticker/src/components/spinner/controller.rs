use crate::{
    action::{SpinnerId, SpinnerSpeed},
    SpinnerPreset,
};
use ratatui::style::Style;
use std::collections::HashMap;

use super::{SpinnerState, SpinnerWidget};

/// Controller that manages multiple spinner instances
///
/// This controller operates purely on events - no time tracking or ticks.
/// Spinners advance their frames only when tick events are received.
#[derive(Debug)]
pub struct SpinnerController {
    /// All managed spinners
    spinners: HashMap<SpinnerId, SpinnerState>,
    /// Spinner groups for batch operations
    groups: HashMap<String, Vec<SpinnerId>>,
    /// Default style for new spinners
    default_style: Style,
}

impl SpinnerController {
    /// Create a new spinner controller
    #[inline]
    pub fn new() -> Self {
        Self {
            spinners: HashMap::with_capacity(16),
            groups: HashMap::with_capacity(4),
            default_style: Style::default(),
        }
    }

    /// Set the default style for new spinners
    #[inline]
    pub fn set_default_style(&mut self, style: Style) {
        self.default_style = style;
    }

    /// Create a new spinner with the given preset
    #[inline]
    pub fn create(&mut self, id: SpinnerId, preset: SpinnerPreset) {
        let state = SpinnerState::new(preset).with_style(self.default_style);
        self.spinners.insert(id, state);
    }

    /// Remove a spinner
    #[inline]
    pub fn remove(&mut self, id: SpinnerId) -> Option<SpinnerState> {
        // Also remove from any groups
        for group in self.groups.values_mut() {
            group.retain(|&spinner_id| spinner_id != id);
        }
        self.spinners.remove(&id)
    }

    /// Tick a spinner forward by one frame
    #[inline]
    pub fn tick(&mut self, id: SpinnerId) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.tick();
        }
    }

    /// Tick multiple spinners in a batch (more efficient)
    #[inline]
    pub fn tick_batch(&mut self, ids: &[SpinnerId]) {
        for &id in ids {
            if let Some(state) = self.spinners.get_mut(&id) {
                state.tick();
            }
        }
    }

    /// Tick all spinners in a group
    #[inline]
    pub fn tick_group(&mut self, group_name: &str) {
        if let Some(group) = self.groups.get(group_name) {
            let ids: Vec<_> = group.clone();
            self.tick_batch(&ids);
        }
    }

    /// Get a widget for rendering a spinner
    #[inline]
    pub fn widget(&self, id: SpinnerId) -> Option<SpinnerWidget> {
        self.spinners.get(&id).map(|state| state.widget())
    }

    /// Set spinner speed multiplier
    #[inline]
    pub fn set_speed(&mut self, id: SpinnerId, speed: SpinnerSpeed) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.set_speed(speed);
        }
    }

    /// Get spinner speed
    #[inline]
    pub fn speed(&self, id: SpinnerId) -> SpinnerSpeed {
        self.spinners
            .get(&id)
            .map(|state| state.speed())
            .unwrap_or(SpinnerSpeed::normal())
    }

    /// Pause a spinner
    #[inline]
    pub fn pause(&mut self, id: SpinnerId) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.pause();
        }
    }

    /// Resume a spinner
    #[inline]
    pub fn resume(&mut self, id: SpinnerId) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.resume();
        }
    }

    /// Check if a spinner is paused
    #[inline]
    pub fn is_paused(&self, id: SpinnerId) -> bool {
        self.spinners
            .get(&id)
            .map(|state| state.is_paused())
            .unwrap_or(false)
    }

    /// Reset a spinner to frame 0
    #[inline]
    pub fn reset(&mut self, id: SpinnerId) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.reset();
        }
    }

    /// Reverse spinner direction
    #[inline]
    pub fn reverse(&mut self, id: SpinnerId) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.reverse();
        }
    }

    /// Set spinner to a specific frame
    #[inline]
    pub fn set_frame(&mut self, id: SpinnerId, frame: usize) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.set_frame(frame);
        }
    }

    /// Get current frame index
    #[inline]
    pub fn current_frame(&self, id: SpinnerId) -> usize {
        self.spinners
            .get(&id)
            .map(|state| state.current_frame())
            .unwrap_or(0)
    }

    /// Get spinner preset name
    #[inline]
    pub fn preset_name(&self, id: SpinnerId) -> &'static str {
        self.spinners
            .get(&id)
            .map(|state| state.preset_name())
            .unwrap_or("Unknown")
    }

    /// Get the number of managed spinners
    #[inline]
    pub fn spinner_count(&self) -> usize {
        self.spinners.len()
    }

    /// Get all spinner IDs
    #[inline]
    pub fn all_spinner_ids(&self) -> Vec<SpinnerId> {
        self.spinners.keys().copied().collect()
    }

    /// Create a group of spinners
    #[inline]
    pub fn create_group(&mut self, name: impl Into<String>, spinners: Vec<SpinnerId>) {
        self.groups.insert(name.into(), spinners);
    }

    /// Remove a group
    #[inline]
    pub fn remove_group(&mut self, name: &str) -> Option<Vec<SpinnerId>> {
        self.groups.remove(name)
    }

    /// Add a spinner to a group
    #[inline]
    pub fn add_to_group(&mut self, group_name: &str, id: SpinnerId) {
        self.groups
            .entry(group_name.to_string())
            .or_insert_with(Vec::new)
            .push(id);
    }

    /// Remove a spinner from a group
    #[inline]
    pub fn remove_from_group(&mut self, group_name: &str, id: SpinnerId) {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.retain(|&spinner_id| spinner_id != id);
        }
    }

    /// Get all group names
    #[inline]
    pub fn get_groups(&self) -> Vec<String> {
        self.groups.keys().cloned().collect()
    }

    /// Get spinners in a group
    #[inline]
    pub fn get_group_spinners(&self, group_name: &str) -> Vec<SpinnerId> {
        self.groups.get(group_name).cloned().unwrap_or_default()
    }

    /// Get the number of groups
    #[inline]
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Clear all spinners and groups
    #[inline]
    pub fn clear(&mut self) {
        self.spinners.clear();
        self.groups.clear();
    }

    /// Apply a style to a specific spinner
    #[inline]
    pub fn set_spinner_style(&mut self, id: SpinnerId, style: Style) {
        if let Some(state) = self.spinners.get_mut(&id) {
            state.set_style(style);
        }
    }

    /// Batch operation: pause all spinners
    #[inline]
    pub fn pause_all(&mut self) {
        for state in self.spinners.values_mut() {
            state.pause();
        }
    }

    /// Batch operation: resume all spinners
    #[inline]
    pub fn resume_all(&mut self) {
        for state in self.spinners.values_mut() {
            state.resume();
        }
    }

    /// Batch operation: reset all spinners
    #[inline]
    pub fn reset_all(&mut self) {
        for state in self.spinners.values_mut() {
            state.reset();
        }
    }
}

impl Default for SpinnerController {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_controller() {
        let mut controller = SpinnerController::new();
        let id = SpinnerId::new(0);

        // Create spinner
        controller.create(id, SpinnerPreset::Dots);
        assert_eq!(controller.spinner_count(), 1);

        // Test tick
        let initial_frame = controller.current_frame(id);
        controller.tick(id);
        assert_ne!(controller.current_frame(id), initial_frame);

        // Test pause/resume
        controller.pause(id);
        assert!(controller.is_paused(id));
        controller.resume(id);
        assert!(!controller.is_paused(id));

        // Test groups
        let id2 = SpinnerId::new(1);
        controller.create(id2, SpinnerPreset::Line);
        controller.create_group("test", vec![id, id2]);
        assert_eq!(controller.group_count(), 1);

        // Test batch tick
        let frame1 = controller.current_frame(id);
        let frame2 = controller.current_frame(id2);
        controller.tick_group("test");
        assert_ne!(controller.current_frame(id), frame1);
        assert_ne!(controller.current_frame(id2), frame2);

        // Test removal
        controller.remove(id);
        assert_eq!(controller.spinner_count(), 1);
    }
}
