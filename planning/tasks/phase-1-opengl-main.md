# Phase 1: Create OpenGL Main Entry Point

## Goal
Replace the ratatui terminal setup with DRM/GBM/EGL OpenGL initialization and create the main render loop.

## Files to Modify
- `src/main.rs` - Complete rewrite of entry point

## Reference Files (patterns to copy)
- `dashboard-system/src/drm_display.rs` - DRM/GBM/EGL setup
- `dashboard-system/src/main.rs` - Render loop structure

## Implementation Tasks

### Task 1.1: Remove ratatui/crossterm setup
- Remove imports: `crossterm`, `ratatui`
- Remove terminal setup code (enable_raw_mode, EnterAlternateScreen)
- Remove Terminal struct and backend
- Keep: `mod api`, `mod app`, `mod config`, `mod mock`, `mod theme`, `mod widgets`, `mod views`
- Remove: `mod ui` (will delete later)

### Task 1.2: Add Display initialization
```rust
use dashboard_system::{
    Display,
    FontAtlas,
    TextRenderer,
    RectRenderer,
    LayoutTree,
    render,
};
use crate::widgets::ChartRenderer;
```

Create display:
```rust
let mut display = Display::new().expect("Failed to initialize DRM display");
let (width, height) = display.size();
```

### Task 1.3: Initialize renderers
```rust
// Font atlas with embedded font
const FONT_DATA: &[u8] = include_bytes!("../assets/Roboto-Medium.ttf");
const FONT_SIZE: f32 = 24.0;

let atlas = FontAtlas::new(&display.gl, FONT_DATA, FONT_SIZE)
    .expect("Failed to create font atlas");
let mut text_renderer = TextRenderer::new(&display.gl)
    .expect("Failed to create text renderer");
let mut rect_renderer = RectRenderer::new(&display.gl)
    .expect("Failed to create rect renderer");
let mut chart_renderer = ChartRenderer::new(&display.gl)
    .expect("Failed to create chart renderer");
```

### Task 1.4: Integrate Tokio runtime
Replace `#[tokio::main]` with manual runtime:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Create channels (same as before)
    let (price_tx, mut price_rx) = mpsc::channel::<PriceUpdate>(100);
    let (candle_req_tx, candle_req_rx) = mpsc::channel::<(String, u32)>(32);

    // Spawn async tasks
    if use_live {
        let ws_provider = BinanceProvider::new(pairs.clone());
        let ws_tx = price_tx.clone();
        rt.spawn(async move { ws_provider.run(ws_tx).await });

        // Candle fetcher task...
    }

    // Run main loop (synchronous)
    run_gl_loop(&mut display, &mut app, &mut price_rx, candle_req_tx, ...)?;

    Ok(())
}
```

### Task 1.5: Create render loop structure
```rust
fn run_gl_loop(
    display: &mut Display,
    app: &mut App,
    price_rx: &mut mpsc::Receiver<PriceUpdate>,
    candle_req_tx: mpsc::Sender<(String, u32)>,
    rt: &tokio::runtime::Runtime,
    // ... renderers, atlas, etc.
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = display.size();
    let mut tree = LayoutTree::new();

    while app.running {
        // 1. Poll Tokio tasks (non-blocking)
        rt.block_on(async { tokio::task::yield_now().await });

        // 2. Handle candle refresh requests
        if app.needs_candle_refresh {
            app.needs_candle_refresh = false;
            let granularity = app.time_window.granularity();
            for pair in pairs {
                let _ = rt.block_on(candle_req_tx.send((pair.clone(), granularity)));
            }
        }

        // 3. Process price updates (non-blocking)
        while let Ok(update) = price_rx.try_recv() {
            app.handle_update(update);
        }

        // 4. Handle input (placeholder for Phase 5)
        // keyboard.poll_events() ...

        // 5. Build layout tree (placeholder for Phase 2/3)
        let root = build_current_view(&mut tree, app, &theme, width as f32, height as f32);

        // 6. Clear screen and render
        unsafe {
            display.gl.clear_color(0.05, 0.05, 0.08, 1.0);
            display.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        render(&display.gl, &tree, root, &atlas, &mut text_renderer, &mut rect_renderer, width, height);

        // 7. Render charts (placeholder for Phase 4)
        // chart_renderer.begin(); ... chart_renderer.end();

        // 8. Swap buffers (vsync)
        display.swap_buffers()?;
    }

    Ok(())
}
```

### Task 1.6: Add placeholder view builder
```rust
fn build_current_view(
    tree: &mut LayoutTree,
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> taffy::NodeId {
    // Temporary: just a colored background panel
    use dashboard_system::{panel, length};
    tree.build(
        panel()
            .width(length(width))
            .height(length(height))
            .background(theme.background)
    )
}
```

## Validation
- Display initializes without crash
- Screen shows solid background color
- App state loads correctly (coins, theme, config)
- WebSocket connection establishes (check logs)
- Price updates are received (debug print)

## Dependencies to Add
Ensure `dashboard-system` is available as dependency:
```toml
[dependencies]
dashboard-system = { path = "dashboard-system" }
```

## Notes
- Keep `#[allow(dead_code)]` on unused modules during transition
- The `ui` module will be removed in Phase 6
- Input handling is placeholder until Phase 5
