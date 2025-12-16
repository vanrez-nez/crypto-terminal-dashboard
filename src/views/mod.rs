//! OpenGL view compositions for the crypto dashboard

pub mod details;
pub mod news;
pub mod notifications;
pub mod overview;
pub mod positions;

pub use details::{build_details_view, ChartArea, CHART_PANEL_PREFIX};
pub use news::build_news_view;
pub use notifications::build_notifications_view;
pub use overview::build_overview_view;
pub use positions::build_positions_view;
