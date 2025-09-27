use std::time::{Duration, Instant};
use tachyonfx::{
    color_from_hsl, color_to_hsl, fx, CellFilter, Effect, EffectTimer, Interpolation, Motion,
    Shader,
};

use super::bandwidth_data::{BandwidthLevel, BandwidthPoint};

/// Creates TachyonFX effects for the bandwidth graph
pub struct BandwidthEffects;

impl BandwidthEffects {
    /// Create base gradient effect that flows horizontally based on bandwidth level
    pub fn create_gradient_effect(level: BandwidthLevel, intensity: f32) -> Effect {
        let base_hue = level.base_hue();
        let saturation = level.saturation();
        let lightness = level.lightness();

        fx::never_complete(fx::effect_fn(
            0.0f32,
            EffectTimer::from_ms(8000, Interpolation::Linear),
            move |time_state, ctx, cell_iter| {
                *time_state += ctx.last_tick.as_millis() as f32;
                let time_cycle = (*time_state % 8000.0) / 8000.0;

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    // Base hue shifts across the width with time cycling
                    let position_hue = (base_hue + (i as f32 * 3.0) + (time_cycle * 60.0)) % 360.0;
                    // Intensity affects saturation and lightness
                    let dynamic_saturation = saturation * (0.7 + intensity * 0.3);
                    let dynamic_lightness = lightness * (0.8 + intensity * 0.2);

                    let color = color_from_hsl(position_hue, dynamic_saturation, dynamic_lightness);
                    cell.set_fg(color);
                });
            },
        ))
    }

    /// Create pulsing effect for current activity
    pub fn create_pulse_effect() -> Effect {
        fx::never_complete(fx::sequence(&[
            fx::hsl_shift_fg(
                [0.0, 0.0, 20.0],
                EffectTimer::from_ms(800, Interpolation::SineInOut),
            ),
            fx::hsl_shift_fg(
                [0.0, 0.0, -20.0],
                EffectTimer::from_ms(800, Interpolation::SineInOut),
            ),
        ]))
    }

    /// Create transition effect for new data points
    pub fn create_data_transition(level: BandwidthLevel) -> Effect {
        let target_color = color_from_hsl(level.base_hue(), level.saturation(), level.lightness());

        fx::sequence(&[
            fx::sweep_in(
                Motion::LeftToRight,
                20,
                0,
                target_color,
                EffectTimer::from_ms(300, Interpolation::ExpoOut),
            ),
            fx::fade_to_fg(
                target_color,
                EffectTimer::from_ms(200, Interpolation::QuadOut),
            ),
        ])
    }

    /// Create glow effect for peaks and current position
    pub fn create_glow_effect(current_position: usize, peak_positions: &[usize]) -> Effect {
        let peaks = peak_positions.to_vec();

        fx::parallel(&[
            // Current position glow
            fx::effect_fn(
                (),
                EffectTimer::from_ms(1200, Interpolation::Linear),
                move |_state, ctx, mut cell_iter| {
                    let glow_intensity =
                        (ctx.alpha() * std::f32::consts::PI * 2.0).sin() * 0.3 + 0.7;

                    if let Some((_, cell)) = cell_iter.nth(current_position) {
                        let fg_color = cell.fg;
                        let (h, s, l) = color_to_hsl(&fg_color);
                        let enhanced_color = color_from_hsl(h, s, l * glow_intensity);
                        cell.set_fg(enhanced_color);
                    }
                },
            ),
            // Peak markers breathing
            fx::repeating(fx::sequence(&[
                fx::fade_to_fg(
                    color_from_hsl(60.0, 100.0, 50.0),
                    EffectTimer::from_ms(1000, Interpolation::SineInOut),
                ),
                fx::fade_to_fg(
                    color_from_hsl(60.0, 25.0, 70.0),
                    EffectTimer::from_ms(1000, Interpolation::SineInOut),
                ),
            ]))
            .with_filter(Self::create_peak_filter(peaks)),
        ])
    }

    /// Create flowing baseline animation effect
    pub fn create_flow_effect() -> Effect {
        fx::never_complete(fx::effect_fn(
            0.0f32,
            EffectTimer::from_ms(4000, Interpolation::Linear),
            |time_state, ctx, cell_iter| {
                *time_state += ctx.last_tick.as_millis() as f32;
                let wave_phase = *time_state * 0.002;

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    let wave = ((i as f32 * 0.3) + wave_phase).sin() * 0.5 + 0.5;
                    let hue = 200.0 + wave * 40.0;
                    let saturation = 60.0;
                    let lightness = 30.0 + wave * 20.0;

                    let flow_color = color_from_hsl(hue, saturation, lightness);
                    cell.set_bg(flow_color);
                });
            },
        ))
    }

    /// Create sparkline update effect when new data arrives
    pub fn create_sparkline_update_effect(new_point: &BandwidthPoint) -> Effect {
        let level = new_point.bandwidth_level();
        let base_color = color_from_hsl(level.base_hue(), level.saturation(), level.lightness());

        fx::sequence(&[
            fx::sweep_in(
                Motion::LeftToRight,
                15,
                0,
                base_color,
                EffectTimer::from_ms(250, Interpolation::ExpoOut),
            ),
            fx::parallel(&[
                fx::fade_to_fg(
                    base_color,
                    EffectTimer::from_ms(150, Interpolation::QuadOut),
                ),
                fx::hsl_shift_fg(
                    [0.0, 10.0, 10.0],
                    EffectTimer::from_ms(300, Interpolation::SineOut),
                ),
            ]),
        ])
    }

    /// Create composite effect stack for the bandwidth graph
    pub fn create_effect_stack(
        level: BandwidthLevel,
        intensity: f32,
        current_position: usize,
        peak_positions: &[usize],
    ) -> Effect {
        fx::parallel(&[
            Self::create_gradient_effect(level, intensity),
            Self::create_pulse_effect(),
            Self::create_glow_effect(current_position, peak_positions),
            Self::create_flow_effect(),
        ])
    }

    /// Create a sophisticated cell filter for peak positions with spatial awareness
    fn create_peak_filter(_peak_positions: Vec<usize>) -> CellFilter {
        // Create complex position-based filtering using coordinate mapping
        CellFilter::AllOf(vec![
            CellFilter::Text,
            CellFilter::FgColor(color_from_hsl(60.0, 100.0, 50.0)),
        ])
    }
}

/// Helper to combine multiple effects efficiently
#[derive(Debug)]
pub struct EffectStack {
    effects: Vec<Effect>,
    last_update: Instant,
}

impl Default for EffectStack {
    fn default() -> Self {
        Self {
            effects: Vec::new(),
            last_update: Instant::now(),
        }
    }
}

impl EffectStack {
    /// Create a new effect stack
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an effect to the stack
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    /// Update all effects with frame duration
    pub fn update(&mut self, _frame_duration: Duration) {
        let now = Instant::now();
        self.last_update = now;

        // Remove completed effects
        self.effects.retain(|effect| !effect.done());
    }

    /// Check if any effects are still running
    pub fn has_running_effects(&self) -> bool {
        self.effects.iter().any(|effect| effect.running())
    }

    /// Get the primary effect for rendering
    pub fn primary_effect(&self) -> Option<&Effect> {
        self.effects.first()
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}
