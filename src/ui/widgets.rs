use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::theme::Theme;

// Grid-based number rendering: 5 wide x 7 tall = 35 positions per digit
// Each digit is defined as a flat array, row-by-row, left-to-right
// 0 = empty space, 1 = filled block (■)
// Placeholder patterns - actual designs to be provided externally
pub const DIGIT_PATTERNS: [[u8; 35]; 10] = [
    // 0
    [
        0, 1, 1, 1, 0,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 1
    [
        0, 0, 1, 0, 0,
        0, 1, 1, 0, 0,
        0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
        0, 1, 1, 1, 0,
    ],
    // 2
    [
        0, 1, 1, 1, 0,
        1, 0, 0, 0, 1,
        0, 0, 0, 0, 1,
        0, 0, 1, 1, 0,
        0, 1, 0, 0, 0,
        1, 0, 0, 0, 0,
        1, 1, 1, 1, 1,
    ],
    // 3
    [
        0, 1, 1, 1, 0,
        1, 0, 0, 0, 1,
        0, 0, 0, 0, 1,
        0, 0, 1, 1, 0,
        0, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 4
    [
        0, 0, 0, 1, 0,
        0, 0, 1, 1, 0,
        0, 1, 0, 1, 0,
        1, 0, 0, 1, 0,
        1, 1, 1, 1, 1,
        0, 0, 0, 1, 0,
        0, 0, 0, 1, 0,
    ],
    // 5
    [
        1, 1, 1, 1, 1,
        1, 0, 0, 0, 0,
        1, 1, 1, 1, 0,
        0, 0, 0, 0, 1,
        0, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 6
    [
        0, 1, 1, 1, 0,
        1, 0, 0, 0, 0,
        1, 0, 0, 0, 0,
        1, 1, 1, 1, 0,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 7
    [
        1, 1, 1, 1, 1,
        0, 0, 0, 0, 1,
        0, 0, 0, 1, 0,
        0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
        0, 0, 1, 0, 0,
    ],
    // 8
    [
        0, 1, 1, 1, 0,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
    // 9
    [
        0, 1, 1, 1, 0,
        1, 0, 0, 0, 1,
        1, 0, 0, 0, 1,
        0, 1, 1, 1, 1,
        0, 0, 0, 0, 1,
        0, 0, 0, 0, 1,
        0, 1, 1, 1, 0,
    ],
];

// Special character patterns (narrower: 3 wide x 7 tall = 21 positions)
pub const SPECIAL_PATTERNS: [([u8; 21], char); 6] = [
    // Decimal point
    (
        [
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
            0, 1, 0,
            0, 1, 0,
        ],
        '.',
    ),
    // Minus
    (
        [
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
            1, 1, 1,
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
        ],
        '-',
    ),
    // Dollar sign
    (
        [
            0, 1, 0,
            1, 1, 1,
            1, 0, 0,
            0, 1, 0,
            0, 0, 1,
            1, 1, 1,
            0, 1, 0,
        ],
        '$',
    ),
    // Percent sign
    (
        [
            1, 0, 0,
            1, 0, 1,
            0, 0, 1,
            0, 1, 0,
            1, 0, 0,
            1, 0, 1,
            0, 0, 1,
        ],
        '%',
    ),
    // Comma
    (
        [
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
            0, 1, 0,
            0, 1, 0,
            1, 0, 0,
        ],
        ',',
    ),
    // Plus
    (
        [
            0, 0, 0,
            0, 1, 0,
            0, 1, 0,
            1, 1, 1,
            0, 1, 0,
            0, 1, 0,
            0, 0, 0,
        ],
        '+',
    ),
];

const GRID_CHAR_FILLED: char = '█';
const GRID_CHAR_EMPTY: char = ' ';
const DIGIT_WIDTH: usize = 5;
const DIGIT_HEIGHT: usize = 7;
const SPECIAL_WIDTH: usize = 3;
const DIGIT_SPACING: usize = 1;

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

// =============================================================================
// Grid-based number rendering functions
// =============================================================================

/// Get the pattern for a single character (digit or special)
fn get_char_pattern(c: char) -> Option<(Vec<u8>, usize)> {
    if let Some(digit) = c.to_digit(10) {
        Some((DIGIT_PATTERNS[digit as usize].to_vec(), DIGIT_WIDTH))
    } else {
        SPECIAL_PATTERNS
            .iter()
            .find(|(_, ch)| *ch == c)
            .map(|(pattern, _)| (pattern.to_vec(), SPECIAL_WIDTH))
    }
}

/// Render a single row of a character's grid pattern
fn render_char_row(pattern: &[u8], width: usize, row: usize) -> String {
    let start = row * width;
    let end = start + width;

    if end > pattern.len() {
        return " ".repeat(width);
    }

    pattern[start..end]
        .iter()
        .map(|&p| if p == 1 { GRID_CHAR_FILLED } else { GRID_CHAR_EMPTY })
        .collect()
}

/// Padding for grid number rendering (top, right, bottom, left) - like CSS
pub type GridPadding = (usize, usize, usize, usize);

/// Render a complete number string as lines of grid characters with padding
/// Returns Vec of Lines with padding applied (top, right, bottom, left)
pub fn render_number_grid(number: &str, color: Color, padding: GridPadding) -> Vec<Line<'static>> {
    let (pad_top, pad_right, pad_bottom, pad_left) = padding;

    // Collect all character patterns
    let char_patterns: Vec<(Vec<u8>, usize)> = number
        .chars()
        .filter_map(get_char_pattern)
        .collect();

    if char_patterns.is_empty() {
        return vec![Line::from(""); pad_top + DIGIT_HEIGHT + pad_bottom];
    }

    // Calculate content width for padding
    let content_width: usize = char_patterns.iter()
        .map(|(_, w)| w)
        .sum::<usize>() + (char_patterns.len().saturating_sub(1) * DIGIT_SPACING);

    let left_pad = " ".repeat(pad_left);
    let right_pad = " ".repeat(pad_right);
    let empty_line = " ".repeat(pad_left + content_width + pad_right);

    let mut lines: Vec<Line<'static>> = Vec::with_capacity(pad_top + DIGIT_HEIGHT + pad_bottom);

    // Top padding
    for _ in 0..pad_top {
        lines.push(Line::from(empty_line.clone()));
    }

    // Build each row of the grid
    for row in 0..DIGIT_HEIGHT {
        let mut row_str = left_pad.clone();

        for (i, (pattern, width)) in char_patterns.iter().enumerate() {
            if i > 0 {
                row_str.push_str(&" ".repeat(DIGIT_SPACING));
            }
            row_str.push_str(&render_char_row(pattern, *width, row));
        }

        row_str.push_str(&right_pad);

        lines.push(Line::from(Span::styled(
            row_str,
            Style::default().fg(color),
        )));
    }

    // Bottom padding
    for _ in 0..pad_bottom {
        lines.push(Line::from(empty_line.clone()));
    }

    lines
}

/// Calculate the total width needed to render a number string
pub fn grid_number_width(number: &str) -> usize {
    let mut width = 0;
    let mut first = true;

    for c in number.chars() {
        if let Some((_, char_width)) = get_char_pattern(c) {
            if !first {
                width += DIGIT_SPACING;
            }
            width += char_width;
            first = false;
        }
    }

    width
}

/// Render a grid number into a frame at a specific area
pub fn render_grid_number(
    frame: &mut Frame,
    area: Rect,
    number: &str,
    color: Color,
    padding: GridPadding,
    block: Option<Block>,
) {
    let lines = render_number_grid(number, color, padding);
    let paragraph = if let Some(b) = block {
        Paragraph::new(lines).block(b)
    } else {
        Paragraph::new(lines)
    };
    frame.render_widget(paragraph, area);
}
