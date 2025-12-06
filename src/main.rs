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

use api::coinbase::CoinbaseProvider;
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

    // Determine provider
    let provider = config.provider();
    let use_live = provider == "coinbase";

    // Create app with appropriate data source
    let coins = if use_live {
        coins_from_pairs(&pairs)
    } else {
        generate_mock_coins()
    };
    let mut app = App::new(coins, theme, provider);

    // Spawn WebSocket task if using live data
    if use_live {
        let provider = CoinbaseProvider::new(pairs);
        tokio::spawn(async move {
            provider.run(tx).await;
        });
    }

    // Main loop
    let result = run_app(&mut terminal, &mut app, &mut rx).await;

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
) -> io::Result<()> {
    while app.running {
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
