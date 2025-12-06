use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::theme::Theme;

pub fn render_sparkline(data: &[u64], width: usize, theme: &Theme) -> Line<'static> {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if data.is_empty() {
        return Line::from("");
    }

    let min = *data.iter().min().unwrap_or(&0);
    let max = *data.iter().max().unwrap_or(&100);
    let range = if max > min { max - min } else { 1 };

    let step = data.len().max(1) / width.max(1);
    let step = step.max(1);

    let chars: String = data
        .iter()
        .step_by(step)
        .take(width)
        .map(|&v| {
            let normalized = ((v - min) * 7) / range;
            BARS[normalized as usize]
        })
        .collect();

    Line::from(Span::styled(chars, Style::default().fg(theme.accent)))
}

pub fn format_rsi(rsi: f64, theme: &Theme) -> (String, Color) {
    let color = if rsi < 30.0 {
        theme.positive // Oversold
    } else if rsi > 70.0 {
        theme.negative // Overbought
    } else {
        theme.neutral // Neutral
    };
    (format!("{:.1}", rsi), color)
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

pub fn format_macd(line: f64, signal: f64, histogram: f64, theme: &Theme) -> Vec<(String, Color)> {
    let hist_color = if histogram >= 0.0 {
        theme.positive
    } else {
        theme.negative
    };
    let arrow = if histogram >= 0.0 { "▲" } else { "▼" };

    vec![
        (format!("{:+.1}", line), theme.foreground),
        (format!("{:+.1}", signal), theme.foreground_muted),
        (format!("{:+.1} {}", histogram, arrow), hist_color),
    ]
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

pub fn format_volume(volume: f64) -> String {
    if volume >= 1_000_000_000.0 {
        format!("${:.1}B", volume / 1_000_000_000.0)
    } else if volume >= 1_000_000.0 {
        format!("${:.0}M", volume / 1_000_000.0)
    } else {
        format!("${:.0}K", volume / 1_000.0)
    }
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
