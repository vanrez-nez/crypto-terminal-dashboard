mod api;
mod app;
mod config;
mod events;
mod mock;
mod views;
mod widgets;

use tokio::sync::mpsc;

use dashboard_system::{
    glow, panel, render, taffy, Display, FocusManager, FontAtlas, KeyboardInput, LayoutTree,
    RectRenderer, ScissorStack, TextRenderer,
};
use glow::HasContext;

use api::binance::{fetch_candles, granularity_to_interval, BinanceProvider};
use api::PriceUpdate;
use app::App;
use config::Config;
use events::handle_gl_events;
use mock::{coins_from_pairs, generate_mock_coins};
use widgets::chart_renderer::ChartRenderer;
use widgets::theme::GlTheme;

// Font data embedded from dashboard-system
const FONT_DATA: &[u8] = include_bytes!("../dashboard-system/assets/Roboto-Medium.ttf");
const FONT_SIZE: f32 = 17.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create tokio runtime manually (not async main)
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Load config
    let config = Config::load("config.json");
    let pairs = config.pairs();

    // Create GlTheme from config
    let gl_theme = match &config.theme {
        Some(theme_config) => GlTheme::from_config(theme_config),
        None => GlTheme::default(),
    };

    // Initialize DRM/GBM/EGL display
    let mut display = Display::new().expect("Failed to initialize DRM display");
    let (width, height) = (display.width, display.height);

    // Font atlas
    let atlas = FontAtlas::new(&display.gl, FONT_DATA, FONT_SIZE)?;

    // Renderers
    let mut text_renderer = TextRenderer::new(&display.gl)?;
    let mut rect_renderer = RectRenderer::new(&display.gl)?;
    let mut chart_renderer = ChartRenderer::new(&display.gl)?;
    let mut scissor_stack = ScissorStack::new(height);
    let focus_manager = FocusManager::new();

    // Create channels for price updates and candle requests
    let (price_tx, mut price_rx) = mpsc::channel::<PriceUpdate>(100);
    let (candle_req_tx, mut candle_req_rx) = mpsc::channel::<(String, u32)>(32);

    // Determine provider
    let provider = config.provider();
    let use_live = provider == "binance";

    // Create app with appropriate data source
    let coins = if use_live {
        coins_from_pairs(&pairs)
    } else {
        generate_mock_coins()
    };

    let mut app = App::new(coins, provider);

    // Spawn WebSocket task if using live data
    if use_live {
        let ws_provider = BinanceProvider::new(pairs.clone());
        let ws_tx = price_tx.clone();
        rt.spawn(async move {
            ws_provider.run(ws_tx).await;
        });

        // Spawn candle fetcher task
        let candle_tx = price_tx.clone();
        rt.spawn(async move {
            while let Some((symbol, granularity)) = candle_req_rx.recv().await {
                let interval = granularity_to_interval(granularity);
                match fetch_candles(&symbol, interval).await {
                    Ok(candles) => {
                        // Extract symbol (e.g., "BTCUSDT" -> "BTC")
                        let sym = symbol.trim_end_matches("USDT").to_string();
                        let _ = candle_tx
                            .send(PriceUpdate::Candles {
                                symbol: sym,
                                candles,
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = candle_tx
                            .send(PriceUpdate::Error(format!("Candle fetch error: {}", e)))
                            .await;
                    }
                }
            }
        });
    }

    // Initialize keyboard input (evdev-based)
    let mut keyboard = KeyboardInput::new();

    // Run the OpenGL render loop
    run_gl_loop(
        &mut display,
        &mut app,
        &mut keyboard,
        &mut price_rx,
        candle_req_tx,
        &rt,
        &pairs,
        &atlas,
        &mut text_renderer,
        &mut rect_renderer,
        &mut chart_renderer,
        &mut scissor_stack,
        &focus_manager,
        &gl_theme,
    )?;

    Ok(())
}

fn run_gl_loop(
    display: &mut Display,
    app: &mut App,
    keyboard: &mut KeyboardInput,
    price_rx: &mut mpsc::Receiver<PriceUpdate>,
    candle_req_tx: mpsc::Sender<(String, u32)>,
    rt: &tokio::runtime::Runtime,
    pairs: &[String],
    atlas: &FontAtlas,
    text_renderer: &mut TextRenderer,
    rect_renderer: &mut RectRenderer,
    _chart_renderer: &mut ChartRenderer, // Placeholder for Phase 4
    scissor_stack: &mut ScissorStack,
    focus_manager: &FocusManager,
    theme: &GlTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = (display.width, display.height);

    while app.running {
        // 1. Poll tokio tasks (non-blocking)
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

        // 4. Handle keyboard input (evdev-based)
        handle_gl_events(keyboard, app);

        // 5. Build layout tree
        let mut tree = LayoutTree::new();
        let view_result = build_current_view(&mut tree, app, theme, width as f32, height as f32);
        tree.compute_with_text(view_result.root, width as f32, height as f32, atlas);

        // 6. Clear screen
        unsafe {
            display.gl.clear_color(
                theme.background[0],
                theme.background[1],
                theme.background[2],
                theme.background[3],
            );
            display.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        // 7. Render layout tree
        render(
            &display.gl,
            &tree,
            view_result.root,
            rect_renderer,
            text_renderer,
            atlas,
            scissor_stack,
            focus_manager,
            width,
            height,
        );

        // 8. Chart rendering (Phase 4)
        // Chart areas are available in view_result.chart_areas for Details view
        // TODO: Use chart_renderer to draw charts in each chart area
        let _ = &view_result.chart_areas; // Suppress unused warning for now

        // 9. Swap buffers (vsync)
        display.swap_buffers()?;
    }

    Ok(())
}

/// Result of building a view, includes layout root and optional chart areas
struct ViewResult {
    root: taffy::NodeId,
    chart_areas: Vec<views::ChartArea>,
}

fn build_current_view(
    tree: &mut LayoutTree,
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> ViewResult {
    use crate::app::View;
    use crate::views::{build_details_view, build_overview_view};

    match app.view {
        View::Overview => ViewResult {
            root: build_overview_view(app, theme, width, height).build(tree),
            chart_areas: vec![],
        },
        View::Details => {
            let (panel, chart_areas) = build_details_view(app, theme, width, height);
            ViewResult {
                root: panel.build(tree),
                chart_areas,
            }
        }
    }
}
