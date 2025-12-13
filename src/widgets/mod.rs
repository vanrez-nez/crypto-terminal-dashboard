//! OpenGL UI widgets for the crypto dashboard
//!
//! This module contains custom widgets built on top of the dashboard-system framework.

pub mod chart_renderer;
pub mod coin_table;
pub mod control_footer;
pub mod format;
pub mod indicator_panel;
pub mod price_panel;
pub mod status_header;
pub mod theme;
pub mod titled_panel;

// Re-exports
pub use chart_renderer::{
    calculate_visible_range, ChartBounds, ChartRenderer, PixelRect, VisibleRange,
};
pub use coin_table::build_coin_table;
pub use control_footer::{build_control_footer, build_details_footer, build_overview_footer};
pub use format::*;
pub use indicator_panel::build_indicator_panel;
pub use price_panel::build_price_panel;
pub use status_header::build_status_header;
pub use theme::{Color, GlTheme};
pub use titled_panel::{titled_panel, titled_panel_colored};
