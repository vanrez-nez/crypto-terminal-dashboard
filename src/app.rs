use crate::api::PriceUpdate;
use crate::mock::CoinData;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Overview,
    Details,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartType {
    Line,
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

    /// Returns the Coinbase API granularity in seconds for this time window
    pub fn granularity(&self) -> u32 {
        match self {
            TimeWindow::Min15 => 900,   // 15 minutes
            TimeWindow::Hour1 => 3600,  // 1 hour
            TimeWindow::Hour4 => 21600, // 6 hours (closest to 4h available)
            TimeWindow::Day1 => 86400,  // 1 day
        }
    }

    /// Returns the window duration in seconds (for filtering high/low range)
    pub fn duration_secs(&self) -> i64 {
        match self {
            TimeWindow::Min15 => 900,    // 15 minutes
            TimeWindow::Hour1 => 3600,   // 1 hour
            TimeWindow::Hour4 => 14400,  // 4 hours
            TimeWindow::Day1 => 86400,   // 24 hours
        }
    }
}

pub struct App {
    pub view: View,
    pub coins: Vec<CoinData>,
    pub selected_index: usize,
    pub checked: Vec<bool>,
    pub running: bool,
    pub theme: Theme,
    pub connection_status: ConnectionStatus,
    pub provider: String,
    pub time_window: TimeWindow,
    pub needs_candle_refresh: bool,
    pub chart_type: ChartType,
    pub candle_scroll_offset: isize,
}

impl App {
    pub fn new(coins: Vec<CoinData>, theme: Theme, provider: &str) -> Self {
        let coin_count = coins.len();
        let use_mock = provider == "mock";
        Self {
            view: View::Overview,
            coins,
            selected_index: 0,
            checked: vec![false; coin_count],
            running: true,
            theme,
            connection_status: if use_mock {
                ConnectionStatus::Mock
            } else {
                ConnectionStatus::Connecting
            },
            provider: provider.to_string(),
            time_window: TimeWindow::Hour1,
            needs_candle_refresh: true, // Fetch candles on startup
            chart_type: ChartType::Line,
            candle_scroll_offset: 0,
        }
    }

    /// Cycle between Line and Candlestick chart types
    pub fn cycle_chart_type(&mut self) {
        self.chart_type = match self.chart_type {
            ChartType::Line => ChartType::Candlestick,
            ChartType::Candlestick => ChartType::Line,
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
        self.view = match self.view {
            View::Overview => View::Details,
            View::Details => View::Overview,
        };
    }

    pub fn selected_count(&self) -> usize {
        self.checked.iter().filter(|&&c| c).count()
    }

    pub fn selected_coins(&self) -> Vec<&CoinData> {
        self.coins
            .iter()
            .zip(self.checked.iter())
            .filter(|(_, &checked)| checked)
            .map(|(coin, _)| coin)
            .collect()
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
        }
    }
}

impl Default for App {
    fn default() -> Self {
        use crate::mock::generate_mock_coins;
        Self::new(generate_mock_coins(), Theme::default(), "mock")
    }
}
