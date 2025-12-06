use ratatui::style::Color;

use crate::config::ThemeConfig;

#[derive(Clone, Copy)]
pub struct Theme {
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
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            foreground: Color::White,
            foreground_muted: Color::Gray,
            foreground_inactive: Color::DarkGray,
            accent: Color::Cyan,
            accent_secondary: Color::Yellow,
            positive: Color::Green,
            negative: Color::Red,
            neutral: Color::Yellow,
            selection_bg: Color::DarkGray,
            status_live: Color::Green,
            status_connecting: Color::Yellow,
            status_disconnected: Color::Red,
            status_mock: Color::Magenta,
            candle_bullish: Color::Rgb(14, 203, 129),  // Binance green #0ECB81
            candle_bearish: Color::Rgb(246, 70, 93),   // Binance red #F6465D
            indicator_primary: Color::Rgb(255, 165, 0),
            indicator_secondary: Color::Magenta,
            indicator_tertiary: Color::Rgb(100, 80, 120),
            // Price change intensity - green shades
            price_up_high: Color::Rgb(14, 203, 129),   // Bright green
            price_up_mid: Color::Rgb(10, 153, 97),     // Medium green
            price_up_low: Color::Rgb(6, 102, 65),      // Dim green
            // Price change intensity - red shades
            price_down_high: Color::Rgb(246, 70, 93),  // Bright red
            price_down_mid: Color::Rgb(185, 53, 70),   // Medium red
            price_down_low: Color::Rgb(123, 35, 47),   // Dim red
        }
    }
}

impl Theme {
    pub fn from_config(config: &ThemeConfig) -> Self {
        let d = Self::default();
        Self {
            foreground: parse_color(config.get("foreground")).unwrap_or(d.foreground),
            foreground_muted: parse_color(config.get("foreground.muted")).unwrap_or(d.foreground_muted),
            foreground_inactive: parse_color(config.get("foreground.inactive")).unwrap_or(d.foreground_inactive),
            accent: parse_color(config.get("accent")).unwrap_or(d.accent),
            accent_secondary: parse_color(config.get("accent.secondary")).unwrap_or(d.accent_secondary),
            positive: parse_color(config.get("positive")).unwrap_or(d.positive),
            negative: parse_color(config.get("negative")).unwrap_or(d.negative),
            neutral: parse_color(config.get("neutral")).unwrap_or(d.neutral),
            selection_bg: parse_color(config.get("selection.background")).unwrap_or(d.selection_bg),
            status_live: parse_color(config.get("status.live")).unwrap_or(d.status_live),
            status_connecting: parse_color(config.get("status.connecting")).unwrap_or(d.status_connecting),
            status_disconnected: parse_color(config.get("status.disconnected")).unwrap_or(d.status_disconnected),
            status_mock: parse_color(config.get("status.mock")).unwrap_or(d.status_mock),
            candle_bullish: parse_color(config.get("candle.bullish")).unwrap_or(d.candle_bullish),
            candle_bearish: parse_color(config.get("candle.bearish")).unwrap_or(d.candle_bearish),
            indicator_primary: parse_color(config.get("indicator.primary")).unwrap_or(d.indicator_primary),
            indicator_secondary: parse_color(config.get("indicator.secondary")).unwrap_or(d.indicator_secondary),
            indicator_tertiary: parse_color(config.get("indicator.tertiary")).unwrap_or(d.indicator_tertiary),
            price_up_high: parse_color(config.get("price.up.high")).unwrap_or(d.price_up_high),
            price_up_mid: parse_color(config.get("price.up.mid")).unwrap_or(d.price_up_mid),
            price_up_low: parse_color(config.get("price.up.low")).unwrap_or(d.price_up_low),
            price_down_high: parse_color(config.get("price.down.high")).unwrap_or(d.price_down_high),
            price_down_mid: parse_color(config.get("price.down.mid")).unwrap_or(d.price_down_mid),
            price_down_low: parse_color(config.get("price.down.low")).unwrap_or(d.price_down_low),
        }
    }
}

fn parse_color(s: Option<&str>) -> Option<Color> {
    let s = s?.trim();
    if s.is_empty() {
        return None;
    }

    if s.starts_with('#') {
        return parse_hex(&s[1..]);
    }

    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => None,
    }
}

fn parse_hex(hex: &str) -> Option<Color> {
    match hex.len() {
        3 => Some(Color::Rgb(
            u8::from_str_radix(&hex[0..1], 16).ok()? * 17,
            u8::from_str_radix(&hex[1..2], 16).ok()? * 17,
            u8::from_str_radix(&hex[2..3], 16).ok()? * 17,
        )),
        6 => Some(Color::Rgb(
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        )),
        _ => None,
    }
}
