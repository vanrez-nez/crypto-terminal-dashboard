use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
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

    render_header(frame, chunks[0], &app.theme);
    render_content(frame, chunks[1], app);
    render_footer(frame, chunks[2], &app.theme);
}

fn render_header(frame: &mut Frame, area: Rect, theme: &Theme) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  [", Style::default().fg(theme.foreground_inactive)),
        Span::styled("Overview", Style::default().fg(theme.foreground_inactive)),
        Span::styled("]  [Tab: ", Style::default().fg(theme.foreground_inactive)),
        Span::styled("Details", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
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

    for (i, coin) in selected.iter().enumerate() {
        render_coin_panel(frame, columns[i], coin, &app.theme);
    }
}

fn render_coin_panel(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    let chunks = Layout::vertical([
        Constraint::Length(8),  // Price info
        Constraint::Length(8),  // Indicators
        Constraint::Min(3),     // Sparkline
    ])
    .split(area);

    render_price_info(frame, chunks[0], coin, theme);
    render_indicators(frame, chunks[1], coin, theme);
    render_sparkline_section(frame, chunks[2], coin, theme);
}

fn render_price_info(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    let (change_str, change_color, arrow) = widgets::format_change(coin.change_24h, theme);

    let lines = vec![
        Line::from(vec![
            Span::styled("Price:    ", Style::default().fg(theme.foreground_muted)),
            Span::styled(
                widgets::format_price(coin.price),
                Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("24H:      ", Style::default().fg(theme.foreground_muted)),
            Span::styled(change_str, Style::default().fg(change_color)),
            Span::raw(" "),
            Span::styled(arrow, Style::default().fg(change_color)),
        ]),
        Line::from(vec![
            Span::styled("Volume:   ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_volume(coin.volume), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("High:     ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(coin.high_24h), Style::default().fg(theme.positive)),
        ]),
        Line::from(vec![
            Span::styled("Low:      ", Style::default().fg(theme.foreground_muted)),
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
    let (rsi_str, rsi_color) = widgets::format_rsi(ind.rsi, theme);
    let macd_parts = widgets::format_macd(ind.macd_line, ind.macd_signal, ind.macd_histogram, theme);

    let lines = vec![
        Line::from(vec![
            Span::styled("RSI(14):  ", Style::default().fg(theme.foreground_muted)),
            Span::styled(rsi_str, Style::default().fg(rsi_color)),
            Span::styled(" ●", Style::default().fg(rsi_color)),
        ]),
        Line::from(vec![
            Span::styled("EMA(9):   ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(ind.ema_9), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("EMA(21):  ", Style::default().fg(theme.foreground_muted)),
            Span::styled(widgets::format_price(ind.ema_21), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("MACD:     ", Style::default().fg(theme.foreground_muted)),
            Span::styled(macd_parts[0].0.clone(), Style::default().fg(macd_parts[0].1)),
        ]),
        Line::from(vec![
            Span::styled("Signal:   ", Style::default().fg(theme.foreground_muted)),
            Span::styled(macd_parts[1].0.clone(), Style::default().fg(macd_parts[1].1)),
        ]),
        Line::from(vec![
            Span::styled("Hist:     ", Style::default().fg(theme.foreground_muted)),
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

fn render_sparkline_section(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let sparkline = widgets::render_sparkline(&coin.sparkline, inner_width, theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 24h Price ")
        .title_style(Style::default().fg(theme.foreground_muted));

    let paragraph = Paragraph::new(sparkline).block(block);
    frame.render_widget(paragraph, area);
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
