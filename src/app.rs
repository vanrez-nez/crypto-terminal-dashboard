use crate::mock::{generate_mock_coins, CoinData};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Overview,
    Details,
}

pub struct App {
    pub view: View,
    pub coins: Vec<CoinData>,
    pub selected_index: usize,
    pub checked: Vec<bool>,
    pub running: bool,
    pub theme: Theme,
}

impl App {
    pub fn new(theme: Theme) -> Self {
        let coins = generate_mock_coins();
        let coin_count = coins.len();
        Self {
            view: View::Overview,
            coins,
            selected_index: 0,
            checked: vec![false; coin_count],
            running: true,
            theme,
        }
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
}

impl Default for App {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}
