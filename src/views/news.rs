//! News view - cryptocurrency news from NewsData.io

use crate::api::news::{format_relative_time, has_api_keys};
use crate::app::App;
use crate::base::{
    panel,
    view::{ViewMetrics, ViewSpacing},
    PanelBuilder,
};
use crate::widgets::{
    control_footer::build_news_footer,
    status_header::build_status_header,
    text_box::{build_text_box, char_width_px},
    theme::GlTheme,
    titled_panel::titled_panel,
};
use taffy::prelude::*;

/// Build the news view
pub fn build_news_view(app: &App, theme: &GlTheme, width: f32, height: f32) -> PanelBuilder {
    let spacing = ViewSpacing::new(theme);
    let metrics = ViewMetrics::new(width, height, &spacing, theme);


    panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
        .gap(spacing.section_gap)
        .padding_all(spacing.outer_padding)
        .background(theme.background)
        // Header - fixed height
        .child(build_status_header(
            app.view,
            &app.provider,
            app.time_window,
            app.chart_type,
            app.connection_status,
            app.notification_manager.unread_count,
            theme,
        ))
        // Main content: headlines + article content
        .child(
            build_news_content(
                app,
                theme,
                metrics.inner_width,
                metrics.content_height,
                &spacing,
            )
        )
        // Footer - fixed height with extra top margin
        .child(build_news_footer(app.news_loading, theme).margin(
            spacing.footer_margin(),
            0.0,
            0.0,
            0.0,
        ))
}

/// Build the main news content with 30/70 split layout
fn build_news_content(
    app: &App,
    theme: &GlTheme,
    width: f32,
    available_height: f32,
    spacing: &ViewSpacing,
) -> PanelBuilder {
    let gap = spacing.section_gap;


    // Check if API key is configured
    if !has_api_keys() {
        return panel()
            .flex_grow(1.0)
            .flex_direction(FlexDirection::Column)
            .gap(gap)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .child(panel().text(
                "News API key not configured",
                theme.foreground_muted,
                theme.font_normal,
            ))
            .child(panel().text(
                "Set NEWSDATA_API_KEY environment variable",
                theme.foreground_muted,
                theme.font_small,
            ));
    }

    // Show loading state
    if app.news_loading {
        return panel()
            .flex_grow(1.0)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .child(panel().text("Loading news...", theme.foreground_muted, theme.font_normal));
    }

    // Show empty state
    if app.news_articles.is_empty() {
        return panel()
            .flex_grow(1.0)
            .flex_direction(FlexDirection::Column)
            .gap(gap)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .child(panel().text(
                "No news articles",
                theme.foreground_muted,
                theme.font_normal,
            ))
            .child(panel().text(
                "Press [r] to refresh",
                theme.foreground_muted,
                theme.font_small,
            ));
    }

    // Split layout: headlines (30%) and content (70%)
    // Account for the gap between panels
    let available_for_content = available_height - gap;
    let headlines_height = (available_for_content * 0.30).max(80.0);
    let content_height = (available_for_content * 0.70).max(160.0);


    panel()
        .flex_direction(FlexDirection::Column)
        .gap(gap)
        // Headlines panel (30%)
        .child(
            titled_panel(
                "Headlines",
                theme,
                build_headlines_list(app, theme, width, headlines_height),
            )
                .height(length(headlines_height))
                .flex_shrink(0.0),
        )
        // Content panel (70%)
        .child(
            build_content_panel(app, theme)
                .height(length(content_height))
                .flex_shrink(0.0),
        )
}

/// Build the headlines list (titles only, compact)
fn build_headlines_list(app: &App, theme: &GlTheme, width: f32, available_height: f32) -> PanelBuilder {
    let gap = theme.panel_gap;

    // Calculate max characters for headline truncation
    // Account for panel chrome: border, padding, titled_panel overhead
    let usable_width = width - theme.panel_padding * 4.0 - 4.0;
    let char_width = char_width_px('M', theme.font_size * theme.font_small).max(1.0);
    let max_chars = ((usable_width / char_width).floor() as usize).clamp(5, 150);

    let total = app.news_articles.len();
    let mut container = panel()
        .flex_direction(FlexDirection::Column)
        .gap(2.0)
        .flex_grow(1.0)
        .clip(true); // Fill the available panel height and clip overflow

    // Estimate how many headlines fit in the allotted height
    let line_height = theme.font_size * theme.font_small * 1.6;
    let per_row = line_height + gap;
    let visible_count = ((available_height / per_row).floor() as usize)
        .clamp(4, total.max(1));
    let selected = app.news_selected;

    // Compute scroll offset to keep selection visible
    let scroll_offset = if selected < visible_count / 2 {
        0
    } else if selected > total.saturating_sub(visible_count / 2) {
        total.saturating_sub(visible_count)
    } else {
        selected.saturating_sub(visible_count / 2)
    };

    let end = (scroll_offset + visible_count).min(total);

    for (idx, article) in app
        .news_articles
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(end - scroll_offset)
    {
        let is_selected = idx == app.news_selected;

        let bg_color = if is_selected {
            theme.selection_bg
        } else {
            theme.background_panel
        };

        let text_color = if is_selected {
            theme.foreground
        } else {
            theme.foreground_muted
        };

        let title = truncate_text(&article.title, max_chars);

        container = container.child(
            panel()
                .padding(2.0, gap / 2.0, 2.0, gap / 2.0)
                .background(bg_color)
                .child(panel().text(&title, text_color, theme.font_small)),
        );
    }

    // Scroll indicator
    if total > visible_count {
        container = container.child(panel().text(
            &format!("{}/{}", selected + 1, total),
            theme.foreground_muted,
            theme.font_small,
        ));
    }

    container
}

/// Build the content panel with article body (70% of space)
fn build_content_panel(
    app: &App,
    theme: &GlTheme,
) -> PanelBuilder {
    let gap = theme.panel_gap;

    // Get selected article
    let article = match app.news_articles.get(app.news_selected) {
        Some(a) => a,
        None => {
            return panel()
                .background(theme.background_panel)
                .border_solid(1.0, theme.border)
                .padding_all(gap * 2.0)
                .child(panel().text(
                    "No article selected",
                    theme.foreground_muted,
                    theme.font_normal,
                ));
        }
    };

    let time_str = format_relative_time(article.published_at);

    // Use description if available, otherwise show a placeholder
    let content = if article.description.is_empty() {
        "No description available for this article."
    } else {
        &article.description
    };

    panel()
        .background(theme.background_panel)
        .border_solid(1.0, theme.border)
        .padding_all(gap)
        .flex_direction(FlexDirection::Column)
        .gap(gap / 2.0)
        // Badges row: source and time
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap)
                .child(
                    panel()
                        .background(theme.accent_secondary)
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .child(panel().text(&article.source, theme.background, theme.font_small)),
                )
                .child(
                    panel()
                        .background(theme.background)
                        .border_solid(1.0, theme.border)
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .child(panel().text(&time_str, theme.foreground_muted, theme.font_small)),
                ),
        )
        // Article body with text reflow and scrolling
        .child(
            build_text_box(
                content,
                app.news_content_scroll,
                theme,
            )
            .flex_grow(1.0),
        )
}

/// Truncate text to fit within max characters
fn truncate_text(text: &str, max_chars: usize) -> String {
    if max_chars < 4 {
        return text.chars().take(max_chars).collect();
    }
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    }
}
