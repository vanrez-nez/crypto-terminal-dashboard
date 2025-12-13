//! Details view - price charts and indicators for selected coins

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use crate::app::{App, TimeWindow};
use crate::mock::CoinData;
use crate::widgets::{
    control_footer::build_details_footer, indicator_panel::build_indicator_panel,
    price_panel::build_price_panel, status_header::build_status_header, theme::GlTheme,
    titled_panel::titled_panel,
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

/// Spacing configuration for the details layout.
#[derive(Clone, Copy)]
struct DetailsSpacing {
    outer_padding: f32,
    vertical_gap: f32,
    column_gap: f32,
    footer_gap: f32,
}

impl DetailsSpacing {
    fn new(theme: &GlTheme) -> Self {
        let base = theme.panel_gap;
        Self {
            outer_padding: base,
            vertical_gap: base,
            column_gap: base,
            footer_gap: base * 2.0,
        }
    }

    fn footer_margin(&self) -> f32 {
        (self.footer_gap - self.vertical_gap).max(0.0)
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
    let spacing = DetailsSpacing::new(theme);

    let mut chart_areas = Vec::new();

    // Build coin columns
    let columns: Vec<PanelBuilder> = active_coins
        .iter()
        .enumerate()
        .map(|(chart_idx, (coin_idx, coin))| {
            chart_areas.push(ChartArea::new(*coin_idx));
            build_coin_column(coin, count, app.time_window, chart_idx, theme, &spacing)
        })
        .collect();

    let view = panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
        .gap(spacing.vertical_gap)
        .padding_all(spacing.outer_padding)
        .background(theme.background)
        // Header
        .child(build_status_header(
            app.view,
            &app.provider,
            app.time_window,
            app.chart_type,
            app.connection_status,
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
        .child(build_details_footer(theme).margin(spacing.footer_margin(), 0.0, 0.0, 0.0));

    (view, chart_areas)
}

fn build_coin_column(
    coin: &CoinData,
    _total_columns: usize,
    time_window: TimeWindow,
    chart_idx: usize,
    theme: &GlTheme,
    spacing: &DetailsSpacing,
) -> PanelBuilder {
    let gap = spacing.vertical_gap;
    let symbol = &coin.symbol;

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
        .child(titled_panel("Chart", theme, build_chart_placeholder(chart_idx)).flex_grow(1.0))
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
