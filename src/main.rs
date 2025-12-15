mod api;
mod app;
mod base;
mod config;
mod events;
mod mock;
mod notifications;
mod views;
mod widgets;

use tokio::sync::mpsc;

use crate::base::{
    glow, render, taffy, Display, FocusManager, FontAtlas, KeyboardInput, LayoutTree, RectRenderer,
    ScissorStack, TextRenderer,
};
use glow::HasContext;

use api::binance::{fetch_candles, granularity_to_interval, BinanceProvider};
use api::PriceUpdate;
use app::{App, ChartType};
use config::Config;
use events::handle_gl_events;
use mock::{coins_from_pairs, generate_mock_coins};
use notifications::{audio, persistence, NotificationManager};
use views::CHART_PANEL_PREFIX;
use widgets::candlestick_chart::render_candlestick_chart;
use widgets::polygonal_chart::render_polygonal_chart;
use widgets::chart_renderer::{ChartRenderer, PixelRect};
use widgets::theme::GlTheme;

// Font data embedded from fonts directory
const FONT_DATA: &[u8] = include_bytes!("../fonts/CascadiaMonoPL.ttf");
const FONT_SIZE: f32 = 17.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create tokio runtime manually (not async main)
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Load config
    let config = Config::load("config.json");
    let pairs = config.pairs();

    // Create GlTheme from config (loads theme by name)
    let gl_theme = match config.theme_config() {
        Some(theme_config) => GlTheme::from_config(&theme_config),
        None => GlTheme::default(),
    };

    // Initialize DRM/GBM/EGL display
    let mut display = Display::new().expect("Failed to initialize DRM display");
    let height = display.height;

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

    // Initialize notification manager from config
    let notif_config = config.notifications_config();
    let mut notification_manager = NotificationManager::new(
        notif_config.rules.clone(),
        notif_config.cooldown_secs,
        notif_config.max_log_entries,
    );

    // Load existing notifications from log file
    let existing_notifications = persistence::load_notifications(&notif_config.log_file);
    notification_manager.load_notifications(existing_notifications);

    // Initialize audio if enabled
    if notif_config.audio_enabled {
        audio::init_audio();
    }

    let mut app = App::with_notification_manager(coins, provider, notification_manager);

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
        &config,
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
    chart_renderer: &mut ChartRenderer,
    scissor_stack: &mut ScissorStack,
    focus_manager: &FocusManager,
    theme: &GlTheme,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = (display.width, display.height);
    let notifications_enabled = config.notifications_enabled();
    let audio_enabled = config.audio_enabled();
    let log_file = config.log_file();
    let ticker_tones_config = config.ticker_tones_config();

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

        // 3.5. Play ticker tones for price changes (checked coins only, if not muted)
        if ticker_tones_config.enabled && !app.ticker_muted {
            notifications::process_ticker_tones(&app.coins, &app.checked, &ticker_tones_config);
        }

        // 4. Check notification rules after price updates (checked coins only)
        if notifications_enabled {
            let new_notifications = app.notification_manager.check_rules(&app.coins, &app.checked);
            if !new_notifications.is_empty() {
                // Play audio for each new notification
                if audio_enabled {
                    for notif in &new_notifications {
                        audio::play_alert(notif.sound.as_deref());
                    }
                }
                // Save updated notifications to log file
                persistence::save_notifications(&app.notification_manager.notifications, &log_file);
            }
        }

        // 5. Handle keyboard input (evdev-based)
        handle_gl_events(keyboard, app);

        // 6. Build layout tree
        let mut tree = LayoutTree::new();
        let view_result = build_current_view(&mut tree, app, theme, width as f32, height as f32);
        tree.compute_with_text(view_result.root, width as f32, height as f32, atlas);

        // 7. Clear screen
        unsafe {
            display.gl.clear_color(
                theme.background[0],
                theme.background[1],
                theme.background[2],
                theme.background[3],
            );
            display.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        // 8. Render layout tree
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

        // 9. Chart rendering
        if !view_result.chart_areas.is_empty() {
            // Find chart panel bounds from layout
            let chart_bounds = tree.find_panels_by_prefix(view_result.root, CHART_PANEL_PREFIX);

            // Match chart areas with their resolved bounds and render
            for chart_area in &view_result.chart_areas {
                // Find the matching bounds by chart index
                let marker_id = format!("{}{}", CHART_PANEL_PREFIX,
                    view_result.chart_areas.iter().position(|a| a.coin_index == chart_area.coin_index).unwrap_or(0));

                if let Some((_, x, y, w, h)) = chart_bounds.iter().find(|(id, _, _, _, _)| id == &marker_id) {
                    if let Some(coin) = app.coins.get(chart_area.coin_index) {
                        let rect = PixelRect::new(*x, *y, *w, *h);

                        // Enable scissor test to clip chart to its bounds
                        unsafe {
                            display.gl.enable(glow::SCISSOR_TEST);
                            // GL scissor uses bottom-left origin, convert from top-left
                            let scissor_y = height as i32 - (*y as i32 + *h as i32);
                            display.gl.scissor(*x as i32, scissor_y, *w as i32, *h as i32);
                        }

                        chart_renderer.begin();
                        match app.chart_type {
                            ChartType::Candlestick => render_candlestick_chart(
                                chart_renderer,
                                &coin.candles,
                                &coin.chart_indicators, // Use cached indicators
                                app.candle_scroll_offset,
                                app.visible_candles,
                                0.05, // 5% price margin
                                rect,
                                theme,
                            ),
                            ChartType::Polygonal => render_polygonal_chart(
                                chart_renderer,
                                &coin.candles,
                                app.candle_scroll_offset,
                                app.visible_candles,
                                0.05, // 5% price margin
                                rect,
                                theme,
                            ),
                        }
                        chart_renderer.end(&display.gl, width, height);

                        unsafe {
                            display.gl.disable(glow::SCISSOR_TEST);
                        }
                    }
                }
            }
        }

        // 10. Swap buffers (vsync)
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
    use crate::views::{build_details_view, build_notifications_view, build_overview_view};

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
        View::Notifications => ViewResult {
            root: build_notifications_view(app, theme, width, height).build(tree),
            chart_areas: vec![],
        },
    }
}
