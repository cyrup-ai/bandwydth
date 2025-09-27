use crate::network::BandwidthClass;
use tachyonfx::{color_from_hsl, fx, CellFilter, Effect, EffectTimer, Interpolation};

/// Creates sophisticated gradient effects for bandwidth status indicators
pub struct StatusEffects;

impl StatusEffects {
    /// Create a gradient effect that matches the bandwidth class
    pub fn create_class_effect(class: &BandwidthClass) -> Effect {
        match class {
            BandwidthClass::Blazing => Self::create_blazing_effect(),
            BandwidthClass::Excellent => Self::create_excellent_effect(),
            BandwidthClass::Good => Self::create_good_effect(),
            BandwidthClass::Fair => Self::create_fair_effect(),
            BandwidthClass::Poor => Self::create_poor_effect(),
            BandwidthClass::Inconclusive => Self::create_inconclusive_effect(),
        }
    }

    /// Blazing: Fast-cycling fire colors (red-orange-yellow)
    fn create_blazing_effect() -> Effect {
        fx::never_complete(fx::effect_fn(
            (),
            EffectTimer::from_ms(800, Interpolation::Linear),
            |_state, ctx, cell_iter| {
                let time_cycle = ctx.alpha();

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    let position_offset = i as f32 * 15.0; // Rapid position-based color shift
                    let hue = (time_cycle * 180.0 + position_offset) % 60.0; // Red to yellow range
                    let saturation = 95.0 + (time_cycle * 20.0).sin() * 5.0; // High saturation with slight variation
                    let lightness = 50.0 + (time_cycle * 30.0).sin() * 15.0; // Pulsing brightness

                    let color = color_from_hsl(hue, saturation, lightness);
                    cell.set_fg(color);
                });
            },
        ))
    }

    /// Excellent: Smooth green gradient with blue highlights
    fn create_excellent_effect() -> Effect {
        fx::never_complete(fx::effect_fn(
            (),
            EffectTimer::from_ms(1200, Interpolation::Linear),
            |_state, ctx, cell_iter| {
                let time_cycle = ctx.alpha();

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    let position_offset = i as f32 * 8.0;
                    let hue = 120.0 + (time_cycle * 60.0 + position_offset).sin() * 30.0; // Green to cyan range
                    let saturation = 80.0 + (time_cycle * 10.0).cos() * 10.0;
                    let lightness = 55.0 + (time_cycle * 15.0).sin() * 10.0;

                    let color = color_from_hsl(hue, saturation, lightness);
                    cell.set_fg(color);
                });
            },
        ))
    }

    /// Good: Steady blue gradient
    fn create_good_effect() -> Effect {
        fx::never_complete(fx::effect_fn(
            (),
            EffectTimer::from_ms(1600, Interpolation::Linear),
            |_state, ctx, cell_iter| {
                let time_cycle = ctx.alpha();

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    let position_offset = i as f32 * 6.0;
                    let hue = 200.0 + (time_cycle * 40.0 + position_offset).sin() * 20.0; // Blue range
                    let saturation = 75.0 + (time_cycle * 8.0).sin() * 10.0;
                    let lightness = 50.0 + (time_cycle * 12.0).cos() * 8.0;

                    let color = color_from_hsl(hue, saturation, lightness);
                    cell.set_fg(color);
                });
            },
        ))
    }

    /// Fair: Yellow-orange gradient
    fn create_fair_effect() -> Effect {
        fx::never_complete(fx::effect_fn(
            (),
            EffectTimer::from_ms(2000, Interpolation::Linear),
            |_state, ctx, cell_iter| {
                let time_cycle = ctx.alpha();

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    let position_offset = i as f32 * 4.0;
                    let hue = 40.0 + (time_cycle * 30.0 + position_offset).sin() * 15.0; // Yellow-orange range
                    let saturation = 70.0 + (time_cycle * 6.0).cos() * 8.0;
                    let lightness = 55.0 + (time_cycle * 10.0).sin() * 6.0;

                    let color = color_from_hsl(hue, saturation, lightness);
                    cell.set_fg(color);
                });
            },
        ))
    }

    /// Poor: Dull red gradient
    fn create_poor_effect() -> Effect {
        fx::never_complete(fx::effect_fn(
            (),
            EffectTimer::from_ms(2400, Interpolation::Linear),
            |_state, ctx, cell_iter| {
                let time_cycle = ctx.alpha();

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    let position_offset = i as f32 * 3.0;
                    let hue = 0.0 + (time_cycle * 20.0 + position_offset).sin() * 10.0; // Red range
                    let saturation = 60.0 + (time_cycle * 5.0).sin() * 5.0; // Lower saturation
                    let lightness = 45.0 + (time_cycle * 8.0).cos() * 4.0; // Dimmer

                    let color = color_from_hsl(hue, saturation, lightness);
                    cell.set_fg(color);
                });
            },
        ))
    }

    /// Inconclusive: Subtle gray gradient with slow pulse
    fn create_inconclusive_effect() -> Effect {
        fx::never_complete(fx::effect_fn(
            (),
            EffectTimer::from_ms(3000, Interpolation::Linear),
            |_state, ctx, cell_iter| {
                let time_cycle = ctx.alpha();

                cell_iter.enumerate().for_each(|(i, (_pos, cell))| {
                    let position_offset = i as f32 * 2.0;
                    let hue = 0.0; // No hue variation
                    let saturation = 5.0 + (time_cycle * 4.0 + position_offset).sin() * 3.0; // Very low saturation
                    let lightness = 50.0 + (time_cycle * 6.0).cos() * 10.0; // Gentle pulse

                    let color = color_from_hsl(hue, saturation, lightness);
                    cell.set_fg(color);
                });
            },
        ))
    }

    /// Create a cell filter to target only the status indicator text
    pub fn create_status_filter() -> CellFilter {
        // Target cells that contain text
        CellFilter::Text
    }

    /// Get the display text for a bandwidth class (without emoji)
    pub fn get_class_text(class: &BandwidthClass) -> &'static str {
        match class {
            BandwidthClass::Blazing => "Blazing",
            BandwidthClass::Excellent => "Excellent",
            BandwidthClass::Good => "Good",
            BandwidthClass::Fair => "Fair",
            BandwidthClass::Poor => "Poor",
            BandwidthClass::Inconclusive => "Inconclusive",
        }
    }
}
