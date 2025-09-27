use crate::action::SpinnerSpeed;
use crate::spinners::SpinnerPreset;
use ratatui::style::Style;

use super::SpinnerWidget;

/// Direction of spinner animation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerDirection {
    /// Animate forward through frames
    Forward,
    /// Animate backward through frames
    Backward,
}

impl SpinnerDirection {
    /// Reverse the direction
    #[inline(always)]
    pub const fn reverse(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

/// State for a single spinner instance
///
/// This is a pure event-driven state - no time tracking.
/// Frame advancement happens only through tick events.
#[derive(Debug, Clone)]
pub struct SpinnerState {
    /// The spinner preset defining frames
    preset: SpinnerPreset,
    /// Current frame index
    frame_index: usize,
    /// Total number of frames (cached for performance)
    frame_count: usize,
    /// Animation direction
    direction: SpinnerDirection,
    /// Whether the spinner is paused
    paused: bool,
    /// Speed multiplier (affects how many frames to advance per tick)
    speed: SpinnerSpeed,
    /// Style for rendering
    style: Style,
}

impl SpinnerState {
    /// Create a new spinner state with the given preset
    #[inline]
    pub fn new(preset: SpinnerPreset) -> Self {
        let frame_count = preset.frames().len();
        Self {
            preset,
            frame_index: 0,
            frame_count,
            direction: SpinnerDirection::Forward,
            paused: false,
            speed: SpinnerSpeed::normal(),
            style: Style::default(),
        }
    }

    /// Create with a specific style
    #[inline]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Advance the spinner by one tick
    ///
    /// The actual number of frames advanced depends on the speed multiplier.
    /// When paused, this has no effect.
    #[inline]
    pub fn tick(&mut self) {
        if self.paused || self.frame_count == 0 {
            return;
        }

        // Calculate frames to advance based on speed
        let advance = if self.speed.0 >= 1.0 {
            self.speed.0 as usize
        } else {
            // For speeds < 1.0, we skip ticks instead
            // This is a simple approach that avoids floating point math
            if self.should_skip_tick() {
                return;
            }
            1
        };

        // Advance the frame index
        match self.direction {
            SpinnerDirection::Forward => {
                self.frame_index = (self.frame_index + advance) % self.frame_count;
            }
            SpinnerDirection::Backward => {
                if advance > self.frame_index {
                    // Wrap around
                    self.frame_index =
                        self.frame_count - ((advance - self.frame_index) % self.frame_count);
                } else {
                    self.frame_index -= advance;
                }
            }
        }
    }

    /// Determine if this tick should be skipped for slow speeds
    #[inline]
    fn should_skip_tick(&self) -> bool {
        // Simple deterministic skip pattern based on speed
        if self.speed.0 >= 1.0 {
            false
        } else if self.speed.0 >= 0.5 {
            // Skip every other tick
            self.frame_index.is_multiple_of(2)
        } else if self.speed.0 >= 0.25 {
            // Skip 3 out of 4 ticks
            !self.frame_index.is_multiple_of(4)
        } else {
            // Skip 7 out of 8 ticks for very slow speeds
            !self.frame_index.is_multiple_of(8)
        }
    }

    /// Get the current frame text
    #[inline]
    pub fn current_frame_text(&self) -> &'static str {
        let frames = self.preset.frames();
        frames[self.frame_index]
    }

    /// Get the current frame index
    #[inline(always)]
    pub const fn current_frame(&self) -> usize {
        self.frame_index
    }

    /// Set the frame index directly
    #[inline]
    pub fn set_frame(&mut self, frame: usize) {
        if frame < self.frame_count {
            self.frame_index = frame;
        }
    }

    /// Reset to the first frame
    #[inline]
    pub fn reset(&mut self) {
        self.frame_index = 0;
    }

    /// Pause the spinner
    #[inline]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the spinner
    #[inline]
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if paused
    #[inline(always)]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Toggle pause state
    #[inline]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Reverse the animation direction
    #[inline]
    pub fn reverse(&mut self) {
        self.direction = self.direction.reverse();
    }

    /// Set the animation direction
    #[inline]
    pub fn set_direction(&mut self, direction: SpinnerDirection) {
        self.direction = direction;
    }

    /// Get the current direction
    #[inline(always)]
    pub const fn direction(&self) -> SpinnerDirection {
        self.direction
    }

    /// Set the speed multiplier
    #[inline]
    pub fn set_speed(&mut self, speed: SpinnerSpeed) {
        self.speed = speed;
    }

    /// Get the speed multiplier
    #[inline(always)]
    pub const fn speed(&self) -> SpinnerSpeed {
        self.speed
    }

    /// Set the style
    #[inline]
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Get the style
    #[inline(always)]
    pub const fn style(&self) -> Style {
        self.style
    }

    /// Change the preset and reset state
    #[inline]
    pub fn set_preset(&mut self, preset: SpinnerPreset) {
        self.preset = preset;
        self.frame_count = preset.frames().len();
        self.frame_index = 0;
    }

    /// Get the current preset
    #[inline(always)]
    pub const fn preset(&self) -> SpinnerPreset {
        self.preset
    }

    /// Get the preset name
    #[inline]
    pub fn preset_name(&self) -> &'static str {
        #[allow(unreachable_patterns)]
        match self.preset {
            SpinnerPreset::Dots => "dots",
            SpinnerPreset::Line => "line",
            SpinnerPreset::Arc => "arc",
            SpinnerPreset::BouncingBar => "bounce",
            SpinnerPreset::CircleQuarters => "circle",
            SpinnerPreset::Toggle => "toggle",
            // Handle all other spinner variants (when features are enabled)
            _ => "spinner",
        }
    }

    /// Create a widget for rendering this spinner
    #[inline]
    pub fn widget(&self) -> SpinnerWidget {
        SpinnerWidget::new(self.current_frame_text(), self.style)
    }

    /// Get total frame count
    #[inline(always)]
    pub const fn frame_count(&self) -> usize {
        self.frame_count
    }
}

impl Default for SpinnerState {
    #[inline]
    fn default() -> Self {
        Self::new(SpinnerPreset::Dots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_state_tick() {
        let mut state = SpinnerState::new(SpinnerPreset::Dots);
        assert_eq!(state.current_frame(), 0);

        // Test forward tick
        state.tick();
        assert_eq!(state.current_frame(), 1);

        // Test pause
        state.pause();
        let frame = state.current_frame();
        state.tick();
        assert_eq!(state.current_frame(), frame); // No change when paused

        // Test resume
        state.resume();
        state.tick();
        assert_ne!(state.current_frame(), frame);
    }

    #[test]
    fn test_spinner_direction() {
        let mut state = SpinnerState::new(SpinnerPreset::Dots);
        state.set_frame(2);

        // Test backward
        state.set_direction(SpinnerDirection::Backward);
        state.tick();
        assert_eq!(state.current_frame(), 1);

        // Test wrap around
        state.set_frame(0);
        state.tick();
        assert_eq!(state.current_frame(), state.frame_count() - 1);
    }

    #[test]
    fn test_spinner_speed() {
        let mut state = SpinnerState::new(SpinnerPreset::Dots);

        // Test double speed
        state.set_speed(SpinnerSpeed::double());
        state.tick();
        assert_eq!(state.current_frame(), 2);

        // Test reset
        state.reset();
        assert_eq!(state.current_frame(), 0);
    }
}
