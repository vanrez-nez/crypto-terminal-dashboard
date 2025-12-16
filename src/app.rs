use crate::api::margin::MarginAccount;
use crate::api::news::NewsArticle;
use crate::api::PriceUpdate;
use crate::mock::CoinData;
use crate::notifications::NotificationManager;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Overview,
    Details,
    Notifications,
    News,
    Positions,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartType {
    Polygonal,
    Candlestick,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Mock,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeWindow {
    Min15,
    Hour1,
    Hour4,
    Day1,
}

impl TimeWindow {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeWindow::Min15 => "15m",
            TimeWindow::Hour1 => "1h",
            TimeWindow::Hour4 => "4h",
            TimeWindow::Day1 => "1d",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TimeWindow::Min15 => TimeWindow::Hour1,
            TimeWindow::Hour1 => TimeWindow::Hour4,
            TimeWindow::Hour4 => TimeWindow::Day1,
            TimeWindow::Day1 => TimeWindow::Min15,
        }
    }

    /// Returns the candle interval in seconds for this time window
    pub fn granularity(&self) -> u32 {
        match self {
            TimeWindow::Min15 => 900,   // 15 minutes
            TimeWindow::Hour1 => 3600,  // 1 hour
            TimeWindow::Hour4 => 14400, // 4 hours
            TimeWindow::Day1 => 86400,  // 1 day
        }
    }
}

/// Zoom level presets: fewer candles = zoomed in, more candles = zoomed out
const ZOOM_LEVELS: [usize; 5] = [20, 35, 50, 80, 120];

pub struct App {
    pub view: View,
    pub coins: Vec<CoinData>,
    pub selected_index: usize,
    pub checked: Vec<bool>,
    pub running: bool,
    pub connection_status: ConnectionStatus,
    pub provider: String,
    pub time_window: TimeWindow,
    pub needs_candle_refresh: bool,
    pub chart_type: ChartType,
    pub candle_scroll_offset: isize,
    /// Number of visible candles (zoom level)
    pub visible_candles: usize,
    /// Notification manager
    pub notification_manager: NotificationManager,
    /// Scroll offset for notifications view
    pub notification_scroll: usize,
    /// Whether ticker tones are muted
    pub ticker_muted: bool,
    /// News articles from API
    pub news_articles: Vec<NewsArticle>,
    /// Selected news article index
    pub news_selected: usize,
    /// Scroll offset for news content (line-based)
    pub news_content_scroll: usize,
    /// Whether news is currently being fetched
    pub news_loading: bool,
    /// Flag to trigger news refresh
    pub needs_news_refresh: bool,
    /// Margin account data
    pub margin_account: Option<MarginAccount>,
    /// Selected position index for navigation
    pub positions_selected: usize,
    /// Scroll offset for positions table
    pub positions_scroll: usize,
    /// Flag to trigger positions refresh
    pub needs_positions_refresh: bool,
    /// Whether positions are currently loading
    pub positions_loading: bool,
    /// Whether positions API is available (API keys configured)
    pub positions_available: bool,
}

impl App {
    pub fn new(coins: Vec<CoinData>, provider: &str) -> Self {
        Self::with_notification_manager(coins, provider, NotificationManager::default())
    }

    pub fn with_notification_manager(
        coins: Vec<CoinData>,
        provider: &str,
        notification_manager: NotificationManager,
    ) -> Self {
        let coin_count = coins.len();
        let use_mock = provider == "mock";
        Self {
            view: View::Overview,
            coins,
            selected_index: 0,
            checked: vec![false; coin_count],
            running: true,
            connection_status: if use_mock {
                ConnectionStatus::Mock
            } else {
                ConnectionStatus::Connecting
            },
            provider: provider.to_string(),
            time_window: TimeWindow::Hour1,
            needs_candle_refresh: true, // Fetch candles on startup
            chart_type: ChartType::Candlestick,
            candle_scroll_offset: 0,
            visible_candles: 50, // Default zoom level
            notification_manager,
            notification_scroll: 0,
            ticker_muted: false,
            news_articles: Vec::new(),
            news_selected: 0,
            news_content_scroll: 0,
            news_loading: false,
            needs_news_refresh: false,
            margin_account: None,
            positions_selected: 0,
            positions_scroll: 0,
            needs_positions_refresh: false,
            positions_loading: false,
            positions_available: false,
        }
    }

    /// Enable positions feature (call when API keys are available)
    pub fn enable_positions(&mut self) {
        self.positions_available = true;
    }

    /// Toggle ticker tone mute state
    pub fn toggle_mute(&mut self) {
        self.ticker_muted = !self.ticker_muted;
    }

    /// Cycle between Polygonal and Candlestick chart types
    pub fn cycle_chart_type(&mut self) {
        self.chart_type = match self.chart_type {
            ChartType::Polygonal => ChartType::Candlestick,
            ChartType::Candlestick => ChartType::Polygonal,
        };
    }

    /// Scroll candle chart left (back in time)
    pub fn scroll_candles_left(&mut self) {
        self.candle_scroll_offset += 5;
    }

    /// Scroll candle chart right (forward in time, can go negative to snap to last candles)
    pub fn scroll_candles_right(&mut self) {
        self.candle_scroll_offset -= 5;
    }

    /// Reset candle scroll to most recent
    pub fn reset_candle_scroll(&mut self) {
        self.candle_scroll_offset = 0;
    }

    /// Zoom in: show fewer candles (each wider)
    pub fn zoom_in(&mut self) {
        if let Some(pos) = ZOOM_LEVELS.iter().position(|&z| z == self.visible_candles) {
            if pos > 0 {
                self.visible_candles = ZOOM_LEVELS[pos - 1];
            }
        }
    }

    /// Zoom out: show more candles (each thinner)
    pub fn zoom_out(&mut self) {
        if let Some(pos) = ZOOM_LEVELS.iter().position(|&z| z == self.visible_candles) {
            if pos < ZOOM_LEVELS.len() - 1 {
                self.visible_candles = ZOOM_LEVELS[pos + 1];
            }
        }
    }

    /// Cycle to the next time window. Sets flag to trigger candle refetch.
    pub fn cycle_window(&mut self) {
        self.time_window = self.time_window.next();
        self.needs_candle_refresh = true;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.coins.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn toggle_selection(&mut self) {
        let current = self.checked[self.selected_index];
        if current {
            self.checked[self.selected_index] = false;
        } else {
            let selected_count = self.checked.iter().filter(|&&c| c).count();
            if selected_count < 3 {
                self.checked[self.selected_index] = true;
            }
        }
    }

    pub fn switch_view(&mut self) {
        // Mark notifications as read when leaving Notifications view
        if self.view == View::Notifications {
            self.notification_manager.mark_all_read();
        }

        let next_view = match self.view {
            View::Overview => View::Details,
            View::Details => View::Notifications,
            View::Notifications => View::News,
            View::News => View::Positions,
            View::Positions => View::Overview,
        };

        // Trigger positions refresh when entering Positions view
        if next_view == View::Positions {
            self.needs_positions_refresh = true;
        }

        self.view = next_view;
    }

    /// Request news refresh
    pub fn refresh_news(&mut self) {
        self.needs_news_refresh = true;
    }

    /// Select previous news article
    pub fn scroll_news_up(&mut self) {
        if self.news_selected > 0 {
            self.news_selected -= 1;
            self.news_content_scroll = 0; // Reset content scroll on selection change
        }
    }

    /// Select next news article
    pub fn scroll_news_down(&mut self) {
        if !self.news_articles.is_empty() && self.news_selected < self.news_articles.len() - 1 {
            self.news_selected += 1;
            self.news_content_scroll = 0; // Reset content scroll on selection change
        }
    }

    /// Scroll content up (for article body)
    pub fn scroll_content_up(&mut self) {
        if self.news_content_scroll > 0 {
            self.news_content_scroll -= 1;
        }
    }

    /// Scroll content down (for article body)
    pub fn scroll_content_down(&mut self) {
        self.news_content_scroll += 1;
    }

    /// Set news articles from API response
    pub fn set_news(&mut self, articles: Vec<NewsArticle>) {
        self.news_articles = articles;
        self.news_loading = false;
        self.news_selected = 0;
        self.news_content_scroll = 0;
    }

    /// Request positions refresh
    pub fn refresh_positions(&mut self) {
        self.needs_positions_refresh = true;
    }

    /// Navigate to previous position
    pub fn select_prev_position(&mut self) {
        if self.positions_selected > 0 {
            self.positions_selected -= 1;
        }
    }

    /// Navigate to next position
    pub fn select_next_position(&mut self) {
        if let Some(account) = &self.margin_account {
            if self.positions_selected < account.positions.len().saturating_sub(1) {
                self.positions_selected += 1;
            }
        }
    }

    /// Handle margin account update from API
    pub fn set_margin_account(&mut self, account: MarginAccount) {
        self.margin_account = Some(account);
        self.positions_loading = false;
        self.positions_selected = 0;
    }

    /// Get selected coin symbols for news filtering
    pub fn selected_symbols(&self) -> Vec<String> {
        self.checked
            .iter()
            .enumerate()
            .filter(|(_, &checked)| checked)
            .filter_map(|(i, _)| self.coins.get(i).map(|c| c.symbol.clone()))
            .collect()
    }

    /// Toggle the currently selected notification rule
    pub fn toggle_notification_rule(&mut self) {
        self.notification_manager.toggle_selected_rule();
    }

    /// Move notification rule selection up
    pub fn select_prev_rule(&mut self) {
        self.notification_manager.select_prev();
    }

    /// Move notification rule selection down
    pub fn select_next_rule(&mut self) {
        self.notification_manager.select_next();
    }

    pub fn selected_count(&self) -> usize {
        self.checked.iter().filter(|&&c| c).count()
    }

    /// Returns indices and references to selected (checked) coins
    pub fn selected_coins_with_index(&self) -> Vec<(usize, &CoinData)> {
        self.coins
            .iter()
            .enumerate()
            .filter(|(i, _)| self.checked.get(*i).copied().unwrap_or(false))
            .collect()
    }

    /// If no coins selected, return the currently highlighted coin
    pub fn active_coins(&self) -> Vec<(usize, &CoinData)> {
        let selected = self.selected_coins_with_index();
        if selected.is_empty() {
            vec![(self.selected_index, &self.coins[self.selected_index])]
        } else {
            selected
        }
    }

    /// Handle a price update from the WebSocket
    pub fn handle_update(&mut self, update: PriceUpdate) {
        match update {
            PriceUpdate::Ticker {
                symbol,
                price,
                change_24h,
                volume_24h_usd,
                volume_24h_base,
                high_24h,
                low_24h,
            } => {
                if let Some(coin) = self.coins.iter_mut().find(|c| c.symbol == symbol) {
                    // Update price, sparkline, and recalculate indicators
                    coin.update_price(price);

                    coin.change_24h = change_24h;
                    coin.volume_usd = volume_24h_usd;
                    coin.volume_base = volume_24h_base;
                    if high_24h > 0.0 {
                        coin.high_24h = high_24h;
                    }
                    if low_24h > 0.0 {
                        coin.low_24h = low_24h;
                    }
                }
            }
            PriceUpdate::Connected => {
                self.connection_status = ConnectionStatus::Connected;
            }
            PriceUpdate::Disconnected => {
                self.connection_status = ConnectionStatus::Disconnected;
            }
            PriceUpdate::Candles { symbol, candles } => {
                if let Some(coin) = self.coins.iter_mut().find(|c| c.symbol == symbol) {
                    coin.set_candles(candles);
                }
            }
            PriceUpdate::Error(_) => {
                // Could log the error or show it in UI
            }
            PriceUpdate::MarginPositions { account } => {
                self.set_margin_account(account);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        use crate::mock::generate_mock_coins;
        Self::new(generate_mock_coins(), "mock")
    }
}
