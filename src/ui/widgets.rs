use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::Span,
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType,
        canvas::{Canvas, Line},
    },
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api::Candle;

use crate::theme::Theme;

/// Calculate color for price based on change compared to historical average
///
/// Compares current change magnitude against the average of recent changes:
/// - Low: change <= avg
/// - Mid: avg < change <= 2*avg
/// - High: change > 2*avg
///
/// Returns discrete colors from theme (price.up/down.high/mid/low)
pub fn price_change_color(current: f64, previous: f64, avg_change: f64, theme: &Theme) -> Color {
    let change = current - previous;

    if change == 0.0 {
        return theme.neutral;
    }

    let abs_change = change.abs();
    let is_up = change > 0.0;

    // No history yet - use low intensity
    if avg_change <= 0.0 {
        return if is_up { theme.price_up_low } else { theme.price_down_low };
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

pub fn format_change(change: f64, theme: &Theme) -> (String, Color, &'static str) {
    let color = if change >= 0.0 {
        theme.positive
    } else {
        theme.negative
    };
    let arrow = if change >= 0.0 { "▲" } else { "▼" };
    (format!("{:+.2}%", change), color, arrow)
}

pub fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        let whole = price as u64;
        let frac = ((price - whole as f64) * 100.0).round() as u64;
        let formatted = format_with_commas(whole);
        format!("${}.{:02}", formatted, frac)
    } else if price >= 1.0 {
        format!("${:.2}", price)
    } else {
        format!("${:.4}", price)
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

/// Format volume as "$722M USD / 511,319 ETH"
pub fn format_volume_full(volume_usd: f64, volume_base: f64, symbol: &str) -> String {
    let usd_part = if volume_usd >= 1_000_000_000.0 {
        format!("${:.1}B", volume_usd / 1_000_000_000.0)
    } else if volume_usd >= 1_000_000.0 {
        format!("${:.0}M", volume_usd / 1_000_000.0)
    } else {
        format!("${:.0}K", volume_usd / 1_000.0)
    };

    let base_part = format_base_volume(volume_base);
    format!("{} USD / {} {}", usd_part, base_part, symbol)
}

fn format_base_volume(volume: f64) -> String {
    if volume >= 1_000_000.0 {
        format!("{:.1}M", volume / 1_000_000.0)
    } else if volume >= 1_000.0 {
        format_with_commas(volume as u64)
    } else {
        format!("{:.0}", volume)
    }
}

/// Render a price chart using ratatui's Chart widget
pub fn render_price_chart(
    frame: &mut Frame,
    area: Rect,
    data: &[(f64, f64)],
    bounds: (f64, f64),
    window: &str,
    theme: &Theme,
) {
    if data.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Price ({}) - No data ", window))
            .title_style(Style::default().fg(theme.foreground_muted));
        frame.render_widget(block, area);
        return;
    }

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.accent))
        .data(data);

    let title = format!(" Price ({}) ", window);
    let x_max = data.len() as f64;

    // Format Y-axis labels based on price magnitude
    let y_labels = create_y_labels(bounds.0, bounds.1);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(theme.foreground_muted))
                .bounds([0.0, x_max]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(theme.foreground_muted))
                .labels(y_labels)
                .bounds([bounds.0, bounds.1]),
        );

    frame.render_widget(chart, area);
}

/// Create Y-axis labels for the price chart
fn create_y_labels(min: f64, max: f64) -> Vec<Span<'static>> {
    let format_price_label = |price: f64| -> String {
        if price >= 10000.0 {
            format!("${:.0}k", price / 1000.0)
        } else if price >= 1000.0 {
            format!("${:.1}k", price / 1000.0)
        } else if price >= 1.0 {
            format!("${:.2}", price)
        } else {
            format!("${:.4}", price)
        }
    };

    let mid = (min + max) / 2.0;
    vec![
        Span::raw(format_price_label(min)),
        Span::raw(format_price_label(mid)),
        Span::raw(format_price_label(max)),
    ]
}

/// Calculate time remaining until current candle closes
pub fn calculate_time_remaining(last_candle_time: i64, granularity: u32) -> (u32, u32, u32) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let candle_end = last_candle_time + granularity as i64;
    let remaining = (candle_end - now).max(0) as u32;

    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    (hours, minutes, seconds)
}

/// Format time remaining as HH:MM:SS
pub fn format_time_remaining(hours: u32, minutes: u32, seconds: u32) -> String {
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Render a candlestick chart using Canvas for sub-character precision
/// 3-unit width per candle, wick in center, with navigation
/// Returns the clamped scroll_offset to sync back to App state
pub fn render_candlestick_chart(
    frame: &mut Frame,
    area: Rect,
    candles: &[Candle],
    window: &str,
    theme: &Theme,
    time_remaining: Option<(u32, u32, u32)>,
    scroll_offset: isize,
) -> isize {
    // Build title with optional countdown timer
    let title = match time_remaining {
        Some((h, m, s)) => format!(" Candles ({}) [{}] ", window, format_time_remaining(h, m, s)),
        None => format!(" Candles ({}) ", window),
    };

    if candles.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Candles ({}) - No data ", window))
            .title_style(Style::default().fg(theme.foreground_muted));
        frame.render_widget(block, area);
        return scroll_offset;
    }

    // Each candle takes ~3 chars width, calculate visible slots
    let inner_width = area.width.saturating_sub(2) as usize;
    let visible_slots = (inner_width / 3).max(1);
    let total_candles = candles.len();

    // Clamp scroll_offset:
    // - Left limit: can't scroll past all candles (leave at least 1 visible)
    // - Right limit: must show at least 2 candles (negative offset = empty space on right)
    let max_left = (total_candles.saturating_sub(1)) as isize;
    let max_right = -((visible_slots.saturating_sub(2)) as isize);
    let scroll_offset = scroll_offset.clamp(max_right, max_left);

    // Calculate which candles to display and empty space on right
    let (start_idx, end_idx, empty_right) = if scroll_offset >= 0 {
        // Normal left scroll (viewing history)
        let offset = scroll_offset as usize;
        let end = total_candles.saturating_sub(offset);
        let start = end.saturating_sub(visible_slots);
        (start, end, 0usize)
    } else {
        // Right pan - show fewer candles with empty space on right
        let right_offset = (-scroll_offset) as usize;
        let candles_to_show = visible_slots.saturating_sub(right_offset).max(2);
        let start = total_candles.saturating_sub(candles_to_show);
        (start, total_candles, right_offset)
    };

    let display_candles = &candles[start_idx..end_idx];

    if display_candles.is_empty() {
        return scroll_offset;
    }

    let num_candles = display_candles.len();
    let total_slots = num_candles + empty_right;

    // Calculate price bounds for visible candles only (auto-scale)
    let min_price = display_candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
    let max_price = display_candles.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);

    // Candle spacing: 3 units per candle
    let candle_unit_width = 3.0;
    let body_width = 2.0;

    // Prepare candle data for closure (using theme colors)
    let bullish_color = theme.candle_bullish;
    let bearish_color = theme.candle_bearish;

    let candles_data: Vec<(f64, f64, f64, f64, Color)> = display_candles
        .iter()
        .map(|c| {
            let color = if c.close >= c.open { bullish_color } else { bearish_color };
            (c.open, c.high, c.low, c.close, color)
        })
        .collect();

    let canvas = Canvas::default()
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL),
        )
        .marker(Marker::Braille)
        .x_bounds([0.0, (total_slots as f64) * candle_unit_width])
        .y_bounds([min_price, max_price])
        .paint(move |ctx| {
            for (i, (open, high, low, close, color)) in candles_data.iter().enumerate() {
                let x_center = (i as f64 * candle_unit_width) + (candle_unit_width / 2.0);

                // Draw wick (vertical line from low to high)
                ctx.draw(&Line {
                    x1: x_center,
                    y1: *low,
                    x2: x_center,
                    y2: *high,
                    color: *color,
                });

                // Draw body (thicker vertical line from open to close)
                let body_bottom = open.min(*close);
                let body_top = open.max(*close);

                let half_width = body_width / 2.0;
                ctx.draw(&Line {
                    x1: x_center - half_width,
                    y1: body_bottom,
                    x2: x_center - half_width,
                    y2: body_top,
                    color: *color,
                });
                ctx.draw(&Line {
                    x1: x_center,
                    y1: body_bottom,
                    x2: x_center,
                    y2: body_top,
                    color: *color,
                });
                ctx.draw(&Line {
                    x1: x_center + half_width,
                    y1: body_bottom,
                    x2: x_center + half_width,
                    y2: body_top,
                    color: *color,
                });
            }
        });

    frame.render_widget(canvas, area);
    scroll_offset
}
