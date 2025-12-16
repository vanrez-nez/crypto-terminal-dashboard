//! Reusable table widget with auto-width and fixed-width column support
//!
//! This module provides a flexible table widget that supports:
//! - Fixed-width columns
//! - Auto-width columns (calculated from content)
//! - Flexible columns (fill remaining space)
//! - Hybrid cell content (text or custom PanelBuilder)
//! - Scrollable content with fixed header
//! - Optional per-row styling
//!
//! # Examples
//!
//! ## Basic table with estimated widths
//!
//! ```rust
//! use crate::widgets::table::*;
//!
//! let columns = vec![
//!     ColumnConfig::auto("Name", 0.0),
//!     ColumnConfig::fixed("Status", 80.0),
//!     ColumnConfig::flex("Description", 1.0),
//! ];
//!
//! let rows = vec![
//!     vec![
//!         CellBuilder::text("Alice", theme.foreground),
//!         CellBuilder::text("Active", theme.positive),
//!         CellBuilder::text("Administrator", theme.foreground_muted),
//!     ],
//! ];
//!
//! let columns = estimate_column_widths(&columns, &rows, theme.font_size, theme.font_normal, theme.panel_gap);
//! let table = build_table(&columns, &rows, theme);
//! ```

use crate::base::font_atlas::FontAtlas;
use crate::base::layout::{HAlign, VAlign};
use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::theme::GlTheme;

/// Column width specification
#[derive(Clone, Debug)]
pub enum ColumnWidth {
    /// Fixed width in pixels
    Fixed(f32),
    /// Auto-width based on content (width in pixels, pre-calculated)
    Auto(f32),
    /// Flexible - uses flex_grow to fill remaining space
    Flex(f32),
}

/// Column configuration
#[derive(Clone, Debug)]
pub struct ColumnConfig {
    /// Column header text
    pub header: String,
    /// Width specification
    pub width: ColumnWidth,
    /// Horizontal alignment for cells
    pub align: HAlign,
}

impl ColumnConfig {
    /// Create a fixed-width column
    ///
    /// # Arguments
    /// * `header` - Column header text
    /// * `width` - Fixed width in pixels
    pub fn fixed(header: impl Into<String>, width: f32) -> Self {
        Self {
            header: header.into(),
            width: ColumnWidth::Fixed(width),
            align: HAlign::Left,
        }
    }

    /// Create an auto-width column (width will be calculated)
    ///
    /// # Arguments
    /// * `header` - Column header text
    /// * `width` - Initial width (typically 0.0, will be calculated)
    pub fn auto(header: impl Into<String>, width: f32) -> Self {
        Self {
            header: header.into(),
            width: ColumnWidth::Auto(width),
            align: HAlign::Left,
        }
    }

    /// Create a flexible column that grows to fill space
    ///
    /// # Arguments
    /// * `header` - Column header text
    /// * `grow` - Flex-grow value (typically 1.0)
    pub fn flex(header: impl Into<String>, grow: f32) -> Self {
        Self {
            header: header.into(),
            width: ColumnWidth::Flex(grow),
            align: HAlign::Left,
        }
    }

    /// Set horizontal alignment for this column
    pub fn with_align(mut self, align: HAlign) -> Self {
        self.align = align;
        self
    }
}

/// Cell content - supports both strings and custom panels
#[derive(Clone, Debug)]
pub enum CellContent {
    /// Simple text with color
    Text { text: String, color: [f32; 4] },
    /// Custom panel builder (for complex content)
    /// Function pointer that takes theme and returns a PanelBuilder
    Panel(fn(&GlTheme) -> PanelBuilder),
}

/// Helper for creating cell content
pub struct CellBuilder;

impl CellBuilder {
    /// Create a text cell
    ///
    /// # Arguments
    /// * `text` - Cell text content
    /// * `color` - Text color as RGBA
    pub fn text(text: impl Into<String>, color: [f32; 4]) -> CellContent {
        CellContent::Text {
            text: text.into(),
            color,
        }
    }

    /// Create a custom panel cell
    ///
    /// # Arguments
    /// * `builder` - Function that builds the panel content
    pub fn panel(builder: fn(&GlTheme) -> PanelBuilder) -> CellContent {
        CellContent::Panel(builder)
    }
}

/// A single row of cell data
pub type TableRow = Vec<CellContent>;

/// Row styling configuration
#[derive(Clone, Debug, Default)]
pub struct RowStyle {
    /// Optional background color for the row
    pub background: Option<[f32; 4]>,
    /// Optional custom height for the row
    pub height: Option<f32>,
}

/// Calculate auto-widths for columns based on actual text measurement
///
/// This function should be called where FontAtlas is available.
/// It returns updated ColumnConfigs with Auto widths properly calculated.
///
/// # Arguments
/// * `columns` - Column configurations (Auto widths will be calculated)
/// * `rows` - All table rows (to measure cell content)
/// * `font_atlas` - FontAtlas for text measurement
/// * `font_scale` - Font scale to use (e.g., theme.font_normal)
/// * `padding` - Extra padding per cell (horizontal)
///
/// # Returns
/// Updated column configurations with calculated Auto widths
pub fn calculate_column_widths(
    columns: &[ColumnConfig],
    rows: &[TableRow],
    font_atlas: &FontAtlas,
    font_scale: f32,
    padding: f32,
) -> Vec<ColumnConfig> {
    columns
        .iter()
        .enumerate()
        .map(|(col_idx, col)| {
            let mut config = col.clone();

            // Only calculate for Auto columns
            if matches!(col.width, ColumnWidth::Auto(_)) {
                let mut max_width = 0.0f32;

                // Measure header
                let (header_w, _) = font_atlas.measure_text(&col.header, font_scale);
                max_width = max_width.max(header_w);

                // Measure all cells in this column
                for row in rows {
                    if let Some(cell) = row.get(col_idx) {
                        if let CellContent::Text { text, .. } = cell {
                            let (w, _) = font_atlas.measure_text(text, font_scale);
                            max_width = max_width.max(w);
                        }
                    }
                }

                // Add padding
                config.width = ColumnWidth::Auto(max_width + padding * 2.0);
            }

            config
        })
        .collect()
}

/// Estimate column widths using heuristics (no FontAtlas needed)
///
/// This is a fallback when FontAtlas isn't available. Less accurate but simpler.
/// Uses average character width of 0.6 * font_size * font_scale
///
/// # Arguments
/// * `columns` - Column configurations (Auto widths will be estimated)
/// * `rows` - All table rows (to find longest text)
/// * `font_size` - Base font size (e.g., theme.font_size)
/// * `font_scale` - Font scale to use (e.g., theme.font_normal)
/// * `padding` - Extra padding per cell (horizontal)
///
/// # Returns
/// Updated column configurations with estimated Auto widths
pub fn estimate_column_widths(
    columns: &[ColumnConfig],
    rows: &[TableRow],
    font_size: f32,
    font_scale: f32,
    padding: f32,
) -> Vec<ColumnConfig> {
    let avg_char_width = font_size * font_scale * 0.6;

    columns
        .iter()
        .enumerate()
        .map(|(col_idx, col)| {
            let mut config = col.clone();

            if matches!(col.width, ColumnWidth::Auto(_)) {
                let mut max_chars = col.header.len();

                // Find longest text in this column
                for row in rows {
                    if let Some(CellContent::Text { text, .. }) = row.get(col_idx) {
                        max_chars = max_chars.max(text.len());
                    }
                }

                let width = (max_chars as f32) * avg_char_width + padding * 2.0;
                config.width = ColumnWidth::Auto(width);
            }

            config
        })
        .collect()
}

/// Build a reusable table widget
///
/// # Arguments
/// * `columns` - Column configurations (with calculated widths)
/// * `rows` - Row data
/// * `theme` - Theme for styling
///
/// # Returns
/// A PanelBuilder containing the complete table (header + scrollable body)
pub fn build_table(
    columns: &[ColumnConfig],
    rows: &[TableRow],
    theme: &GlTheme,
) -> PanelBuilder {
    if rows.is_empty() {
        return build_empty_table(theme);
    }

    // Build header
    let header = build_table_header(columns, theme);

    // Build rows
    let row_panels: Vec<PanelBuilder> = rows
        .iter()
        .map(|row| build_table_row(row, columns, theme))
        .collect();

    panel()
        .width(percent(1.0))
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Column)
        .overflow_scroll()
        .clip(true)
        .child(header)
        .children(row_panels)
}

fn build_table_header(columns: &[ColumnConfig], theme: &GlTheme) -> PanelBuilder {
    let row_height = theme.font_size * 2.0;
    let gap = theme.panel_gap;

    let mut row = panel()
        .width(percent(1.0))
        .height(length(row_height))
        .padding(gap / 2.0, gap, gap / 2.0, gap)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .background(theme.background);

    for col in columns {
        let mut cell = panel()
            .text(&col.header, theme.accent_secondary, theme.font_normal)
            .text_align(col.align, VAlign::Center);

        // Apply width based on column type
        cell = match col.width {
            ColumnWidth::Fixed(w) | ColumnWidth::Auto(w) => cell.width(length(w)),
            // Use proportion() for Flex to ensure alignment across rows
            // proportion() sets flex_basis=0, making layout ignore content size
            ColumnWidth::Flex(grow) => cell.proportion(grow),
        };

        row = row.child(cell);
    }

    row
}

fn build_table_row(row: &TableRow, columns: &[ColumnConfig], theme: &GlTheme) -> PanelBuilder {
    let row_height = theme.font_size * 2.5;
    let gap = theme.panel_gap;

    let mut row_panel = panel()
        .width(percent(1.0))
        .height(length(row_height))
        .padding(gap / 2.0, gap, gap / 2.0, gap)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center);

    for (idx, col) in columns.iter().enumerate() {
        let cell_content = row.get(idx);

        let mut cell = match cell_content {
            Some(CellContent::Text { text, color }) => panel()
                .text(text, *color, theme.font_normal)
                .text_align(col.align, VAlign::Center),
            Some(CellContent::Panel(builder_fn)) => {
                // Call the builder function to create custom panel
                builder_fn(theme)
            }
            None => panel(), // Empty cell
        };

        // Apply width
        cell = match col.width {
            ColumnWidth::Fixed(w) | ColumnWidth::Auto(w) => cell.width(length(w)),
            // Use proportion() for Flex to ensure alignment across rows
            ColumnWidth::Flex(grow) => cell.proportion(grow),
        };

        row_panel = row_panel.child(cell);
    }

    row_panel
}

fn build_empty_table(theme: &GlTheme) -> PanelBuilder {
    panel()
        .flex_grow(1.0)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .text("No data", theme.accent_secondary, theme.font_normal)
}

/// Build a table with per-row styling
///
/// # Arguments
/// * `columns` - Column configurations (with calculated widths)
/// * `rows` - Row data
/// * `row_styles` - Per-row styling configuration
/// * `theme` - Theme for styling
///
/// # Returns
/// A PanelBuilder containing the complete table with custom row styles
pub fn build_table_styled(
    columns: &[ColumnConfig],
    rows: &[TableRow],
    row_styles: &[RowStyle],
    theme: &GlTheme,
) -> PanelBuilder {
    if rows.is_empty() {
        return build_empty_table(theme);
    }

    let header = build_table_header(columns, theme);

    let row_panels: Vec<PanelBuilder> = rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let style = row_styles.get(idx).cloned().unwrap_or_default();
            build_table_row_styled(row, columns, &style, theme)
        })
        .collect();

    panel()
        .width(percent(1.0))
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Column)
        .overflow_scroll()
        .clip(true)
        .child(header)
        .children(row_panels)
}

fn build_table_row_styled(
    row: &TableRow,
    columns: &[ColumnConfig],
    style: &RowStyle,
    theme: &GlTheme,
) -> PanelBuilder {
    let row_height = style.height.unwrap_or(theme.font_size * 2.5);
    let gap = theme.panel_gap;

    let mut row_panel = panel()
        .width(percent(1.0))
        .height(length(row_height))
        .padding(gap / 2.0, gap, gap / 2.0, gap)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center);

    if let Some(bg) = style.background {
        row_panel = row_panel.background(bg);
    }

    for (idx, col) in columns.iter().enumerate() {
        let cell_content = row.get(idx);

        let mut cell = match cell_content {
            Some(CellContent::Text { text, color }) => panel()
                .text(text, *color, theme.font_normal)
                .text_align(col.align, VAlign::Center),
            Some(CellContent::Panel(builder_fn)) => builder_fn(theme),
            None => panel(),
        };

        cell = match col.width {
            ColumnWidth::Fixed(w) | ColumnWidth::Auto(w) => cell.width(length(w)),
            // Use proportion() for Flex to ensure alignment across rows
            ColumnWidth::Flex(grow) => cell.proportion(grow),
        };

        row_panel = row_panel.child(cell);
    }

    row_panel
}
