use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
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
