# Phase 4: Chart Integration

## Goal
Wire the ChartRenderer to display actual price data with line charts, candlestick charts, and volume bars.

## Files to Modify
- `src/views/details.rs` - Add chart rendering function
- `src/main.rs` - Call chart rendering after layout

## Implementation Tasks

### Task 4.1: Create chart rendering function
In `src/views/details.rs`:
```rust
use crate::widgets::{
    ChartRenderer, ChartBounds, PixelRect,
    calculate_visible_range, GlTheme,
};
use crate::app::ChartType;
use crate::mock::{CoinData, Candle};
use dashboard_system::{TextRenderer, FontAtlas, glow};

pub fn render_charts(
    gl: &glow::Context,
    chart_renderer: &mut ChartRenderer,
    text_renderer: &mut TextRenderer,
    atlas: &FontAtlas,
    app: &App,
    chart_bounds: &[PixelRect],
    theme: &GlTheme,
    screen_width: u32,
    screen_height: u32,
) {
    let active_coins = app.active_coins();

    chart_renderer.begin();

    for (i, (coin_idx, coin)) in active_coins.iter().enumerate() {
        if let Some(bounds) = chart_bounds.get(i) {
            render_single_chart(
                chart_renderer,
                coin,
                app.chart_type,
                app.candle_scroll_offset,
                *bounds,
                theme,
            );
        }
    }

    chart_renderer.end(gl, screen_width, screen_height);

    // Render price labels (text) after chart primitives
    render_chart_labels(
        gl, text_renderer, atlas, &active_coins, chart_bounds, theme, screen_width, screen_height
    );
}
```

### Task 4.2: Implement single chart rendering
```rust
fn render_single_chart(
    renderer: &mut ChartRenderer,
    coin: &CoinData,
    chart_type: ChartType,
    scroll_offset: isize,
    rect: PixelRect,
    theme: &GlTheme,
) {
    if coin.candles.is_empty() {
        return;
    }

    // Reserve bottom 20% for volume
    let volume_height = rect.height * 0.18;
    let chart_rect = PixelRect::new(
        rect.x,
        rect.y,
        rect.width,
        rect.height - volume_height - 4.0, // 4px gap
    );
    let volume_rect = PixelRect::new(
        rect.x,
        rect.y + chart_rect.height + 4.0,
        rect.width,
        volume_height,
    );

    // Calculate price bounds
    let price_bounds = calculate_price_bounds(&coin.candles, scroll_offset, &chart_rect);

    // Draw grid
    renderer.draw_grid(
        chart_rect.x, chart_rect.y,
        chart_rect.width, chart_rect.height,
        4, 6, // 4 horizontal, 6 vertical lines
        1.0,
        [theme.border[0], theme.border[1], theme.border[2], 0.3],
    );

    match chart_type {
        ChartType::Line => {
            render_line_chart(renderer, coin, scroll_offset, &chart_rect, &price_bounds, theme);
        }
        ChartType::Candlestick => {
            render_candlestick_chart(renderer, coin, scroll_offset, &chart_rect, &price_bounds, theme);
        }
    }

    // Render volume bars
    render_volume_bars(renderer, coin, scroll_offset, &volume_rect, theme);
}
```

### Task 4.3: Implement line chart rendering
```rust
fn render_line_chart(
    renderer: &mut ChartRenderer,
    coin: &CoinData,
    scroll_offset: isize,
    rect: &PixelRect,
    bounds: &ChartBounds,
    theme: &GlTheme,
) {
    let candle_width = 12.0;
    let visible_slots = (rect.width / candle_width) as usize;
    let range = calculate_visible_range(coin.candles.len(), visible_slots, scroll_offset);

    // Build points from close prices
    let points: Vec<(f32, f32)> = coin.candles[range.start_idx..range.end_idx]
        .iter()
        .enumerate()
        .map(|(i, candle)| {
            let slot = i + range.empty_right_slots;
            let x = rect.x + (slot as f32 + 0.5) * candle_width;
            let (_, y) = bounds.to_pixel(0.0, candle.close, rect);
            (x, y)
        })
        .collect();

    if points.len() < 2 {
        return;
    }

    // Draw filled area under line
    let fill_color = [theme.accent[0], theme.accent[1], theme.accent[2], 0.15];
    renderer.draw_filled_area(&points, rect.y + rect.height, fill_color);

    // Draw line
    renderer.draw_polyline(&points, 2.0, theme.accent);

    // Draw current price marker
    if let Some(last) = points.last() {
        renderer.draw_marker(last.0, last.1, 8.0, theme.accent);
    }
}
```

### Task 4.4: Implement candlestick chart rendering
```rust
fn render_candlestick_chart(
    renderer: &mut ChartRenderer,
    coin: &CoinData,
    scroll_offset: isize,
    rect: &PixelRect,
    bounds: &ChartBounds,
    theme: &GlTheme,
) {
    let candle_width = 12.0;
    let body_width = 8.0;
    let wick_width = 2.0;
    let visible_slots = (rect.width / candle_width) as usize;
    let range = calculate_visible_range(coin.candles.len(), visible_slots, scroll_offset);

    for (i, candle) in coin.candles[range.start_idx..range.end_idx].iter().enumerate() {
        let slot = i + range.empty_right_slots;
        let x = rect.x + (slot as f32 + 0.5) * candle_width;

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
```

### Task 4.5: Implement volume bars
```rust
fn render_volume_bars(
    renderer: &mut ChartRenderer,
    coin: &CoinData,
    scroll_offset: isize,
    rect: &PixelRect,
    theme: &GlTheme,
) {
    let candle_width = 12.0;
    let bar_width = 6.0;
    let visible_slots = (rect.width / candle_width) as usize;
    let range = calculate_visible_range(coin.candles.len(), visible_slots, scroll_offset);

    // Find max volume for scaling
    let max_vol = coin.candles[range.start_idx..range.end_idx]
        .iter()
        .map(|c| c.volume)
        .fold(0.0f64, f64::max);

    if max_vol <= 0.0 {
        return;
    }

    for (i, candle) in coin.candles[range.start_idx..range.end_idx].iter().enumerate() {
        let slot = i + range.empty_right_slots;
        let x = rect.x + (slot as f32 + 0.5) * candle_width;
        let height = (candle.volume / max_vol) as f32 * rect.height;

        let color = if candle.close >= candle.open {
            [theme.candle_bullish[0], theme.candle_bullish[1], theme.candle_bullish[2], 0.5]
        } else {
            [theme.candle_bearish[0], theme.candle_bearish[1], theme.candle_bearish[2], 0.5]
        };

        renderer.draw_volume_bar(x, rect.y + rect.height, height, bar_width, color);
    }
}
```

### Task 4.6: Calculate price bounds
```rust
fn calculate_price_bounds(
    candles: &[Candle],
    scroll_offset: isize,
    rect: &PixelRect,
) -> ChartBounds {
    let candle_width = 12.0;
    let visible_slots = (rect.width / candle_width) as usize;
    let range = calculate_visible_range(candles.len(), visible_slots, scroll_offset);

    let visible = &candles[range.start_idx..range.end_idx];

    if visible.is_empty() {
        return ChartBounds::new(0.0, 1.0, 0.0, 1.0);
    }

    let (mut min_price, mut max_price) = (f64::MAX, f64::MIN);
    for c in visible {
        min_price = min_price.min(c.low);
        max_price = max_price.max(c.high);
    }

    // Add 5% padding
    ChartBounds::new(0.0, visible.len() as f64, min_price, max_price)
        .with_padding(0.05)
}
```

### Task 4.7: Render price labels
```rust
fn render_chart_labels(
    gl: &glow::Context,
    text_renderer: &mut TextRenderer,
    atlas: &FontAtlas,
    coins: &[(usize, &CoinData)],
    chart_bounds: &[PixelRect],
    theme: &GlTheme,
    screen_width: u32,
    screen_height: u32,
) {
    text_renderer.begin();

    for (i, (_, coin)) in coins.iter().enumerate() {
        if let Some(rect) = chart_bounds.get(i) {
            // Draw current price label on right edge
            if let Some(last) = coin.candles.last() {
                let price_str = format!("{:.2}", last.close);
                text_renderer.draw_text(
                    atlas,
                    &price_str,
                    rect.x + rect.width - 80.0,
                    rect.y + 20.0,
                    1.0,
                    theme.foreground,
                    HAlign::Right,
                    VAlign::Top,
                    0.0,
                );
            }
        }
    }

    text_renderer.end(gl, screen_width, screen_height);
}
```

### Task 4.8: Wire to main render loop
In `src/main.rs`, after layout render:
```rust
// In run_gl_loop, after render():

if let View::Details = app.view {
    let chart_bounds = calculate_chart_bounds(
        width as f32,
        height as f32,
        view_result.chart_areas.len().max(1),
        50.0,   // header
        36.0,   // footer
        80.0,   // price panel
        120.0,  // indicators
        4.0,    // gap
    );

    render_charts(
        &display.gl,
        &mut chart_renderer,
        &mut text_renderer,
        &atlas,
        &app,
        &chart_bounds,
        &gl_theme,
        width,
        height,
    );
}
```

## Validation
- Line chart draws smooth curve from candle closes
- Filled area appears under line with transparency
- Candlesticks show correct open/high/low/close
- Bullish candles are green, bearish are red
- Volume bars scale correctly and match candle colors
- Scrolling updates visible candles
- Current price marker visible at latest data point

## Notes
- ChartRenderer batches all draws before flush
- Text labels rendered in separate pass after chart geometry
- Scissor clipping may be needed if charts overflow bounds
