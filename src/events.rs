//! Keyboard event handling for OpenGL dashboard

use crate::app::{App, View};
use dashboard_system::{KeyEvent, KeyboardInput};

/// Key event types we care about
pub enum AppEvent {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Select,
    SwitchView,
    CycleWindow,
    CycleChartType,
    ResetScroll,
    None,
}

/// Poll and handle keyboard events
pub fn handle_gl_events(keyboard: &mut KeyboardInput, app: &mut App) {
    for event in keyboard.poll_events() {
        let action = map_key_event(event, app.view);
        apply_action(app, action);
    }
}

fn map_key_event(event: KeyEvent, view: View) -> AppEvent {
    match event {
        KeyEvent::Escape | KeyEvent::Char('q') => AppEvent::Quit,

        // Navigation
        KeyEvent::Up | KeyEvent::Char('k') => AppEvent::MoveUp,
        KeyEvent::Down | KeyEvent::Char('j') => AppEvent::MoveDown,
        KeyEvent::Left | KeyEvent::Char('h') => {
            if view == View::Details {
                AppEvent::MoveLeft
            } else {
                AppEvent::None
            }
        }
        KeyEvent::Right | KeyEvent::Char('l') => {
            if view == View::Details {
                AppEvent::MoveRight
            } else {
                AppEvent::None
            }
        }

        // Actions
        KeyEvent::Space => AppEvent::Select,
        KeyEvent::Tab | KeyEvent::Enter => AppEvent::SwitchView,
        KeyEvent::Char('w') => AppEvent::CycleWindow,
        KeyEvent::Char('c') => AppEvent::CycleChartType,
        KeyEvent::Char('r') | KeyEvent::Home => AppEvent::ResetScroll,

        _ => AppEvent::None,
    }
}

fn apply_action(app: &mut App, action: AppEvent) {
    match action {
        AppEvent::Quit => app.quit(),
        AppEvent::MoveUp => {
            if app.view == View::Overview {
                app.move_up();
            }
        }
        AppEvent::MoveDown => {
            if app.view == View::Overview {
                app.move_down();
            }
        }
        AppEvent::MoveLeft => {
            app.scroll_candles_left();
        }
        AppEvent::MoveRight => {
            app.scroll_candles_right();
        }
        AppEvent::Select => app.toggle_selection(),
        AppEvent::SwitchView => app.switch_view(),
        AppEvent::CycleWindow => app.cycle_window(),
        AppEvent::CycleChartType => app.cycle_chart_type(),
        AppEvent::ResetScroll => app.reset_candle_scroll(),
        AppEvent::None => {}
    }
}
