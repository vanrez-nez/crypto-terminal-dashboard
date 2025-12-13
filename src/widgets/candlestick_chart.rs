//! Candlestick chart widget with RSI overlay, EMA lines, and volume bars

use crate::api::Candle;
use crate::widgets::chart_renderer::{
    calculate_visible_range, ChartBounds, ChartRenderer, PixelRect,
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

    // 2. Calculate price bounds (auto-scale to visible wicks)
    let price_bounds = calculate_price_bounds(visible_slice, price_margin);

    // 3. Calculate volume bounds
    let volume_bounds = calculate_volume_bounds(visible_slice);

    // 4. Calculate layout areas
    let volume_height_ratio = 0.15;
    let volume_height = rect.height * volume_height_ratio;

    let price_area = PixelRect::new(rect.x, rect.y, rect.width, rect.height - volume_height);

    let volume_area = PixelRect::new(
        rect.x,
        rect.y + rect.height - volume_height,
        rect.width,
        volume_height,
    );

    // 5. Calculate slot dimensions
    let slot_width = rect.width / visible_candles as f32;
    let body_width = slot_width * 0.7;
    let wick_width = (body_width * 0.15).max(1.0);

    // 6. Draw grid
    render_grid(renderer, &price_area, 4, 6, theme);

    // 7. Draw volume bars (semi-transparent, at bottom)
    render_volume_bars(
        renderer,
        visible_slice,
        &volume_bounds,
        &volume_area,
        slot_width,
        visible.empty_right_slots,
        0.4,
        theme,
    );

    // 8. Calculate indicators for overlay
    let indicators = CandleIndicators::from_candles(candles, 14);

    // 9. Draw EMA lines
    render_ema_lines(
        renderer,
        &indicators,
        visible.start_idx,
        visible.end_idx,
        &price_bounds,
        &price_area,
        slot_width,
        theme,
    );

    // 10. Draw candlesticks
    render_candles(
        renderer,
        visible_slice,
        &price_bounds,
        &price_area,
        slot_width,
        body_width,
        wick_width,
        visible.empty_right_slots,
        theme,
    );

    // 11. Draw RSI overlay
    render_rsi_overlay(
        renderer,
        &indicators.rsi,
        visible.start_idx,
        visible.end_idx,
        &price_area,
        slot_width,
        theme,
    );
}

/// Calculate price bounds from visible candles with margin
fn calculate_price_bounds(candles: &[Candle], margin: f64) -> ChartBounds {
    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;

    for candle in candles {
        min_price = min_price.min(candle.low);
        max_price = max_price.max(candle.high);
    }

    // Add margin
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
fn calculate_volume_bounds(candles: &[Candle]) -> ChartBounds {
    let max_volume = candles
        .iter()
        .map(|c| c.volume)
        .fold(0.0f64, |a, b| a.max(b));

    ChartBounds::new(0.0, candles.len() as f64, 0.0, max_volume)
}

/// Render grid lines
fn render_grid(
    renderer: &mut ChartRenderer,
    rect: &PixelRect,
    h_lines: usize,
    v_lines: usize,
    theme: &GlTheme,
) {
    let mut grid_color = theme.border;
    grid_color[3] = 0.3; // Semi-transparent

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

/// Render candlesticks
fn render_candles(
    renderer: &mut ChartRenderer,
    candles: &[Candle],
    bounds: &ChartBounds,
    rect: &PixelRect,
    slot_width: f32,
    body_width: f32,
    wick_width: f32,
    empty_right_slots: usize,
    theme: &GlTheme,
) {
    for (i, candle) in candles.iter().enumerate() {
        // Slot position (centered in slot), accounting for empty slots on right
        let slot_index = i;
        let x = rect.x + (slot_index as f32 + 0.5) * slot_width;

        // Convert OHLC to pixel Y coordinates
        let (_, open_y) = bounds.to_pixel(0.0, candle.open, rect);
        let (_, high_y) = bounds.to_pixel(0.0, candle.high, rect);
        let (_, low_y) = bounds.to_pixel(0.0, candle.low, rect);
        let (_, close_y) = bounds.to_pixel(0.0, candle.close, rect);

        // Determine color
        let color = if candle.close >= candle.open {
            theme.candle_bullish
        } else {
            theme.candle_bearish
        };

        // Draw candle
        renderer.draw_candle(x, open_y, high_y, low_y, close_y, body_width, wick_width, color);
    }

    // Suppress unused warning
    let _ = empty_right_slots;
}

/// Render volume bars at the bottom of the chart
fn render_volume_bars(
    renderer: &mut ChartRenderer,
    candles: &[Candle],
    volume_bounds: &ChartBounds,
    rect: &PixelRect,
    slot_width: f32,
    empty_right_slots: usize,
    opacity: f32,
    theme: &GlTheme,
) {
    let bar_width = slot_width * 0.6;

    for (i, candle) in candles.iter().enumerate() {
        let x = rect.x + (i as f32 + 0.5) * slot_width;

        // Normalize volume to bar height
        let vol_ratio = if volume_bounds.y_max > 0.0 {
            (candle.volume / volume_bounds.y_max).min(1.0) as f32
        } else {
            0.0
        };
        let bar_height = vol_ratio * rect.height;

        // Color based on price direction with reduced opacity
        let mut color = if candle.close >= candle.open {
            theme.candle_bullish
        } else {
            theme.candle_bearish
        };
        color[3] = opacity;

        renderer.draw_volume_bar(x, rect.y + rect.height, bar_height, bar_width, color);
    }

    // Suppress unused warning
    let _ = empty_right_slots;
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
    rsi_color_dim[3] = 0.3; // Semi-transparent

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
