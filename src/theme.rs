use ratatui::style::Color;

use crate::config::ThemeColors;

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
        }
    }
}

impl Theme {
    pub fn from_colors(colors: &ThemeColors) -> Self {
        let default = Self::default();

        Self {
            foreground: parse_color(&colors.foreground).unwrap_or(default.foreground),
            foreground_muted: parse_color(&colors.foreground_muted).unwrap_or(default.foreground_muted),
            foreground_inactive: parse_color(&colors.foreground_inactive).unwrap_or(default.foreground_inactive),
            accent: parse_color(&colors.accent).unwrap_or(default.accent),
            accent_secondary: parse_color(&colors.accent_secondary).unwrap_or(default.accent_secondary),
            positive: parse_color(&colors.positive).unwrap_or(default.positive),
            negative: parse_color(&colors.negative).unwrap_or(default.negative),
            neutral: parse_color(&colors.neutral).unwrap_or(default.neutral),
            selection_bg: parse_color(&colors.selection_background).unwrap_or(default.selection_bg),
            status_live: parse_color(&colors.status_live).unwrap_or(default.status_live),
        }
    }
}

fn parse_color(s: &str) -> Option<Color> {
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
        "" | "null" | "default" => None,
        _ => None,
    }
}
