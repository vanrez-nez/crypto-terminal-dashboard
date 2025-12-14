//! Candlestick chart widget with RSI overlay, EMA lines, and volume bars

use crate::api::Candle;
use crate::widgets::chart_renderer::{
    calculate_visible_range, ChartBounds, ChartRenderer, PixelRect,
};
use crate::widgets::chart_utils::{
    calculate_price_bounds, calculate_volume_bounds, render_grid, render_volume_bars, ChartLayout,
};
use crate::widgets::indicators::CandleIndicators;
use crate::widgets::theme::GlTheme;

/// Render a complete candlestick chart with overlays
pub fn render_candlestick_chart(
    renderer: &mut ChartRenderer,
    candles: &[Candle],
    scroll_offset: isize,
    visible_candles: usize,
    price_margin: f64,
    rect: PixelRect,
    theme: &GlTheme,
) {
    if candles.is_empty() || rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    // 1. Calculate visible range
    let visible = calculate_visible_range(candles.len(), visible_candles, scroll_offset);

    let visible_slice = &candles[visible.start_idx..visible.end_idx];
    if visible_slice.is_empty() {
        return;
    }

    // 2. Calculate bounds
    let price_bounds = calculate_price_bounds(visible_slice, price_margin);
    let volume_bounds = calculate_volume_bounds(visible_slice);

    // 3. Calculate layout
    let layout = ChartLayout::new(&rect, visible_candles);

    // 4. Calculate candle dimensions (no horizontal gaps)
    let body_width = layout.slot_width * 0.95;
    let wick_width = (body_width * 0.1).max(1.0);

    // 5. Draw grid
    render_grid(renderer, &layout.price_area, 4, 6, theme);

    // 6. Draw volume bars
    render_volume_bars(
        renderer,
        visible_slice,
        &volume_bounds,
        &layout.volume_area,
        layout.slot_width,
        0.4,
        theme,
    );

    // 7. Calculate indicators for overlay
    let indicators = CandleIndicators::from_candles(candles, 14);

    // 8. Draw EMA lines
    render_ema_lines(
        renderer,
        &indicators,
        visible.start_idx,
        visible.end_idx,
        &price_bounds,
        &layout.price_area,
        layout.slot_width,
        theme,
    );

    // 9. Draw candlesticks
    render_candles(
        renderer,
        visible_slice,
        &price_bounds,
        &layout.price_area,
        layout.slot_width,
        body_width,
        wick_width,
        theme,
    );

    // 10. Draw RSI overlay
    render_rsi_overlay(
        renderer,
        &indicators.rsi,
        visible.start_idx,
        visible.end_idx,
        &layout.price_area,
        layout.slot_width,
        theme,
    );
}

/// Render candlesticks
fn render_candles(
    renderer: &mut ChartRenderer,
    candles: &[Candle],
    bounds: &ChartBounds,
    rect: &PixelRect,
    slot_width: f32,
    body_width: f32,
    wick_width: f32,
    theme: &GlTheme,
) {
    for (i, candle) in candles.iter().enumerate() {
        let x = rect.x + (i as f32 + 0.5) * slot_width;

        // Convert OHLC to pixel Y coordinates
        let (_, open_y) = bounds.to_pixel(0.0, candle.open, rect);
        let (_, high_y) = bounds.to_pixel(0.0, candle.high, rect);
        let (_, low_y) = bounds.to_pixel(0.0, candle.low, rect);
        let (_, close_y) = bounds.to_pixel(0.0, candle.close, rect);

        let color = if candle.close >= candle.open {
            theme.candle_bullish
        } else {
            theme.candle_bearish
        };

        renderer.draw_candle(x, open_y, high_y, low_y, close_y, body_width, wick_width, color);
    }
}

/// Render EMA lines as polylines
fn render_ema_lines(
    renderer: &mut ChartRenderer,
    indicators: &CandleIndicators,
    start_idx: usize,
    end_idx: usize,
    bounds: &ChartBounds,
    rect: &PixelRect,
    slot_width: f32,
    theme: &GlTheme,
) {
    let ema_configs = [
        (&indicators.ema_7, theme.indicator_primary, 1.5f32),
        (&indicators.ema_25, theme.indicator_secondary, 1.2f32),
        (&indicators.ema_99, theme.indicator_tertiary, 1.0f32),
    ];

    for (ema_values, color, thickness) in ema_configs {
        let points: Vec<(f32, f32)> = (start_idx..end_idx)
            .filter_map(|i| {
                if i < ema_values.len() && ema_values[i] > 0.0 {
                    let x = rect.x + ((i - start_idx) as f32 + 0.5) * slot_width;
                    let (_, y) = bounds.to_pixel(0.0, ema_values[i], rect);
                    Some((x, y))
                } else {
                    None
                }
            })
            .collect();

        if points.len() >= 2 {
            renderer.draw_polyline(&points, thickness, color);
        }
    }
}

/// Render RSI as an overlay with its own 0-100 Y-axis scale
fn render_rsi_overlay(
    renderer: &mut ChartRenderer,
    rsi_values: &[f64],
    start_idx: usize,
    end_idx: usize,
    rect: &PixelRect,
    slot_width: f32,
    theme: &GlTheme,
) {
    // RSI bounds are always 0-100
    let rsi_bounds = ChartBounds::new(0.0, (end_idx - start_idx) as f64, 0.0, 100.0);

    // Draw RSI reference lines (30 oversold, 70 overbought)
    let mut rsi_color_dim = theme.accent;
    rsi_color_dim[3] = 0.3;

    let (_, y_30) = rsi_bounds.to_pixel(0.0, 30.0, rect);
    let (_, y_70) = rsi_bounds.to_pixel(0.0, 70.0, rect);

    renderer.draw_dashed_line_h(rect.x, y_30, rect.width, 1.0, 5.0, 3.0, rsi_color_dim);
    renderer.draw_dashed_line_h(rect.x, y_70, rect.width, 1.0, 5.0, 3.0, rsi_color_dim);

    // Draw RSI line
    let points: Vec<(f32, f32)> = (start_idx..end_idx)
        .filter_map(|i| {
            if i < rsi_values.len() {
                let x = rect.x + ((i - start_idx) as f32 + 0.5) * slot_width;
                let (_, y) = rsi_bounds.to_pixel(0.0, rsi_values[i], rect);
                Some((x, y))
            } else {
                None
            }
        })
        .collect();

    if points.len() >= 2 {
        renderer.draw_polyline(&points, 1.5, theme.accent);
    }
}
