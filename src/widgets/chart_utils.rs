//! Shared chart utilities for candlestick and polygonal charts

use crate::api::Candle;
use crate::widgets::chart_renderer::{ChartBounds, ChartRenderer, PixelRect};
use crate::widgets::theme::GlTheme;

/// Common chart layout areas
pub struct ChartLayout {
    pub price_area: PixelRect,
    pub volume_area: PixelRect,
    pub slot_width: f32,
}

impl ChartLayout {
    /// Create layout with volume section at bottom (15% height)
    pub fn new(rect: &PixelRect, visible_candles: usize) -> Self {
        let volume_height_ratio = 0.15;
        let volume_height = rect.height * volume_height_ratio;

        Self {
            price_area: PixelRect::new(rect.x, rect.y, rect.width, rect.height - volume_height),
            volume_area: PixelRect::new(
                rect.x,
                rect.y + rect.height - volume_height,
                rect.width,
                volume_height,
            ),
            slot_width: rect.width / visible_candles as f32,
        }
    }
}

/// Calculate price bounds from visible candles (high/low) with margin
pub fn calculate_price_bounds(candles: &[Candle], margin: f64) -> ChartBounds {
    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;

    for candle in candles {
        min_price = min_price.min(candle.low);
        max_price = max_price.max(candle.high);
    }

    let range = max_price - min_price;
    let margin_amount = range * margin;

    ChartBounds::new(
        0.0,
        candles.len() as f64,
        min_price - margin_amount,
        max_price + margin_amount,
    )
}

/// Calculate price bounds from candle closes only (for polygonal chart)
pub fn calculate_price_bounds_from_closes(candles: &[Candle], margin: f64) -> ChartBounds {
    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;

    for candle in candles {
        min_price = min_price.min(candle.close);
        max_price = max_price.max(candle.close);
    }

    let range = max_price - min_price;
    let margin_amount = range * margin;

    ChartBounds::new(
        0.0,
        candles.len() as f64,
        min_price - margin_amount,
        max_price + margin_amount,
    )
}

/// Calculate volume bounds from visible candles
pub fn calculate_volume_bounds(candles: &[Candle]) -> ChartBounds {
    let max_volume = candles
        .iter()
        .map(|c| c.volume)
        .fold(0.0f64, |a, b| a.max(b));

    ChartBounds::new(0.0, candles.len() as f64, 0.0, max_volume)
}

/// Render grid lines
pub fn render_grid(
    renderer: &mut ChartRenderer,
    rect: &PixelRect,
    h_lines: usize,
    v_lines: usize,
    theme: &GlTheme,
) {
    let mut grid_color = theme.border;
    grid_color[3] = 0.3;

    renderer.draw_grid(
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        h_lines,
        v_lines,
        1.0,
        grid_color,
    );
}

/// Render volume bars at the bottom of the chart
pub fn render_volume_bars(
    renderer: &mut ChartRenderer,
    candles: &[Candle],
    volume_bounds: &ChartBounds,
    rect: &PixelRect,
    slot_width: f32,
    opacity: f32,
    theme: &GlTheme,
) {
    let bar_width = slot_width * 0.6;

    for (i, candle) in candles.iter().enumerate() {
        let x = rect.x + (i as f32 + 0.5) * slot_width;

        let vol_ratio = if volume_bounds.y_max > 0.0 {
            (candle.volume / volume_bounds.y_max).min(1.0) as f32
        } else {
            0.0
        };
        let bar_height = vol_ratio * rect.height;

        let mut color = if candle.close >= candle.open {
            theme.candle_bullish
        } else {
            theme.candle_bearish
        };
        color[3] = opacity;

        renderer.draw_volume_bar(x, rect.y + rect.height, bar_height, bar_width, color);
    }
}
