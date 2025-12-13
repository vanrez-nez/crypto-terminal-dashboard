# Phase 2: Implement Overview View

## Goal
Create the Overview view that displays the coin table with selection, status header, and control footer.

## Files to Create
- `src/views/overview.rs`

## Files to Modify
- `src/views/mod.rs` - Add exports
- `src/main.rs` - Wire view to render loop

## Implementation Tasks

### Task 2.1: Update views/mod.rs
```rust
//! OpenGL view compositions for the crypto dashboard

pub mod overview;

pub use overview::build_overview_view;
```

### Task 2.2: Create views/overview.rs
```rust
//! Overview view - coin table with selection

use dashboard_system::{panel, PanelBuilder, taffy};
use taffy::prelude::*;

use crate::app::{App, ConnectionStatus, View, TimeWindow, ChartType};
use crate::widgets::{
    build_coin_table,
    build_status_header,
    build_overview_footer,
    GlTheme,
};

pub fn build_overview_view(
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> PanelBuilder {
    panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
        .background(theme.background)
        // Header - fixed height
        .child(
            build_status_header(
                app.view,
                &app.provider,
                app.time_window,
                app.chart_type,
                app.connection_status,
                theme,
            )
        )
        // Coin table - grows to fill space
        .child(
            build_coin_table(
                &app.coins,
                app.selected_index,
                &app.checked,
                theme,
            )
        )
        // Footer - fixed height
        .child(
            build_overview_footer(theme)
        )
}
```

### Task 2.3: Wire view to main render loop
Update `build_current_view` in main.rs:
```rust
use crate::views::build_overview_view;
use crate::widgets::GlTheme;

fn build_current_view(
    tree: &mut LayoutTree,
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> taffy::NodeId {
    match app.view {
        View::Overview => {
            tree.build(build_overview_view(app, theme, width, height))
        }
        View::Details => {
            // Placeholder until Phase 3
            tree.build(
                panel()
                    .width(length(width))
                    .height(length(height))
                    .background(theme.background)
            )
        }
    }
}
```

### Task 2.4: Verify existing widgets work
Check that these widgets from `src/widgets/` compile and render:
- `status_header.rs` - `build_status_header()`
- `coin_table.rs` - `build_coin_table()`
- `control_footer.rs` - `build_overview_footer()`

Fix any import issues or API mismatches.

### Task 2.5: Handle GlTheme vs ratatui Theme
The app currently uses `theme::Theme` (ratatui). Need to:
1. Add `GlTheme` construction from config
2. Pass `GlTheme` to views

In `config.rs` or `main.rs`:
```rust
use crate::widgets::GlTheme;

impl Config {
    pub fn build_gl_theme(&self) -> GlTheme {
        // Convert config colors to GlTheme
        GlTheme::from_config(self)
    }
}
```

Or use default:
```rust
let gl_theme = GlTheme::default();
```

## Layout Structure
```
+------------------------------------------+
| Status Header (50px)                     |
| [OVERVIEW] BTC/USD | Binance | 1h | Line |
+------------------------------------------+
| Coin Table (flex grow)                   |
| [ ] PAIR   PRICE    24h%   VOL   H/L    |
| [x] BTC    67432   +2.34%  28.4B ...    |
| [ ] ETH    3521    -0.82%  14.2B ...    |
| [ ] SOL    142     +5.21%  2.1B  ...    |
+------------------------------------------+
| Footer (36px)                            |
| [Space] Select  [Tab] Details  [q] Quit |
+------------------------------------------+
```

## Validation
- Overview view renders with all three sections
- Coin data displays correctly (symbol, price, change, volume)
- Selected row shows highlight background
- Checked coins show [x] checkbox
- Colors match theme (positive=green, negative=red)
- Text is readable and properly aligned

## Potential Issues
- Font size may need adjustment for readability
- Column widths may need tuning based on screen resolution
- Scroll functionality won't work until focus system is wired (Phase 5)
