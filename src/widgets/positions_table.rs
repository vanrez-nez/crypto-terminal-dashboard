//! Positions table widget for displaying margin positions

use crate::api::margin::MarginPosition;
use crate::base::layout::{HAlign, VAlign};
use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::format::format_price;
use super::theme::GlTheme;

/// Build the positions table widget
pub fn build_positions_table(
    positions: &[MarginPosition],
    selected_index: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    if positions.is_empty() {
        return build_empty_table(theme);
    }

    // Build header row
    let header = build_header_row(theme);

    // Build data rows
    let rows: Vec<PanelBuilder> = positions
        .iter()
        .enumerate()
        .map(|(i, pos)| {
            let is_selected = i == selected_index;
            build_position_row(pos, is_selected, theme)
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
    let row_height = theme.font_size * 2.0;
    let gap = theme.panel_gap;

    panel()
        .width(percent(1.0))
        .height(length(row_height))
        .padding(gap / 2.0, gap, gap / 2.0, gap)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .background(theme.background)
        .child(
            panel()
                .width(length(80.0))
                .text("ASSET", theme.accent_secondary, theme.font_normal)
                .text_align(HAlign::Left, VAlign::Center),
        )
        .child(
            panel()
                .width(length(140.0))
                .text("AMOUNT", theme.accent_secondary, theme.font_normal)
                .text_align(HAlign::Left, VAlign::Center),
        )
        .child(
            panel()
                .width(length(130.0))
                .text("PRICE", theme.accent_secondary, theme.font_normal)
                .text_align(HAlign::Left, VAlign::Center),
        )
        .child(
            panel()
                .width(length(160.0))
                .text("BORROWED VAL", theme.accent_secondary, theme.font_normal)
                .text_align(HAlign::Left, VAlign::Center),
        )
        .child(
            panel()
                .flex_grow(1.0)
                .text("NET VALUE", theme.accent_secondary, theme.font_normal)
                .text_align(HAlign::Left, VAlign::Center),
        )
}

fn build_position_row(pos: &MarginPosition, is_selected: bool, theme: &GlTheme) -> PanelBuilder {
    let row_height = theme.font_size * 2.5;
    let gap = theme.panel_gap;

    let bg_color = if is_selected {
        theme.selection_bg
    } else {
        [0.0, 0.0, 0.0, 0.0] // Transparent
    };

    // Color for borrowed (red if borrowed, normal if not)
    let borrowed_color = if pos.borrowed > 0.0001 {
        theme.negative
    } else {
        theme.foreground
    };

    // Color for net value (green if positive, red if negative)
    let net_color = if pos.net_value_usd > 0.0 {
        theme.positive
    } else if pos.net_value_usd < 0.0 {
        theme.negative
    } else {
        theme.foreground
    };

    panel()
        .width(percent(1.0))
        .height(length(row_height))
        .padding(gap / 2.0, gap, gap / 2.0, gap)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .background(bg_color)
        // Asset
        .child(
            panel()
                .width(length(80.0))
                .text(&pos.asset, theme.foreground, theme.font_normal)
                .text_align(HAlign::Left, VAlign::Center),
        )
        // Amount (total owned = free + locked)
        .child(
            panel()
                .width(length(140.0))
                .text(
                    &format!("{:.4}", pos.free + pos.locked),
                    theme.foreground,
                    theme.font_normal,
                )
                .text_align(HAlign::Left, VAlign::Center),
        )
        // Current Price
        .child(
            panel()
                .width(length(130.0))
                .text(
                    &format_price(pos.current_price),
                    theme.foreground,
                    theme.font_normal,
                )
                .text_align(HAlign::Left, VAlign::Center),
        )
        // Borrowed Value USD
        .child(
            panel()
                .width(length(160.0))
                .text(
                    &format_price(pos.borrowed_value_usd),
                    borrowed_color,
                    theme.font_normal,
                )
                .text_align(HAlign::Left, VAlign::Center),
        )
        // Net Value USD
        .child(
            panel()
                .flex_grow(1.0)
                .text(
                    &format_price(pos.net_value_usd),
                    net_color,
                    theme.font_normal,
                )
                .text_align(HAlign::Left, VAlign::Center),
        )
}

fn build_empty_table(theme: &GlTheme) -> PanelBuilder {
    panel()
        .flex_grow(1.0)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .text(
            "No active positions",
            theme.accent_secondary,
            theme.font_normal,
        )
}
