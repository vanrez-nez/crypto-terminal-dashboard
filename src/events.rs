//! Keyboard event handling for OpenGL dashboard

use crate::app::{App, View};
use crate::base::{KeyEvent, KeyboardInput};

/// Key event types we care about
pub enum AppEvent {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ZoomIn,
    ZoomOut,
    Select,
    SwitchView,
    CycleWindow,
    CycleChartType,
    ResetScroll,
    ToggleMute,
    // Notifications view events
    NotificationRuleUp,
    NotificationRuleDown,
    ToggleNotificationRule,
    // News view events
    NewsScrollUp,
    NewsScrollDown,
    ContentScrollUp,
    ContentScrollDown,
    RefreshNews,
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

        // Navigation / Zoom (context-dependent on view)
        KeyEvent::Up | KeyEvent::Char('k') => match view {
            View::Details => AppEvent::ZoomIn,
            View::Notifications => AppEvent::NotificationRuleUp,
            View::News => AppEvent::NewsScrollUp,
            View::Overview => AppEvent::MoveUp,
        },
        KeyEvent::Down | KeyEvent::Char('j') => match view {
            View::Details => AppEvent::ZoomOut,
            View::Notifications => AppEvent::NotificationRuleDown,
            View::News => AppEvent::NewsScrollDown,
            View::Overview => AppEvent::MoveDown,
        },
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
        KeyEvent::Space => {
            if view == View::Notifications {
                AppEvent::ToggleNotificationRule
            } else {
                AppEvent::Select
            }
        }
        KeyEvent::Tab | KeyEvent::Enter => AppEvent::SwitchView,
        KeyEvent::Char('w') => AppEvent::CycleWindow,
        KeyEvent::Char('c') => AppEvent::CycleChartType,
        KeyEvent::Char('r') => {
            if view == View::News {
                AppEvent::RefreshNews
            } else {
                AppEvent::ResetScroll
            }
        }
        KeyEvent::Home => AppEvent::ResetScroll,
        KeyEvent::Char('m') => AppEvent::ToggleMute,

        // Page Up/Down for content scrolling in News view
        KeyEvent::PageUp => match view {
            View::News => AppEvent::ContentScrollUp,
            _ => AppEvent::None,
        },
        KeyEvent::PageDown => match view {
            View::News => AppEvent::ContentScrollDown,
            _ => AppEvent::None,
        },

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
        AppEvent::ZoomIn => app.zoom_in(),
        AppEvent::ZoomOut => app.zoom_out(),
        AppEvent::Select => app.toggle_selection(),
        AppEvent::SwitchView => app.switch_view(),
        AppEvent::CycleWindow => app.cycle_window(),
        AppEvent::CycleChartType => app.cycle_chart_type(),
        AppEvent::ResetScroll => app.reset_candle_scroll(),
        AppEvent::ToggleMute => app.toggle_mute(),
        // Notifications view actions
        AppEvent::NotificationRuleUp => app.select_prev_rule(),
        AppEvent::NotificationRuleDown => app.select_next_rule(),
        AppEvent::ToggleNotificationRule => app.toggle_notification_rule(),
        // News view actions
        AppEvent::NewsScrollUp => app.scroll_news_up(),
        AppEvent::NewsScrollDown => app.scroll_news_down(),
        AppEvent::ContentScrollUp => app.scroll_content_up(),
        AppEvent::ContentScrollDown => app.scroll_content_down(),
        AppEvent::RefreshNews => app.refresh_news(),
        AppEvent::None => {}
    }
}
