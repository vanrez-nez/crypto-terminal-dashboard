//! OpenGL view compositions for the crypto dashboard

pub mod overview;
pub mod details;

pub use overview::build_overview_view;
pub use details::{build_details_view, ChartArea};
