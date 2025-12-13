//! OpenGL view compositions for the crypto dashboard

pub mod details;
pub mod overview;

pub use details::{build_details_view, ChartArea};
pub use overview::build_overview_view;
