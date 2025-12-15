//! Details view - price charts and indicators for selected coins

use std::time::{SystemTime, UNIX_EPOCH};

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use crate::app::{App, ChartType, TimeWindow};
use crate::base::view::ViewSpacing;
use crate::mock::CoinData;
use crate::widgets::{
    control_footer::build_details_footer,
    indicator_panel::build_indicator_panel,
    price_panel::build_price_panel,
    status_header::build_status_header,
    theme::GlTheme,
    titled_panel::{titled_panel, titled_panel_with_badge},
};

/// Prefix for chart panel marker IDs
pub const CHART_PANEL_PREFIX: &str = "chart_";

/// Represents a chart area that needs to be rendered separately
#[derive(Clone, Debug)]
pub struct ChartArea {
    pub coin_index: usize,
}

impl ChartArea {
    /// Create a new ChartArea
    pub fn new(coin_index: usize) -> Self {
        Self { coin_index }
    }
}

pub fn build_details_view(
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> (PanelBuilder, Vec<ChartArea>) {
    // Use active_coins which falls back to highlighted coin if none selected
    let active_coins = app.active_coins();
    let count = active_coins.len();
    let spacing = ViewSpacing::new(theme);

    let mut chart_areas = Vec::new();

    // Build coin columns
    let columns: Vec<PanelBuilder> = active_coins
        .iter()
        .enumerate()
        .map(|(chart_idx, (coin_idx, coin))| {
            chart_areas.push(ChartArea::new(*coin_idx));
            build_coin_column(
                coin,
                count,
                app.time_window,
                app.chart_type,
                chart_idx,
                theme,
                &spacing,
            )
        })
        .collect();

    let view =
        panel()
            .width(length(width))
            .height(length(height))
            .flex_direction(FlexDirection::Column)
            .gap(spacing.section_gap)
            .padding_all(spacing.outer_padding)
            .background(theme.background)
            // Header
            .child(build_status_header(
                app.view,
                &app.provider,
                app.time_window,
                app.chart_type,
                app.connection_status,
                app.notification_manager.unread_count,
                theme,
            ))
            // Coin columns (horizontal layout)
            .child(
                panel()
                    .flex_grow(1.0)
                    .flex_direction(FlexDirection::Row)
                    .gap(spacing.column_gap)
                    .children(columns),
            )
            // Footer
            .child(
                build_details_footer(app.time_window, app.chart_type, app.ticker_muted, theme)
                    .margin(spacing.footer_margin(), 0.0, 0.0, 0.0),
            );

    (view, chart_areas)
}

fn build_coin_column(
    coin: &CoinData,
    _total_columns: usize,
    time_window: TimeWindow,
    chart_type: ChartType,
    chart_idx: usize,
    theme: &GlTheme,
    spacing: &ViewSpacing,
) -> PanelBuilder {
    let gap = spacing.section_gap;
    let symbol = &coin.symbol;

    // Build chart panel with countdown badge for candlestick mode
    let chart_panel = match chart_type {
        ChartType::Candlestick => {
            let countdown = candle_countdown(time_window.granularity() as u64);
            titled_panel_with_badge(
                "Chart",
                Some((&countdown, theme.accent_secondary)),
                theme,
                build_chart_placeholder(chart_idx),
            )
        }
        ChartType::Polygonal => titled_panel("Chart", theme, build_chart_placeholder(chart_idx)),
    };

    panel()
        .flex_basis(length(0.0)) // Force equal width distribution
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Column)
        .gap(gap)
        // Price panel with title
        .child(titled_panel(
            &format!("{}/USD ({})", symbol, time_window.as_str()),
            theme,
            build_price_panel(coin, time_window, theme),
        ))
        // Chart area (grows to fill, placeholder for ChartRenderer)
        .child(chart_panel.flex_grow(1.0))
        // Indicator panel with title
        .child(titled_panel(
            "Indicators",
            theme,
            build_indicator_panel(&coin.indicators, theme),
        ))
}

fn build_chart_placeholder(chart_idx: usize) -> PanelBuilder {
    // This panel reserves space for chart rendering
    // The actual chart is drawn by ChartRenderer after layout
    // Marker ID is used to find this panel after layout and get its bounds
    panel()
        .flex_grow(1.0)
        .marker_id(format!("{}{}", CHART_PANEL_PREFIX, chart_idx))
}

/// Calculate time remaining until current candle closes
fn candle_countdown(granularity_secs: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Calculate seconds until next candle boundary
    let elapsed_in_candle = now % granularity_secs;
    let remaining = granularity_secs - elapsed_in_candle;

    // Format as HH:MM:SS or MM:SS depending on duration
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}
