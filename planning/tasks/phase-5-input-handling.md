# Phase 5: Input Handling Migration

## Goal
Replace crossterm keyboard input with evdev-based input from dashboard-system.

## Files to Modify
- `src/events.rs` - Replace crossterm with evdev
- `src/main.rs` - Initialize keyboard and wire to loop

## Reference Files
- `dashboard-system/src/input.rs` - KeyboardInput implementation

## Implementation Tasks

### Task 5.1: Update events.rs
```rust
//! Keyboard event handling for OpenGL dashboard

use dashboard_system::KeyboardInput;
use crate::app::{App, View};

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

fn map_key_event(event: dashboard_system::KeyEvent, view: View) -> AppEvent {
    use dashboard_system::KeyEvent;

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
```

### Task 5.2: Add scroll methods to App
In `src/app.rs`:
```rust
impl App {
    /// Scroll candles left (into history)
    pub fn scroll_candles_left(&mut self) {
        self.candle_scroll_offset += 5; // Scroll by 5 candles
    }

    /// Scroll candles right (toward present)
    pub fn scroll_candles_right(&mut self) {
        self.candle_scroll_offset -= 5;
        if self.candle_scroll_offset < 0 {
            self.candle_scroll_offset = 0;
        }
    }

    /// Reset candle scroll to latest
    pub fn reset_candle_scroll(&mut self) {
        self.candle_scroll_offset = 0;
    }

    /// Quit application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
```

### Task 5.3: Initialize keyboard in main.rs
```rust
use dashboard_system::KeyboardInput;
use crate::events::handle_gl_events;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... display initialization ...

    // Initialize keyboard input
    let mut keyboard = KeyboardInput::new();

    // ... rest of setup ...

    run_gl_loop(&mut display, &mut app, &mut keyboard, ...)?;

    Ok(())
}

fn run_gl_loop(
    display: &mut Display,
    app: &mut App,
    keyboard: &mut KeyboardInput,
    // ... other params ...
) -> Result<(), Box<dyn std::error::Error>> {
    while app.running {
        // ... tokio poll, price updates ...

        // Handle keyboard input
        handle_gl_events(keyboard, app);

        // ... render ...
    }
    Ok(())
}
```

### Task 5.4: Verify KeyEvent mapping in dashboard-system
Check `dashboard-system/src/input.rs` for available key events:
```rust
pub enum KeyEvent {
    Escape,
    Enter,
    Tab,
    Space,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Char(char),
    ShiftTab,
    // ... others
}
```

If some keys are missing, may need to extend the KeyEvent enum.

### Task 5.5: Add character key support
If dashboard-system doesn't support character keys like 'q', 'w', 'c':

In `dashboard-system/src/input.rs`, extend `parse_key`:
```rust
fn parse_key(code: u16) -> Option<KeyEvent> {
    match code {
        // Existing mappings...
        16 => Some(KeyEvent::Char('q')),   // KEY_Q
        17 => Some(KeyEvent::Char('w')),   // KEY_W
        18 => Some(KeyEvent::Char('e')),   // KEY_E
        19 => Some(KeyEvent::Char('r')),   // KEY_R
        // ... etc for other letters
        35 => Some(KeyEvent::Char('h')),   // KEY_H
        36 => Some(KeyEvent::Char('j')),   // KEY_J
        37 => Some(KeyEvent::Char('k')),   // KEY_K
        38 => Some(KeyEvent::Char('l')),   // KEY_L
        46 => Some(KeyEvent::Char('c')),   // KEY_C
        _ => None,
    }
}
```

### Task 5.6: Remove old crossterm code
Delete crossterm event handling from `src/events.rs` (old implementation).

## Key Mappings
| Key | Action | View |
|-----|--------|------|
| q / Esc | Quit | Both |
| Up / k | Move selection up | Overview |
| Down / j | Move selection down | Overview |
| Left / h | Scroll candles left | Details |
| Right / l | Scroll candles right | Details |
| Space | Toggle coin selection | Overview |
| Tab / Enter | Switch view | Both |
| w | Cycle time window | Both |
| c | Cycle chart type | Both |
| r / Home | Reset scroll | Details |

## Linux Key Codes Reference
```
KEY_ESC = 1
KEY_Q = 16
KEY_W = 17
KEY_R = 19
KEY_H = 35
KEY_J = 36
KEY_K = 37
KEY_L = 38
KEY_C = 46
KEY_SPACE = 57
KEY_ENTER = 28
KEY_TAB = 15
KEY_UP = 103
KEY_DOWN = 108
KEY_LEFT = 105
KEY_RIGHT = 106
KEY_HOME = 102
```

## Validation
- Press q to quit application
- Up/Down navigates coin list in Overview
- Space toggles coin selection
- Tab switches between Overview and Details
- Left/Right scrolls candles in Details view
- w cycles through 15m/1h/4h/1d
- c toggles Line/Candlestick
- No input lag or missed key events

## Notes
- evdev reads directly from /dev/input/event*
- Non-blocking poll with timeout
- No terminal mode required (works without TTY)
