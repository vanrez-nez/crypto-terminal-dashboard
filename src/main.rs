mod api;
mod app;
mod config;
mod events;
mod mock;
mod theme;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use api::binance::{fetch_candles, granularity_to_interval, BinanceProvider};
use api::PriceUpdate;
use app::App;
use config::Config;
use mock::{coins_from_pairs, generate_mock_coins};

#[tokio::main]
async fn main() -> io::Result<()> {
    // Load config
    let config = Config::load("config.json");
    let theme = config.build_theme();
    let pairs = config.pairs();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create channel for price updates
    let (tx, mut rx) = mpsc::channel::<PriceUpdate>(100);

    // Create channel for candle fetch requests (product_id, granularity)
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
    let mut app = App::new(coins, theme, provider);

    // Spawn WebSocket task if using live data
    if use_live {
        let ws_provider = BinanceProvider::new(pairs.clone());
        let ws_tx = tx.clone();
        tokio::spawn(async move {
            ws_provider.run(ws_tx).await;
        });

        // Spawn candle fetcher task
        let candle_tx = tx.clone();
        tokio::spawn(async move {
            while let Some((symbol, granularity)) = candle_req_rx.recv().await {
                let interval = granularity_to_interval(granularity);
                match fetch_candles(&symbol, interval).await {
                    Ok(candles) => {
                        // Extract symbol (e.g., "BTCUSDT" -> "BTC")
                        let sym = symbol.trim_end_matches("USDT").to_string();
                        let _ = candle_tx.send(PriceUpdate::Candles { symbol: sym, candles }).await;
                    }
                    Err(e) => {
                        let _ = candle_tx.send(PriceUpdate::Error(format!("Candle fetch error: {}", e))).await;
                    }
                }
            }
        });
    }

    // Main loop
    let result = run_app(&mut terminal, &mut app, &mut rx, candle_req_tx, &pairs).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::Receiver<PriceUpdate>,
    candle_req_tx: mpsc::Sender<(String, u32)>,
    pairs: &[String],
) -> io::Result<()> {
    while app.running {
        // Check if we need to fetch candles (startup or window change)
        if app.needs_candle_refresh {
            app.needs_candle_refresh = false;
            let granularity = app.time_window.granularity();
            for pair in pairs {
                let _ = candle_req_tx.send((pair.clone(), granularity)).await;
            }
        }

        // Process all pending price updates (non-blocking)
        while let Ok(update) = rx.try_recv() {
            app.handle_update(update);
        }

        // Draw UI
        terminal.draw(|frame| ui::render(frame, app))?;

        // Handle keyboard events
        events::handle_events(app)?;
    }
    Ok(())
}
