//! Price panel widget displaying current price, 24h change, and range bar
//! Layout: 3 columns inline - [Price▲] | [CHANGE: +X%] | [L:xxx ▼ H:xxx]

use crate::app::TimeWindow;
use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::format::{format_change, format_price, format_price_short, price_change_color};
use super::theme::GlTheme;
use crate::mock::CoinData;

/// Build the price panel - 3 inline columns: Price+Arrow, Change, High/Low
pub fn build_price_panel(coin: &CoinData, time_window: TimeWindow, theme: &GlTheme) -> PanelBuilder {
    let price_text = format_price(coin.price);
    let gap = theme.panel_gap;

    // For 1d window, use Binance's actual 24h values (rolling, accurate)
    // For other windows, calculate from candles
    let (change_pct, high, low) = match time_window {
        TimeWindow::Day1 => (coin.change_24h, coin.high_24h, coin.low_24h),
        _ => {
            let change = coin.candle_change();
            let (h, l) = coin.candle_high_low();
            (change, h, l)
        }
    };

    let change_text = format_change(change_pct);

    // Price color based on tick direction
    let avg_change = coin.avg_change();
    let price_color = price_change_color(coin.price, coin.prev_price, avg_change, theme);

    // Arrow for price direction (always show placeholder for stable width)
    let price_delta = coin.price - coin.prev_price;
    let (arrow, arrow_color) = if price_delta > f64::EPSILON {
        ("▲", price_color)
    } else if price_delta < -f64::EPSILON {
        ("▼", price_color)
    } else {
        (" ", theme.background) // Invisible placeholder
    };

    let change_color = if change_pct > 0.0 {
        theme.positive
    } else if change_pct < 0.0 {
        theme.negative
    } else {
        theme.foreground_muted
    };

    // Range bar position (0.0 to 1.0)
    let range = high - low;
    let range_pos = if range > 0.0 {
        ((coin.price - low) / range).clamp(0.0, 1.0)
    } else {
        0.5
    };

    let low_text = format!("L:{}", format_price_short(low));
    let high_text = format!("H:{}", format_price_short(high));

    // Single row with 3 columns
    panel()
        .width(percent(1.0))
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(gap * 2.0)
        // Column 1: Price + arrow (always show arrow for stable width)
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .gap(gap / 2.0)
                .child(panel().text(&price_text, price_color, theme.font_big))
                .child(panel().text(arrow, arrow_color, theme.font_medium)),
        )
        // Column 2: Change
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .gap(gap / 2.0)
                .child(panel().text("CHANGE:", theme.foreground_muted, theme.font_medium))
                .child(panel().text(&change_text, change_color, theme.font_medium)),
        )
        // Column 3: High/Low bar (grows to fill)
        .child(
            panel()
                .flex_grow(1.0)
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .gap(gap)
                .child(panel().text(&low_text, theme.foreground_muted, theme.font_medium))
                .child(build_range_indicator(range_pos, theme))
                .child(panel().text(&high_text, theme.foreground_muted, theme.font_medium)),
        )
}

/// Build range indicator with dim bar and triangle marker
fn build_range_indicator(position: f64, theme: &GlTheme) -> PanelBuilder {
    let left_pct = (position * 100.0) as f32;
    let bar_height = 4.0;

    panel()
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        // Triangle indicator positioned by left margin
        .child(
            panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                // Push triangle up by bar height to align with bar
                .margin(0.0, 0.0, -bar_height, 0.0)
                .child(
                    panel()
                        .width(percent(left_pct / 100.0))
                        .height(length(0.0)),
                )
                .child(panel().text("▼", theme.foreground, theme.font_small)),
        )
        // Dim background bar
        .child(
            panel()
                .width(percent(1.0))
                .height(length(bar_height))
                .background(theme.border),
        )
}
