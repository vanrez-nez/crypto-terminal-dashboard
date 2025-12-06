# Crypto Dashboard Frontend Plan (Mock Data Only)

## Scope
Frontend-only implementation of the crypto terminal dashboard using ratatui. All data will be mocked - no API integration.

---

## Files to Create

```
src/
├── main.rs              # Entry point, terminal setup, event loop
├── app.rs               # App state, view switching
├── mock.rs              # Mock data generators
├── ui/
│   ├── mod.rs           # UI router
│   ├── overview.rs      # Overview view (all coins table)
│   ├── details.rs       # Details view (selected coins)
│   └── widgets.rs       # Sparkline, indicator widgets
└── events.rs            # Keyboard input handling
```

---

## Step 1: Dependencies & Project Setup

Update `Cargo.toml`:
```toml
[dependencies]
ratatui = "0.28"
crossterm = "0.28"
```

---

## Step 2: Mock Data (`src/mock.rs`)

Create static mock data structures:
- `CoinData` struct: symbol, price, 24h_change, volume, high, low
- `IndicatorData` struct: RSI, EMA values, MACD values
- `generate_mock_coins()` - returns Vec of 5 coins (BTC, ETH, SOL, XRP, ADA)
- `generate_sparkline_data()` - returns Vec<u64> for price chart

---

## Step 3: App State (`src/app.rs`)

```rust
pub enum View { Overview, Details }
pub enum AppAction { None, Quit }

pub struct App {
    pub view: View,
    pub coins: Vec<CoinData>,
    pub selected_index: usize,      // Cursor position in overview
    pub checked: Vec<bool>,         // Selection checkboxes (max 3)
    pub running: bool,
}
```

Methods:
- `new()` - Initialize with mock data
- `toggle_selection()` - Toggle checkbox (enforce max 3)
- `switch_view()` - Toggle between Overview/Details
- `move_cursor(delta)` - Navigate up/down

---

## Step 4: Event Handling (`src/events.rs`)

Handle keyboard input:
| Key | Action |
|-----|--------|
| q | Quit |
| Up/Down (j/k) | Move cursor |
| Space | Toggle selection |
| Enter/Tab | Switch view |
| r | (no-op for now, future refresh) |

---

## Step 5: Overview View (`src/ui/overview.rs`)

Render the table view:
- Header with tabs showing current view
- Table with columns: checkbox, pair, price, 24h%, volume, high/low
- Highlighted row for cursor
- Footer with key hints
- Color coding: green for positive %, red for negative

---

## Step 6: Details View (`src/ui/details.rs`)

Render side-by-side panels for selected coins (up to 3):
- Split layout based on selection count (1, 2, or 3 columns)
- Each panel shows:
  - Price info (price, 24h change, volume, high/low)
  - Indicators section (RSI, EMA, SMA, MACD based on coin config)
  - Sparkline chart at bottom
- Footer with navigation hints

---

## Step 7: Widgets (`src/ui/widgets.rs`)

Reusable widget functions:
- `render_sparkline()` - Price movement mini-chart using block chars
- `render_indicator_rsi()` - RSI with color (green <30, red >70)
- `render_indicator_ema()` - EMA values list
- `render_indicator_macd()` - MACD line, signal, histogram

---

## Step 8: Main Entry Point (`src/main.rs`)

```rust
fn main() -> Result<()> {
    // 1. Setup terminal (enable raw mode, alternate screen)
    // 2. Create App with mock data
    // 3. Event loop:
    //    - Poll for keyboard events
    //    - Update app state
    //    - Render current view
    // 4. Restore terminal on exit
}
```

---

## Implementation Order

1. **Step 1**: Add dependencies
2. **Step 2**: Create mock data module
3. **Step 3**: Create app state module
4. **Step 4**: Create events module
5. **Step 5**: Create overview view
6. **Step 8**: Wire up main.rs (can test overview at this point)
7. **Step 7**: Create widgets module
8. **Step 6**: Create details view
9. **Final**: Polish and test navigation between views

---

## Mock Data Values

```
BTC/USD: $67,432.10, +2.34%, Vol $28.4B, H $68,102, L $65,201
ETH/USD: $3,521.45, -0.82%, Vol $14.2B, H $3,612, L $3,480
SOL/USD: $142.33, +5.21%, Vol $2.1B, H $145, L $135
XRP/USD: $0.5234, +1.02%, Vol $1.8B, H $0.53, L $0.51
ADA/USD: $0.4521, -0.34%, Vol $890M, H $0.46, L $0.44
```

Indicators (mock):
- RSI: 58.3 (BTC), 42.1 (ETH), 65.2 (SOL), 48.7 (XRP), 51.2 (ADA)
- EMA(9,21,50): Derived from price with small offsets
- MACD: +12.4/+8.2/+4.2 (BTC), -5.1/-3.2/-1.9 (ETH), etc.
