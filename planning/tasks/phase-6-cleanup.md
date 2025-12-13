# Phase 6: Cleanup and Dependencies

## Goal
Remove ratatui and crossterm dependencies, delete old TUI code, and finalize the migration.

## Files to Modify
- `Cargo.toml` - Remove dependencies
- `src/main.rs` - Clean up imports

## Files to Delete
- `src/ui/` - Entire directory (ratatui rendering)
- `src/theme.rs` - Ratatui theme (if separate from GlTheme)

## Implementation Tasks

### Task 6.1: Remove dependencies from Cargo.toml
Remove these dependencies:
```toml
# Remove these lines:
ratatui = "0.28"
crossterm = "0.28"
```

Keep or update:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.24"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures-util = "0.3"
bytemuck = { version = "1.14", features = ["derive"] }

# Local dependency
dashboard-system = { path = "dashboard-system" }
```

### Task 6.2: Delete src/ui/ directory
```bash
rm -rf src/ui/
```

Files being removed:
- `src/ui/mod.rs`
- `src/ui/overview.rs`
- `src/ui/details.rs`

### Task 6.3: Remove old theme.rs if unused
If `src/theme.rs` only contains ratatui theme code:
```bash
rm src/theme.rs
```

The GlTheme in `src/widgets/theme.rs` replaces it.

### Task 6.4: Update main.rs imports
Remove:
```rust
// Remove these
mod ui;
mod theme; // if deleted

use crossterm::{...};
use ratatui::{...};
```

Keep:
```rust
mod api;
mod app;
mod config;
mod events;
mod mock;
mod views;
mod widgets;

use dashboard_system::{Display, FontAtlas, TextRenderer, RectRenderer, LayoutTree, render, glow};
use crate::widgets::{ChartRenderer, GlTheme};
use crate::views::{build_overview_view, build_details_view, render_charts};
use crate::events::handle_gl_events;
```

### Task 6.5: Update src/views/mod.rs
Final exports:
```rust
//! OpenGL view compositions for the crypto dashboard

pub mod overview;
pub mod details;

pub use overview::build_overview_view;
pub use details::{build_details_view, render_charts, ChartArea};
```

### Task 6.6: Clean up config.rs
If config.rs references old theme:
```rust
// Remove
use crate::theme::Theme;

impl Config {
    pub fn build_theme(&self) -> Theme { ... }
}

// Replace with
use crate::widgets::GlTheme;

impl Config {
    pub fn build_gl_theme(&self) -> GlTheme { ... }
}
```

### Task 6.7: Run cargo check
```bash
cargo check
```

Fix any remaining compilation errors from removed code.

### Task 6.8: Run cargo build --release
```bash
cargo build --release
```

Ensure release build succeeds for deployment.

### Task 6.9: Update .gitignore if needed
Ensure compiled assets and target directories are ignored:
```
/target
*.swp
*.swo
.DS_Store
```

### Task 6.10: Test on target hardware
Cross-compile and deploy to Raspberry Pi:
```bash
# Cross-compile (example)
cargo build --release --target aarch64-unknown-linux-gnu

# Copy to Pi
scp target/aarch64-unknown-linux-gnu/release/crypto-dashboard monode.local:~/

# SSH and run
ssh monode.local
./crypto-dashboard
```

## Dependency Changes Summary

### Removed
| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.28 | TUI rendering |
| crossterm | 0.28 | Terminal control |

### Added/Required
| Crate | Version | Purpose |
|-------|---------|---------|
| dashboard-system | local | OpenGL rendering |
| glow | 0.13 | OpenGL ES wrapper |
| khronos-egl | 6.0 | EGL bindings |
| gbm | 0.15 | Buffer management |
| drm | 0.12 | Display control |
| fontdue | 0.9 | Font rasterization |
| taffy | 0.5 | Flexbox layout |

### Unchanged
| Crate | Purpose |
|-------|---------|
| tokio | Async runtime |
| tokio-tungstenite | WebSocket |
| reqwest | HTTP client |
| serde/serde_json | Serialization |
| bytemuck | Vertex data |

## Final File Structure
```
src/
├── main.rs          # OpenGL entry point
├── app.rs           # Application state
├── config.rs        # Configuration loading
├── events.rs        # evdev keyboard handling
├── mock.rs          # CoinData, indicators
├── api/
│   ├── mod.rs
│   ├── binance.rs
│   └── coinbase.rs
├── views/
│   ├── mod.rs
│   ├── overview.rs  # Overview layout
│   └── details.rs   # Details + chart rendering
└── widgets/
    ├── mod.rs
    ├── chart_renderer.rs
    ├── coin_table.rs
    ├── control_footer.rs
    ├── format.rs
    ├── indicator_panel.rs
    ├── price_panel.rs
    ├── status_header.rs
    └── theme.rs
```

## Validation
- `cargo check` passes with no errors
- `cargo build --release` completes successfully
- Application runs on Raspberry Pi
- All views render correctly
- Input handling works
- WebSocket data flows correctly
- No ratatui/crossterm remnants in code

## Notes
- Keep dashboard-system as local path dependency for now
- Consider publishing dashboard-system separately later
- Backup old code before deletion if needed for reference
