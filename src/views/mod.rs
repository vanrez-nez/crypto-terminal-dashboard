//! OpenGL view compositions for the crypto dashboard

pub mod details;
pub mod notifications;
pub mod overview;

pub use details::{build_details_view, ChartArea, CHART_PANEL_PREFIX};
pub use notifications::build_notifications_view;
pub use overview::build_overview_view;
