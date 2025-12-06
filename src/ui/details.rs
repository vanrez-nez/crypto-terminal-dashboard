use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, ChartType, ConnectionStatus};
use crate::mock::CoinData;
use crate::theme::Theme;
use super::widgets::{self, price_change_color, calculate_time_remaining};

pub fn render(frame: &mut Frame, app: &mut App) {
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
        ConnectionStatus::Connecting => ("◌ Connecting", theme.status_connecting),
        ConnectionStatus::Disconnected => ("○ Disconnected", theme.status_disconnected),
        ConnectionStatus::Mock => ("◆ Mock", theme.status_mock),
    };

    let provider_display = capitalize(&app.provider);
    let window_display = app.time_window.as_str();
    let chart_type_display = match app.chart_type {
        ChartType::Line => "Line",
        ChartType::Candlestick => "Candle",
    };
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
        Span::styled("[c] Chart: ", Style::default().fg(theme.foreground_muted)),
        Span::styled(chart_type_display, Style::default().fg(theme.accent)),
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

fn render_content(frame: &mut Frame, area: Rect, app: &mut App) {
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
    let granularity = app.time_window.granularity();
    let scroll_offset = app.candle_scroll_offset;
    let theme = app.theme;
    let chart_type = app.chart_type;

    let mut clamped_offset = None;
    for (i, coin) in selected.iter().enumerate() {
        if let Some(offset) = render_coin_panel(frame, columns[i], coin, window, &theme, chart_type, granularity, scroll_offset) {
            clamped_offset = Some(offset);
        }
    }

    // Sync clamped scroll offset back to app
    if let Some(offset) = clamped_offset {
        app.candle_scroll_offset = offset;
    }
}

fn render_coin_panel(frame: &mut Frame, area: Rect, coin: &CoinData, window: &str, theme: &Theme, chart_type: ChartType, granularity: u32, scroll_offset: isize) -> Option<isize> {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Price box
        Constraint::Length(7),  // Stats info
        Constraint::Length(4),  // Indicators (2 rows + 2 borders)
        Constraint::Min(8),     // Chart
    ])
    .split(area);

    render_price_box(frame, chunks[0], coin, theme);
    render_stats_info(frame, chunks[1], coin, theme);
    render_indicators(frame, chunks[2], coin, theme);
    render_chart_section(frame, chunks[3], coin, window, theme, chart_type, granularity, scroll_offset)
}

fn render_price_box(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    // Calculate price color based on change compared to historical average
    let price_color = price_change_color(coin.price, coin.prev_price, coin.avg_change(), theme);

    let title = format!(" {}/USD ", coin.symbol);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

    // Determine arrow based on price change direction
    let change = coin.price - coin.prev_price;
    let arrow = if change > 0.0 {
        " ▲"
    } else if change < 0.0 {
        " ▼"
    } else {
        ""
    };

    let price_line = Line::from(vec![
        Span::styled(
            format!("  {}", widgets::format_price(coin.price)),
            Style::default().fg(price_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            arrow,
            Style::default().fg(price_color).add_modifier(Modifier::BOLD),
        ),
    ]);

    let paragraph = Paragraph::new(price_line).block(block);
    frame.render_widget(paragraph, area);
}

fn render_stats_info(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    let (change_str, change_color, arrow) = widgets::format_change(coin.change_24h, theme);

    let lines = vec![
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Stats ")
        .title_style(Style::default().fg(theme.accent_secondary));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_indicators(frame: &mut Frame, area: Rect, coin: &CoinData, theme: &Theme) {
    let ind = &coin.indicators;

    let rsi_row = Row::new(vec![
        Cell::from(format!("RSI(6): {:.2}", ind.rsi_6)).style(Style::default().fg(theme.indicator_primary)),
        Cell::from(format!("RSI(12): {:.2}", ind.rsi_12)).style(Style::default().fg(theme.indicator_secondary)),
        Cell::from(format!("RSI(24): {:.2}", ind.rsi_24)).style(Style::default().fg(theme.indicator_tertiary)),
    ]);

    let ema_row = Row::new(vec![
        Cell::from(format!("EMA(7): {}", widgets::format_price(ind.ema_7))).style(Style::default().fg(theme.indicator_primary)),
        Cell::from(format!("EMA(25): {}", widgets::format_price(ind.ema_25))).style(Style::default().fg(theme.indicator_secondary)),
        Cell::from(format!("EMA(99): {}", widgets::format_price(ind.ema_99))).style(Style::default().fg(theme.indicator_tertiary)),
    ]);

    let table = Table::new(
        vec![rsi_row, ema_row],
        [
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Indicators ")
            .title_style(Style::default().fg(theme.accent_secondary)),
    );

    frame.render_widget(table, area);
}

fn render_chart_section(frame: &mut Frame, area: Rect, coin: &CoinData, window: &str, theme: &Theme, chart_type: ChartType, granularity: u32, scroll_offset: isize) -> Option<isize> {
    match chart_type {
        ChartType::Line => {
            let data = coin.chart_data();
            let bounds = coin.price_bounds();
            widgets::render_price_chart(frame, area, &data, bounds, window, theme);
            None
        }
        ChartType::Candlestick => {
            let time_remaining = coin.last_candle_time().map(|t| calculate_time_remaining(t, granularity));
            Some(widgets::render_candlestick_chart(frame, area, &coin.candles, window, theme, time_remaining, scroll_offset))
        }
    }
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
