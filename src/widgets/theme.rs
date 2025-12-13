//! OpenGL-compatible theme with RGBA float colors

use crate::config::ThemeConfig;

/// RGBA color for OpenGL rendering (values 0.0-1.0)
pub type Color = [f32; 4];

/// Theme colors and spacing for the crypto dashboard
#[derive(Clone, Copy)]
pub struct GlTheme {
    pub foreground: Color,
    pub foreground_muted: Color,
    pub foreground_inactive: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub positive: Color,
    pub negative: Color,
    pub neutral: Color,
    pub selection_bg: Color,
    pub status_live: Color,
    pub status_connecting: Color,
    pub status_disconnected: Color,
    pub status_mock: Color,
    pub candle_bullish: Color,
    pub candle_bearish: Color,
    pub indicator_primary: Color,
    pub indicator_secondary: Color,
    pub indicator_tertiary: Color,
    // Price change intensity colors (high/mid/low)
    pub price_up_high: Color,
    pub price_up_mid: Color,
    pub price_up_low: Color,
    pub price_down_high: Color,
    pub price_down_mid: Color,
    pub price_down_low: Color,
    // Background colors
    pub background: Color,
    pub background_panel: Color,
    pub border: Color,
    pub border_focus: Color,
    // Spacing - single point of configuration
    pub panel_gap: f32,
    pub panel_padding: f32,
    // Font configuration
    pub font_size: f32,
    pub font_small: f32,
    pub font_medium: f32,
    pub font_normal: f32,
    pub font_big: f32,
}

impl Default for GlTheme {
    fn default() -> Self {
        Self {
            foreground: [1.0, 1.0, 1.0, 1.0],          // White
            foreground_muted: [0.5, 0.5, 0.5, 1.0],    // Gray
            foreground_inactive: [0.3, 0.3, 0.3, 1.0], // Dark gray
            accent: [0.0, 1.0, 1.0, 1.0],              // Cyan
            accent_secondary: [1.0, 1.0, 0.0, 1.0],    // Yellow
            positive: [0.0, 1.0, 0.0, 1.0],            // Green
            negative: [1.0, 0.0, 0.0, 1.0],            // Red
            neutral: [1.0, 1.0, 0.0, 1.0],             // Yellow
            selection_bg: [0.2, 0.2, 0.2, 1.0],        // Dark gray
            status_live: [0.0, 1.0, 0.0, 1.0],         // Green
            status_connecting: [1.0, 1.0, 0.0, 1.0],   // Yellow
            status_disconnected: [1.0, 0.0, 0.0, 1.0], // Red
            status_mock: [1.0, 0.0, 1.0, 1.0],         // Magenta
            // Binance green #0ECB81 and red #F6465D
            candle_bullish: [0.055, 0.796, 0.506, 1.0],
            candle_bearish: [0.965, 0.275, 0.365, 1.0],
            indicator_primary: [1.0, 0.647, 0.0, 1.0], // Orange
            indicator_secondary: [1.0, 0.0, 1.0, 1.0], // Magenta
            indicator_tertiary: [0.392, 0.314, 0.471, 1.0],
            // Price change intensity - green shades
            price_up_high: [0.055, 0.796, 0.506, 1.0], // Bright green
            price_up_mid: [0.039, 0.600, 0.380, 1.0],  // Medium green
            price_up_low: [0.024, 0.400, 0.255, 1.0],  // Dim green
            // Price change intensity - red shades
            price_down_high: [0.965, 0.275, 0.365, 1.0], // Bright red
            price_down_mid: [0.725, 0.208, 0.275, 1.0],  // Medium red
            price_down_low: [0.482, 0.137, 0.184, 1.0],  // Dim red
            // Background colors for OpenGL
            background: [0.04, 0.04, 0.06, 1.0], // Main dark
            background_panel: [0.08, 0.08, 0.10, 1.0], // Panel background
            border: [0.25, 0.28, 0.32, 1.0],     // Subtle border
            border_focus: [1.0, 0.8, 0.2, 1.0],  // Focus yellow
            // Spacing
            panel_gap: 8.0,
            panel_padding: 8.0,
            // Font
            font_size: 17.0,
            font_small: 0.8,
            font_medium: 0.9,
            font_normal: 1.0,
            font_big: 1.2,
        }
    }
}

impl GlTheme {
    /// Create theme from config file
    pub fn from_config(config: &ThemeConfig) -> Self {
        let d = Self::default();
        Self {
            foreground: parse_color(config.get("foreground")).unwrap_or(d.foreground),
            foreground_muted: parse_color(config.get("foreground.muted"))
                .unwrap_or(d.foreground_muted),
            foreground_inactive: parse_color(config.get("foreground.inactive"))
                .unwrap_or(d.foreground_inactive),
            accent: parse_color(config.get("accent")).unwrap_or(d.accent),
            accent_secondary: parse_color(config.get("accent.secondary"))
                .unwrap_or(d.accent_secondary),
            positive: parse_color(config.get("positive")).unwrap_or(d.positive),
            negative: parse_color(config.get("negative")).unwrap_or(d.negative),
            neutral: parse_color(config.get("neutral")).unwrap_or(d.neutral),
            selection_bg: parse_color(config.get("selection.background")).unwrap_or(d.selection_bg),
            status_live: parse_color(config.get("status.live")).unwrap_or(d.status_live),
            status_connecting: parse_color(config.get("status.connecting"))
                .unwrap_or(d.status_connecting),
            status_disconnected: parse_color(config.get("status.disconnected"))
                .unwrap_or(d.status_disconnected),
            status_mock: parse_color(config.get("status.mock")).unwrap_or(d.status_mock),
            candle_bullish: parse_color(config.get("candle.bullish")).unwrap_or(d.candle_bullish),
            candle_bearish: parse_color(config.get("candle.bearish")).unwrap_or(d.candle_bearish),
            indicator_primary: parse_color(config.get("indicator.primary"))
                .unwrap_or(d.indicator_primary),
            indicator_secondary: parse_color(config.get("indicator.secondary"))
                .unwrap_or(d.indicator_secondary),
            indicator_tertiary: parse_color(config.get("indicator.tertiary"))
                .unwrap_or(d.indicator_tertiary),
            price_up_high: parse_color(config.get("price.up.high")).unwrap_or(d.price_up_high),
            price_up_mid: parse_color(config.get("price.up.mid")).unwrap_or(d.price_up_mid),
            price_up_low: parse_color(config.get("price.up.low")).unwrap_or(d.price_up_low),
            price_down_high: parse_color(config.get("price.down.high"))
                .unwrap_or(d.price_down_high),
            price_down_mid: parse_color(config.get("price.down.mid")).unwrap_or(d.price_down_mid),
            price_down_low: parse_color(config.get("price.down.low")).unwrap_or(d.price_down_low),
            background: parse_color(config.get("background")).unwrap_or(d.background),
            background_panel: parse_color(config.get("background.panel"))
                .unwrap_or(d.background_panel),
            border: parse_color(config.get("border")).unwrap_or(d.border),
            border_focus: parse_color(config.get("border.focus")).unwrap_or(d.border_focus),
            // Spacing uses defaults
            panel_gap: d.panel_gap,
            panel_padding: d.panel_padding,
            // Font uses defaults
            font_size: d.font_size,
            font_small: d.font_small,
            font_medium: d.font_medium,
            font_normal: d.font_normal,
            font_big: d.font_big,
        }
    }
}

/// Parse a color string (hex or named) to RGBA floats
fn parse_color(s: Option<&str>) -> Option<Color> {
    let s = s?.trim();
    if s.is_empty() {
        return None;
    }

    if s.starts_with('#') {
        return parse_hex(&s[1..]);
    }

    // Named colors mapped to RGBA floats
    match s.to_lowercase().as_str() {
        "black" => Some([0.0, 0.0, 0.0, 1.0]),
        "red" => Some([0.8, 0.0, 0.0, 1.0]),
        "green" => Some([0.0, 0.8, 0.0, 1.0]),
        "yellow" => Some([0.8, 0.8, 0.0, 1.0]),
        "blue" => Some([0.0, 0.0, 0.8, 1.0]),
        "magenta" => Some([0.8, 0.0, 0.8, 1.0]),
        "cyan" => Some([0.0, 0.8, 0.8, 1.0]),
        "gray" | "grey" => Some([0.5, 0.5, 0.5, 1.0]),
        "darkgray" | "darkgrey" => Some([0.3, 0.3, 0.3, 1.0]),
        "lightred" => Some([1.0, 0.4, 0.4, 1.0]),
        "lightgreen" => Some([0.4, 1.0, 0.4, 1.0]),
        "lightyellow" => Some([1.0, 1.0, 0.4, 1.0]),
        "lightblue" => Some([0.4, 0.4, 1.0, 1.0]),
        "lightmagenta" => Some([1.0, 0.4, 1.0, 1.0]),
        "lightcyan" => Some([0.4, 1.0, 1.0, 1.0]),
        "white" => Some([1.0, 1.0, 1.0, 1.0]),
        _ => None,
    }
}

/// Parse hex color string to RGBA floats
fn parse_hex(hex: &str) -> Option<Color> {
    match hex.len() {
        3 => Some([
            u8::from_str_radix(&hex[0..1], 16).ok()? as f32 * 17.0 / 255.0,
            u8::from_str_radix(&hex[1..2], 16).ok()? as f32 * 17.0 / 255.0,
            u8::from_str_radix(&hex[2..3], 16).ok()? as f32 * 17.0 / 255.0,
            1.0,
        ]),
        6 => Some([
            u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0,
            u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0,
            u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0,
            1.0,
        ]),
        _ => None,
    }
}

/// Convert RGB u8 values to RGBA floats
pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
}

/// Convert RGBA u8 values to RGBA floats
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ]
}

/// Blend two colors with alpha
pub fn blend(bg: Color, fg: Color) -> Color {
    let alpha = fg[3];
    let inv_alpha = 1.0 - alpha;
    [
        bg[0] * inv_alpha + fg[0] * alpha,
        bg[1] * inv_alpha + fg[1] * alpha,
        bg[2] * inv_alpha + fg[2] * alpha,
        bg[3] + fg[3] * (1.0 - bg[3]),
    ]
}

/// Darken a color by a factor (0.0-1.0)
pub fn darken(color: Color, factor: f32) -> Color {
    let f = 1.0 - factor;
    [color[0] * f, color[1] * f, color[2] * f, color[3]]
}

/// Lighten a color by a factor (0.0-1.0)
pub fn lighten(color: Color, factor: f32) -> Color {
    [
        color[0] + (1.0 - color[0]) * factor,
        color[1] + (1.0 - color[1]) * factor,
        color[2] + (1.0 - color[2]) * factor,
        color[3],
    ]
}
