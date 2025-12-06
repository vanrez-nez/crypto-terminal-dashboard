use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, ConnectionStatus};
use crate::mock::CoinData;
use crate::theme::Theme;
use super::widgets;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(10),    // Content
        Constraint::Length(3),  // Footer
    ])
    .split(frame.area());

    render_header(frame, chunks[0], app);
    render_content(frame, chunks[1], app);
    render_footer(frame, chunks[2], &app.theme);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let (status_text, status_color) = match app.connection_status {
        ConnectionStatus::Connected => ("● Live", theme.status_live),
        ConnectionStatus::Connecting => ("◌ Connecting", Color::Yellow),
        ConnectionStatus::Disconnected => ("○ Disconnected", Color::Red),
        ConnectionStatus::Mock => ("◆ Mock", Color::Magenta),
    };

    let provider_display = capitalize(&app.provider);
    let window_display = app.time_window.as_str();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  [", Style::default().fg(theme.foreground_inactive)),
        Span::styled("Overview", Style::default().fg(theme.foreground_inactive)),
        Span::styled("]  [Tab: ", Style::default().fg(theme.foreground_inactive)),
        Span::styled("Details", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("]", Style::default().fg(theme.foreground_inactive)),
        Span::raw("    "),
        Span::styled("Provider: ", Style::default().fg(theme.foreground_muted)),
        Span::styled(&provider_display, Style::default().fg(theme.foreground)),
        Span::raw("    "),
        Span::styled("[w] Window: ", Style::default().fg(theme.foreground_muted)),
        Span::styled(window_display, Style::default().fg(theme.accent)),
        Span::raw("    "),
        Span::styled(status_text, Style::default().fg(status_color)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Crypto Dashboard ")
            .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    );
    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.selected_coins();

    if selected.is_empty() {
        let msg = Paragraph::new("No coins selected. Press Tab to go back and select up to 3 coins.")
            .style(Style::default().fg(app.theme.foreground_muted))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(msg, area);
        return;
    }

    let constraints: Vec<Constraint> = selected
        .iter()
        .map(|_| Constraint::Ratio(1, selected.len() as u32))
        .collect();

    let columns = Layout::horizontal(constraints).split(area);

    let window = app.time_window.as_str();
    for (i, coin) in selected.iter().enumerate() {
        render_coin_panel(frame, columns[i], coin, window, &app.theme);
    }
}

fn render_coin_panel(frame: &mut Frame, area: Rect, coin: &CoinData, window: &str, theme: &Theme) {
    let chunks = Layout::vertical([
        Constraint::Length(8),  // Price info
        Constraint::Length(8),  // Indicators
        Constraint::Min(8),     // Chart (increased for proper display)
    ])
    .split(area);

    render_price_info(frame, chunks[0], coin, theme);
    render_indicators(frame, chunks[1], coin, theme);
    render_chart_section(frame, chunks[2], coin, window, theme);
}

fn render_price_info(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    let (change_str, change_color, arrow) = widgets::format_change(coin.change_24h, theme);

    let lines = vec![
        Line::from(vec![
            Span::styled("Price:      ", Style::default().fg(theme.foreground_muted)),
            Span::styled(
                widgets::format_price(coin.price),
                Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("24h Change: ", Style::default().fg(theme.foreground_muted)),
            Span::styled(change_str, Style::default().fg(change_color)),
            Span::raw(" "),
            Span::styled(arrow, Style::default().fg(change_color)),
        ]),
        Line::from(vec![
            Span::styled("24h Volume: ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_volume_full(coin.volume_usd, coin.volume_base, &coin.symbol), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("24h High:   ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(coin.high_24h), Style::default().fg(theme.positive)),
        ]),
        Line::from(vec![
            Span::styled("24h Low:    ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(coin.low_24h), Style::default().fg(theme.negative)),
        ]),
    ];

    let title = format!(" {}/USD ", coin.symbol);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_indicators(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    let ind = &coin.indicators;
    let (rsi6_str, rsi6_color) = widgets::format_rsi(ind.rsi_6, theme);
    let (rsi12_str, rsi12_color) = widgets::format_rsi(ind.rsi_12, theme);
    let (rsi24_str, rsi24_color) = widgets::format_rsi(ind.rsi_24, theme);
    let macd_parts = widgets::format_macd(ind.macd_line, ind.macd_signal, ind.macd_histogram, theme);

    let lines = vec![
        // RSI row - all three on one line
        Line::from(vec![
            Span::styled("RSI: ", Style::default().fg(theme.foreground_muted)),
            Span::styled(format!("6:{}", rsi6_str), Style::default().fg(rsi6_color)),
            Span::raw("  "),
            Span::styled(format!("12:{}", rsi12_str), Style::default().fg(rsi12_color)),
            Span::raw("  "),
            Span::styled(format!("24:{}", rsi24_str), Style::default().fg(rsi24_color)),
        ]),
        // EMA values
        Line::from(vec![
            Span::styled("EMA(7):   ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(ind.ema_7), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("EMA(25):  ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(ind.ema_25), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("EMA(99):  ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(ind.ema_99), Style::default().fg(theme.foreground)),
        ]),
        // MACD
        Line::from(vec![
            Span::styled("MACD:     ", Style::default().fg(theme.foreground_muted)),
            Span::styled(macd_parts[0].0.clone(), Style::default().fg(macd_parts[0].1)),
            Span::raw(" / "),
            Span::styled(macd_parts[1].0.clone(), Style::default().fg(macd_parts[1].1)),
            Span::raw(" / "),
            Span::styled(macd_parts[2].0.clone(), Style::default().fg(macd_parts[2].1)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Indicators ")
        .title_style(Style::default().fg(theme.accent_secondary));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_chart_section(frame: &mut Frame, area: Rect, coin: &CoinData, window: &str, theme: &Theme) {
    let data = coin.chart_data();
    let bounds = coin.price_bounds();
    widgets::render_price_chart(frame, area, &data, bounds, window, theme);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  [Tab]", Style::default().fg(theme.accent_secondary)),
        Span::raw(" Overview  "),
        Span::styled("[q]", Style::default().fg(theme.accent_secondary)),
        Span::raw(" Quit"),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
