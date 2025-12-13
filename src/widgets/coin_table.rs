//! Coin table widget for displaying a list of cryptocurrencies with selection

use dashboard_system::{panel, HAlign, PanelBuilder, VAlign, taffy};
use taffy::prelude::*;

use super::format::{format_change, format_price, format_price_short, format_volume_short};
use super::theme::GlTheme;
use crate::mock::CoinData;

/// Build the coin table widget
pub fn build_coin_table(
    coins: &[CoinData],
    selected_index: usize,
    checked: &[bool],
    theme: &GlTheme,
) -> PanelBuilder {
    // Build header row
    let header = build_header_row(theme);

    // Build data rows
    let rows: Vec<PanelBuilder> = coins
        .iter()
        .enumerate()
        .map(|(i, coin)| {
            let is_selected = i == selected_index;
            let is_checked = checked.get(i).copied().unwrap_or(false);
            build_coin_row(coin, is_selected, is_checked, theme)
        })
        .collect();

    panel()
        .width(percent(1.0))
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Column)
        .overflow_scroll()
        .clip(true)
        .child(header)
        .children(rows)
}

fn build_header_row(theme: &GlTheme) -> PanelBuilder {
    panel()
        .width(percent(1.0))
        .height(length(36.0))
        .padding(4.0, 8.0, 4.0, 8.0)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .background(theme.background)
        .child(
            panel()
                .width(length(60.0))
                .text("", theme.accent_secondary, 1.0)
        )
        .child(
            panel()
                .width(length(100.0))
                .text("PAIR", theme.accent_secondary, 1.0)
        )
        .child(
            panel()
                .width(length(140.0))
                .text("PRICE", theme.accent_secondary, 1.0)
        )
        .child(
            panel()
                .width(length(100.0))
                .text("24h %", theme.accent_secondary, 1.0)
        )
        .child(
            panel()
                .width(length(160.0))
                .text("24h VOL", theme.accent_secondary, 1.0)
        )
        .child(
            panel()
                .flex_grow(1.0)
                .text("24h H/L", theme.accent_secondary, 1.0)
        )
}

fn build_coin_row(
    coin: &CoinData,
    is_selected: bool,
    is_checked: bool,
    theme: &GlTheme,
) -> PanelBuilder {
    let checkbox = if is_checked { "[x]" } else { "[ ]" };
    let cursor = if is_selected { ">" } else { " " };
    let checkbox_text = format!("{}{}", cursor, checkbox);

    let pair = format!("{}/USD", coin.symbol);
    let price = format_price(coin.price);
    let change = format_change(coin.change_24h);
    let volume = format_volume_short(coin.volume_usd, coin.volume_base);
    let high_low = format!(
        "{} / {}",
        format_price_short(coin.high_24h),
        format_price_short(coin.low_24h)
    );

    let change_color = if coin.change_24h >= 0.0 {
        theme.positive
    } else {
        theme.negative
    };

    let bg_color = if is_selected {
        theme.selection_bg
    } else {
        [0.0, 0.0, 0.0, 0.0] // Transparent
    };

    panel()
        .width(percent(1.0))
        .height(length(36.0))
        .padding(4.0, 8.0, 4.0, 8.0)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .background(bg_color)
        // Checkbox column
        .child(
            panel()
                .width(length(60.0))
                .text(&checkbox_text, theme.foreground, 1.0)
        )
        // Pair column
        .child(
            panel()
                .width(length(100.0))
                .text(&pair, theme.foreground, 1.0)
        )
        // Price column
        .child(
            panel()
                .width(length(140.0))
                .text(&price, theme.foreground, 1.0)
        )
        // Change column
        .child(
            panel()
                .width(length(100.0))
                .text(&change, change_color, 1.0)
        )
        // Volume column
        .child(
            panel()
                .width(length(160.0))
                .text(&volume, theme.foreground_muted, 1.0)
        )
        // High/Low column
        .child(
            panel()
                .flex_grow(1.0)
                .text(&high_low, theme.foreground_muted, 1.0)
        )
}
