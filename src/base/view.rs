use crate::widgets::theme::GlTheme;

/// Standard spacing used across all views.
#[derive(Clone, Copy)]
pub struct ViewSpacing {
    pub outer_padding: f32,
    pub section_gap: f32,
    pub footer_gap: f32,
    pub column_gap: f32,
}

impl ViewSpacing {
    pub fn new(theme: &GlTheme) -> Self {
        let base = theme.panel_gap;
        Self {
            outer_padding: base,
            section_gap: base,
            footer_gap: base * 2.0,
            column_gap: base,
        }
    }

    pub fn footer_margin(&self) -> f32 {
        (self.footer_gap - self.section_gap).max(0.0)
    }
}

/// Precomputed dimensions for a view, derived from the theme and spacing.
#[derive(Clone, Copy)]
pub struct ViewMetrics {
    pub inner_width: f32,
    pub header_height: f32,
    pub footer_height: f32,
    pub content_height: f32,
}

impl ViewMetrics {
    pub fn new(width: f32, height: f32, spacing: &ViewSpacing, theme: &GlTheme) -> Self {
        let header_height = header_height(theme);
        let footer_height = footer_height(theme);
        let inner_width = inner_width(width, spacing.outer_padding);

        // Remaining space after padding, header/footer, gaps, and footer margin.
        let content_height = (height
            - spacing.outer_padding * 2.0
            - header_height
            - footer_height
            - spacing.section_gap * 2.0
            - spacing.footer_margin())
        .max(0.0);

        Self {
            inner_width,
            header_height,
            footer_height,
            content_height,
        }
    }
}

/// Standard status header height derived from theme sizing.
pub fn header_height(theme: &GlTheme) -> f32 {
    theme.font_size * 3.0
}

/// Standard footer height shared across views.
pub fn footer_height(theme: &GlTheme) -> f32 {
    theme.font_size * 3.0
}

/// Compute inner width for a panel given total width and horizontal padding.
pub fn inner_width(total_width: f32, padding: f32) -> f32 {
    (total_width - padding * 2.0).max(0.0)
}
