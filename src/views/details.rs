//! Details view - price charts and indicators for selected coins

use dashboard_system::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use crate::app::{App, TimeWindow};
use crate::mock::CoinData;
use crate::widgets::{
    build_details_footer, build_indicator_panel, build_price_panel, build_status_header,
    titled_panel, GlTheme, PixelRect,
};

/// Represents a chart area that needs to be rendered separately
#[derive(Clone, Debug)]
pub struct ChartArea {
    pub coin_index: usize,
    pub bounds: PixelRect,
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
        .map(|(idx, coin)| {
            chart_areas.push(ChartArea {
                coin_index: *idx,
                bounds: PixelRect::new(0.0, 0.0, 0.0, 0.0), // Filled after layout
            });
            build_coin_column(coin, count, app.time_window, theme, &spacing)
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
    total_columns: usize,
    time_window: TimeWindow,
    theme: &GlTheme,
    spacing: &DetailsSpacing,
) -> PanelBuilder {
    let gap = spacing.vertical_gap;
    let symbol = &coin.symbol;

    panel()
        .flex_grow(1.0 / total_columns as f32)
        .flex_direction(FlexDirection::Column)
        .gap(gap)
        // Price panel with title
        .child(titled_panel(
            &format!("{}/USD ({})", symbol, time_window.as_str()),
            theme,
            build_price_panel(coin, theme),
        ))
        // Chart area (grows to fill, placeholder for ChartRenderer)
        .child(titled_panel("Chart", theme, build_chart_placeholder(theme)).flex_grow(1.0))
        // Indicator panel with title
        .child(titled_panel(
            "Indicators",
            theme,
            build_indicator_panel(&coin.indicators, theme),
        ))
}

fn build_chart_placeholder(theme: &GlTheme) -> PanelBuilder {
    // This panel reserves space for chart rendering
    // The actual chart is drawn by ChartRenderer after layout
    panel().flex_grow(1.0)
}

/// Calculate chart bounds from known layout constants
pub fn calculate_chart_bounds(
    width: f32,
    height: f32,
    num_charts: usize,
    header_height: f32,
    footer_height: f32,
    price_panel_height: f32,
    indicator_height: f32,
    gap: f32,
) -> Vec<PixelRect> {
    if num_charts == 0 {
        return vec![];
    }

    let content_height = height - header_height - footer_height;
    let chart_height = content_height - price_panel_height - indicator_height - gap * 2.0;
    let chart_width = (width - gap * (num_charts - 1) as f32) / num_charts as f32;
    let chart_y = header_height + price_panel_height + gap;

    (0..num_charts)
        .map(|i| {
            let x = i as f32 * (chart_width + gap);
            PixelRect::new(x, chart_y, chart_width, chart_height)
        })
        .collect()
}
