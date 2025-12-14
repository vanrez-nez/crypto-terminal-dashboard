//! Polygonal (area) chart with gradient fill and border line

use crate::api::Candle;
use crate::widgets::chart_renderer::{calculate_visible_range, ChartRenderer, PixelRect};
use crate::widgets::chart_utils::{
    calculate_price_bounds_from_closes, calculate_volume_bounds, render_grid, render_volume_bars,
    ChartLayout,
};
use crate::widgets::theme::GlTheme;

/// Render a polygonal (area) chart with gradient fill, border line, and volume bars
pub fn render_polygonal_chart(
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
    let price_bounds = calculate_price_bounds_from_closes(visible_slice, price_margin);
    let volume_bounds = calculate_volume_bounds(visible_slice);

    // 3. Calculate layout
    let layout = ChartLayout::new(&rect, visible_candles);

    // 4. Draw grid
    render_grid(renderer, &layout.price_area, 4, 6, theme);

    // 5. Draw volume bars
    render_volume_bars(
        renderer,
        visible_slice,
        &volume_bounds,
        &layout.volume_area,
        layout.slot_width,
        0.4,
        theme,
    );

    // 6. Build points from candle closes
    let points: Vec<(f32, f32)> = visible_slice
        .iter()
        .enumerate()
        .map(|(i, candle)| {
            let x = rect.x + (i as f32 + 0.5) * layout.slot_width;
            let (_, y) = price_bounds.to_pixel(0.0, candle.close, &layout.price_area);
            (x, y)
        })
        .collect();

    if points.len() < 2 {
        return;
    }

    // 7. Draw gradient filled area (normalized across chart height)
    let chart_top_y = layout.price_area.y;
    let baseline_y = layout.price_area.y + layout.price_area.height;
    renderer.draw_gradient_filled_area(
        &points,
        baseline_y,
        chart_top_y,
        theme.poly_fill_top,
        theme.poly_fill_bottom,
    );

    // 8. Draw border line on top
    renderer.draw_polyline(&points, 2.0, theme.poly_line);
}
