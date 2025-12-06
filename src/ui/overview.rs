use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(10),    // Table
        Constraint::Length(3),  // Footer
    ])
    .split(frame.area());

    render_header(frame, chunks[0], &app.theme);
    render_table(frame, chunks[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, theme: &Theme) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  [Tab: ", Style::default().fg(theme.foreground_inactive)),
        Span::styled("Overview", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("]  [", Style::default().fg(theme.foreground_inactive)),
        Span::styled("Details", Style::default().fg(theme.foreground_inactive)),
        Span::styled("]", Style::default().fg(theme.foreground_inactive)),
        Span::raw("                                        "),
        Span::styled("● Live", Style::default().fg(theme.status_live)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Crypto Dashboard ")
            .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    );
    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let header_cells = ["", "PAIR", "PRICE", "24H %", "VOLUME", "HIGH/LOW"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows = app.coins.iter().enumerate().map(|(i, coin)| {
        let is_selected = i == app.selected_index;
        let is_checked = app.checked[i];

        let checkbox = if is_checked { "[x]" } else { "[ ]" };
        let cursor = if is_selected { ">" } else { " " };
        let checkbox_cell = format!("{}{}", cursor, checkbox);

        let pair = format!("{}/USD", coin.symbol);
        let price = format_price(coin.price);
        let change = format!("{:+.2}%", coin.change_24h);
        let volume = format_volume(coin.volume);
        let high_low = format!("{} / {}", format_price_short(coin.high_24h), format_price_short(coin.low_24h));

        let change_color = if coin.change_24h >= 0.0 {
            theme.positive
        } else {
            theme.negative
        };

        let row_style = if is_selected {
            Style::default().bg(theme.selection_bg)
        } else {
            Style::default()
        };

        Row::new(vec![
            Cell::from(checkbox_cell),
            Cell::from(pair).style(Style::default().fg(theme.foreground)),
            Cell::from(price).style(Style::default().fg(theme.foreground)),
            Cell::from(change).style(Style::default().fg(change_color)),
            Cell::from(volume).style(Style::default().fg(theme.foreground_muted)),
            Cell::from(high_low).style(Style::default().fg(theme.foreground_muted)),
        ])
        .style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),   // Checkbox
            Constraint::Length(10),  // Pair
            Constraint::Length(14),  // Price
            Constraint::Length(10),  // Change
            Constraint::Length(10),  // Volume
            Constraint::Length(18),  // High/Low
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let selected_count = app.selected_count();
    let total = app.coins.len();

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  Selected: {}/{}", selected_count, total),
            Style::default().fg(theme.foreground),
        ),
        Span::raw("  │  "),
        Span::styled("[Space]", Style::default().fg(theme.accent_secondary)),
        Span::raw(" Toggle  "),
        Span::styled("[Enter]", Style::default().fg(theme.accent_secondary)),
        Span::raw(" View Details  "),
        Span::styled("[q]", Style::default().fg(theme.accent_secondary)),
        Span::raw(" Quit"),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

fn format_price(price: f64) -> String {
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

fn format_price_short(price: f64) -> String {
    if price >= 1000.0 {
        format!("${:.0}k", price / 1000.0)
    } else if price >= 1.0 {
        format!("${:.0}", price)
    } else {
        format!("${:.2}", price)
    }
}

fn format_volume(volume: f64) -> String {
    if volume >= 1_000_000_000.0 {
        format!("${:.1}B", volume / 1_000_000_000.0)
    } else if volume >= 1_000_000.0 {
        format!("${:.0}M", volume / 1_000_000.0)
    } else {
        format!("${:.0}K", volume / 1_000.0)
    }
}
