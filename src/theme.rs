use ratatui::style::Color;

/// A sophisticated color theme for the bandwidth monitor
#[derive(Clone, Debug)]
pub struct Theme {
    /// Dark background color
    pub _bg: Color,
    /// Elevated surface color
    pub surface: Color,
    /// Highlighted surface color
    pub _surface_bright: Color,

    /// Default border color
    pub border: Color,
    /// Active/focused border color
    pub _border_focused: Color,

    /// Primary text color
    pub _text: Color,
    /// Secondary/dimmed text color
    pub text_dim: Color,
    /// Emphasized text color
    pub text_bright: Color,

    /// Primary accent color (blue)
    pub accent: Color,
    /// Dimmed accent color
    pub _accent_dim: Color,

    /// Success status color (green)
    pub _success: Color,
    /// Warning status color (yellow)
    pub _warning: Color,
    /// Error status color (red)
    pub _error: Color,

    /// Excellent bandwidth status color
    pub excellent: Color,
    /// Good bandwidth status color
    pub good: Color,
    /// Fair bandwidth status color
    pub fair: Color,
    /// Poor bandwidth status color
    pub poor: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Dark, sophisticated background
            _bg: Color::Rgb(28, 31, 38),
            surface: Color::Rgb(40, 44, 52),
            _surface_bright: Color::Rgb(50, 55, 65),

            // Subtle borders that pop when focused
            border: Color::Rgb(60, 65, 75),
            _border_focused: Color::Rgb(120, 130, 145),

            // Readable text hierarchy
            _text: Color::Rgb(200, 205, 215),
            text_dim: Color::Rgb(140, 145, 155),
            text_bright: Color::Rgb(240, 245, 255),

            // Modern blue accent
            accent: Color::Rgb(130, 170, 255),
            _accent_dim: Color::Rgb(80, 120, 200),

            // Status colors
            _success: Color::Rgb(130, 220, 150),
            _warning: Color::Rgb(250, 200, 100),
            _error: Color::Rgb(250, 120, 120),

            // Bandwidth-specific status colors
            excellent: Color::Rgb(130, 220, 150), // Green
            good: Color::Rgb(250, 200, 100),      // Yellow
            fair: Color::Rgb(255, 150, 200),      // Pink
            poor: Color::Rgb(250, 120, 120),      // Red
        }
    }
}

impl Theme {
    /// Get color for bandwidth class
    pub fn bandwidth_color(&self, class_name: &str) -> Color {
        match class_name {
            "Excellent" => self.excellent,
            "Good" => self.good,
            "Fair" => self.fair,
            "Poor" => self.poor,
            _ => self.text_dim,
        }
    }

    /// Get styled text color based on value ranges
    pub fn _value_color(&self, value: f64, excellent: f64, good: f64, fair: f64) -> Color {
        if value >= excellent {
            self.excellent
        } else if value >= good {
            self.good
        } else if value >= fair {
            self.fair
        } else {
            self.poor
        }
    }
}

/// Border characters for modern rounded corners
pub mod borders {
    /// Rounded corner border set
    pub const _ROUNDED: [&str; 8] = ["╭", "─", "╮", "│", "╯", "─", "╰", "│"];
    /// Thick border set
    pub const _THICK: [&str; 8] = ["┏", "━", "┓", "┃", "┛", "━", "┗", "┃"];
    /// Double line border set
    pub const _DOUBLE: [&str; 8] = ["╔", "═", "╗", "║", "╝", "═", "╚", "║"];
}

/// Shadow characters for depth effect
pub mod shadow {
    /// Light shadow character
    pub const _LIGHT: &str = "░";
    /// Medium shadow character
    pub const _MEDIUM: &str = "▒";
    /// Dark shadow character
    pub const _DARK: &str = "▓";
}
