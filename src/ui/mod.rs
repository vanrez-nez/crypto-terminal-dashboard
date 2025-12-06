pub mod details;
pub mod overview;
pub mod widgets;

use ratatui::Frame;

use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App) {
    match app.view {
        View::Overview => overview::render(frame, app),
        View::Details => details::render(frame, app),
    }
}
