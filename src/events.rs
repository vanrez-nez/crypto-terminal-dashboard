use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::{App, View};

pub fn handle_events(app: &mut App) -> std::io::Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            handle_key(app, key);
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Up | KeyCode::Char('k') => {
            if app.view == View::Overview {
                app.move_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.view == View::Overview {
                app.move_down();
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if app.view == View::Details {
                app.scroll_candles_left();
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.view == View::Details {
                app.scroll_candles_right();
            }
        }
        KeyCode::Home => app.reset_candle_scroll(),
        KeyCode::Char(' ') => app.toggle_selection(),
        KeyCode::Enter | KeyCode::Tab => app.switch_view(),
        KeyCode::Char('w') => app.cycle_window(),
        KeyCode::Char('c') => app.cycle_chart_type(),
        KeyCode::Char('r') => app.reset_candle_scroll(),
        _ => {}
    }
}
