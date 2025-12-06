use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::theme::Theme;

/// Calculate color for price based on change compared to historical average
///
/// Compares current change magnitude against the average of recent changes:
/// - Level 1 (light): change <= avg
/// - Level 2 (medium): avg < change <= 2*avg
/// - Level 3 (strong): change > 3*avg
///
/// Returns one of 7 colors: 3 red gradients, gray, or 3 green gradients
pub fn price_change_color(current: f64, previous: f64, avg_change: f64) -> Color {
    // Calculate the change
    let change = current - previous;

    // No change - gray
    if change == 0.0 {
        return Color::Rgb(128, 128, 128);
    }

    let abs_change = change.abs();

    // No history yet - use a subtle color based on direction
    if avg_change <= 0.0 {
        return if change > 0.0 {
            Color::Rgb(100, 180, 100)   // Light green
        } else {
            Color::Rgb(180, 100, 100)   // Light red
        };
    }

    // Compare change to average and determine level
    let ratio = abs_change / avg_change;

    if change > 0.0 {
        // Price went UP - green gradients
        if ratio > 3.0 {
            Color::Rgb(0, 255, 0)       // Level 3: Strong green (> 3x avg)
        } else if ratio > 2.0 {
            Color::Rgb(50, 205, 50)     // Level 2: Medium green (2-3x avg)
        } else {
            Color::Rgb(100, 180, 100)   // Level 1: Light green (<= avg)
        }
    } else {
        // Price went DOWN - red gradients
        if ratio > 3.0 {
            Color::Rgb(255, 0, 0)       // Level 3: Strong red (> 3x avg)
        } else if ratio > 2.0 {
            Color::Rgb(220, 50, 50)     // Level 2: Medium red (2-3x avg)
        } else {
            Color::Rgb(180, 100, 100)   // Level 1: Light red (<= avg)
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
