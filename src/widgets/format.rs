//! Formatting utilities for displaying prices, volumes, and percentages

use super::theme::{Color, GlTheme};

/// Format price with appropriate precision and commas
pub fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        let whole = price as u64;
        let frac = ((price - whole as f64) * 100.0).round() as u64;
        let formatted = format_with_commas(whole);
        format!("${}.{:02}", formatted, frac)
    } else if price >= 1.0 {
        format!("${:.2}", price)
    } else if price >= 0.01 {
        format!("${:.4}", price)
    } else {
        format!("${:.6}", price)
    }
}

/// Format price in short form (e.g., "$67k")
pub fn format_price_short(price: f64) -> String {
    if price >= 1000.0 {
        format!("${:.0}k", price / 1000.0)
    } else if price >= 1.0 {
        format!("${:.0}", price)
    } else {
        format!("${:.2}", price)
    }
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format percentage change with sign
pub fn format_change(change: f64) -> String {
    format!("{:+.2}%", change)
}

/// Get color and arrow for a change value
pub fn change_color_and_arrow(change: f64, theme: &GlTheme) -> (Color, &'static str) {
    if change >= 0.0 {
        (theme.positive, "▲")
    } else {
        (theme.negative, "▼")
    }
}

/// Format volume with USD and base amounts
pub fn format_volume_full(volume_usd: f64, volume_base: f64, symbol: &str) -> String {
    let usd_part = if volume_usd >= 1_000_000_000.0 {
        format!("${:.1}B", volume_usd / 1_000_000_000.0)
    } else if volume_usd >= 1_000_000.0 {
        format!("${:.0}M", volume_usd / 1_000_000.0)
    } else if volume_usd >= 1_000.0 {
        format!("${:.0}K", volume_usd / 1_000.0)
    } else {
        format!("${:.0}", volume_usd)
    };

    let base_part = if volume_base >= 1_000_000.0 {
        format!("{:.1}M", volume_base / 1_000_000.0)
    } else if volume_base >= 1_000.0 {
        format!("{:.0}K", volume_base / 1_000.0)
    } else {
        format!("{:.0}", volume_base)
    };

    format!("{} USD / {} {}", usd_part, base_part, symbol)
}

/// Format volume in short form
pub fn format_volume_short(volume_usd: f64, volume_base: f64) -> String {
    let usd = if volume_usd >= 1_000_000_000.0 {
        format!("${:.1}B", volume_usd / 1_000_000_000.0)
    } else if volume_usd >= 1_000_000.0 {
        format!("${:.0}M", volume_usd / 1_000_000.0)
    } else {
        format!("${:.0}K", volume_usd / 1_000.0)
    };

    let base = if volume_base >= 1_000_000.0 {
        format!("{:.1}M", volume_base / 1_000_000.0)
    } else if volume_base >= 1_000.0 {
        format!("{:.0}K", volume_base / 1_000.0)
    } else {
        format!("{:.0}", volume_base)
    };

    format!("{} / {}", usd, base)
}

/// Capitalize first letter
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Calculate color for price based on change compared to historical average
pub fn price_change_color(current: f64, previous: f64, avg_change: f64, theme: &GlTheme) -> Color {
    let change = current - previous;

    if change == 0.0 {
        return theme.neutral;
    }

    let abs_change = change.abs();
    let is_up = change > 0.0;

    // No history yet - use low intensity
    if avg_change <= 0.0 {
        return if is_up {
            theme.price_up_low
        } else {
            theme.price_down_low
        };
    }

    // Compare change to average and determine level
    let ratio = abs_change / avg_change;

    if is_up {
        if ratio > 2.0 {
            theme.price_up_high
        } else if ratio > 1.0 {
            theme.price_up_mid
        } else {
            theme.price_up_low
        }
    } else {
        if ratio > 2.0 {
            theme.price_down_high
        } else if ratio > 1.0 {
            theme.price_down_mid
        } else {
            theme.price_down_low
        }
    }
}

/// Format time remaining as HH:MM:SS
pub fn format_time_remaining(hours: u32, minutes: u32, seconds: u32) -> String {
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Calculate time remaining until next candle
pub fn calculate_time_remaining(granularity: u32) -> (u32, u32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let current_candle_start = (now / granularity as u64) * granularity as u64;
    let next_candle = current_candle_start + granularity as u64;
    let remaining = next_candle - now;

    let hours = (remaining / 3600) as u32;
    let minutes = ((remaining % 3600) / 60) as u32;
    let seconds = (remaining % 60) as u32;

    (hours, minutes, seconds)
}
