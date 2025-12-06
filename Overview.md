# Crypto Terminal Dashboard

A terminal-based cryptocurrency dashboard built with Rust and ratatui, optimized for low-resource devices like Raspberry Pi B+.

---

## Overview

Two-view dashboard for monitoring cryptocurrency pairs with configurable technical indicators. Designed for minimal memory footprint and efficient terminal rendering.

---

## Installation

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For cross-compiling to Raspberry Pi (optional)
cargo install cross
rustup target add arm-unknown-linux-gnueabihf
```

### Build

```bash
git clone <repo>
cd crypto-dashboard

# Development
cargo run

# Release (native)
cargo build --release

# Release (cross-compile for Pi B+)
cross build --release --target arm-unknown-linux-gnueabihf
```

### Run

```bash
./target/release/crypto-dashboard --config config.json
```

---

## Configuration

### config.json

```json
{
  "api": {
    "source": "coingecko",
    "poll_interval_secs": 30,
    "timeout_secs": 10
  },
  "display": {
    "refresh_rate_ms": 100,
    "max_detail_pairs": 3
  },
  "pairs": [
    {
      "id": "bitcoin",
      "symbol": "BTC",
      "quote": "USD",
      "indicators": ["rsi", "ema", "sma", "volume"]
    },
    {
      "id": "ethereum",
      "symbol": "ETH",
      "quote": "USD",
      "indicators": ["rsi", "macd", "volume"]
    },
    {
      "id": "solana",
      "symbol": "SOL",
      "quote": "USD",
      "indicators": ["rsi", "ema"]
    }
  ],
  "indicators": {
    "rsi": {
      "period": 14
    },
    "ema": {
      "periods": [9, 21, 50]
    },
    "sma": {
      "periods": [20, 50, 200]
    },
    "macd": {
      "fast": 12,
      "slow": 26,
      "signal": 9
    }
  }
}
```

---

## Views

### View 1: Overview (All Coins)

Grid layout showing all configured pairs with summary statistics.

```
┌─ Crypto Dashboard ──────────────────────────────────────────────┐
│  [Tab: Overview]  [Details]                      ↻ 15s  ● Live  │
├─────────────────────────────────────────────────────────────────┤
│  [ ]  PAIR      PRICE         24H %     VOLUME        HIGH/LOW  │
│  ─────────────────────────────────────────────────────────────  │
│  [x]  BTC/USD   $67,432.10    +2.34%    $28.4B    $68k / $65k   │
│ >[x]  ETH/USD   $3,521.45     -0.82%    $14.2B    $3.6k / $3.4k │
│  [ ]  SOL/USD   $142.33       +5.21%    $2.1B     $145 / $135   │
│  [ ]  XRP/USD   $0.5234       +1.02%    $1.8B     $0.53 / $0.51 │
│  [ ]  ADA/USD   $0.4521       -0.34%    $890M     $0.46 / $0.44 │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│  Selected: 2/5  │  [Space] Toggle  [Enter] View Details  [q] Quit│
└─────────────────────────────────────────────────────────────────┘
```

**Columns:**
| Column | Description |
|--------|-------------|
| Checkbox | Selection state for detail view |
| Pair | Symbol/quote currency |
| Price | Current price with formatting |
| 24H % | Percentage change, color-coded |
| Volume | 24h trading volume |
| High/Low | 24h high and low |

**Navigation:**
| Key | Action |
|-----|--------|
| ↑/↓ | Move cursor |
| Space | Toggle selection (max 3) |
| Enter | Switch to Details view |
| Tab | Switch views |
| r | Force refresh |
| q | Quit |

---

### View 2: Details (Selected Coins)

Side-by-side detailed view of up to 3 selected pairs with configured indicators.

```
┌─ Crypto Dashboard ──────────────────────────────────────────────┐
│  [Overview]  [Tab: Details]                      ↻ 15s  ● Live  │
├─────────────────────────────────────────────────────────────────┤
│  BTC/USD                     │  ETH/USD                         │
│  ════════════════════════════│══════════════════════════════════│
│  Price:    $67,432.10        │  Price:    $3,521.45             │
│  24H:      +2.34% ▲          │  24H:      -0.82% ▼              │
│  Volume:   $28.4B            │  Volume:   $14.2B                │
│  High:     $68,102.00        │  High:     $3,612.30             │
│  Low:      $65,201.00        │  Low:      $3,480.10             │
│  ────────────────────────────│──────────────────────────────────│
│  RSI(14):      58.3 ●        │  RSI(14):      42.1 ●            │
│  EMA(9):   $67,102.00        │  MACD:     +12.4 ▲               │
│  EMA(21):  $66,430.00        │  Signal:   +8.2                  │
│  EMA(50):  $64,200.00        │  Hist:     +4.2 ▲                │
│  SMA(20):  $66,800.00        │                                  │
│  ────────────────────────────│──────────────────────────────────│
│  ▁▂▃▄▅▆▇█▇▆▅▆▇█▇▆▅▄▃▄▅▆▇    │  █▇▆▅▄▃▂▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅    │
│  └─── 24h price movement ───┘│  └─── 24h price movement ───────┘│
├─────────────────────────────────────────────────────────────────┤
│  [Tab] Overview  [←/→] Scroll pairs  [1-3] Focus pair  [q] Quit │
└─────────────────────────────────────────────────────────────────┘
```

**Indicator Display:**

Each pair shows indicators based on its `indicators` array in config:

| Indicator | Display |
|-----------|---------|
| rsi | RSI(period): value + overbought/oversold color |
| ema | EMA(period): price level for each configured period |
| sma | SMA(period): price level for each configured period |
| macd | MACD line, Signal line, Histogram |
| volume | 24h volume with change indicator |

**Navigation:**
| Key | Action |
|-----|--------|
| Tab | Switch to Overview |
| ←/→ | Cycle through pairs (if >3 selected) |
| 1/2/3 | Focus/expand specific panel |
| r | Force refresh |
| q | Quit |

---

## Project Structure

```
crypto-dashboard/
├── Cargo.toml
├── config.json
├── README.md
└── src/
    ├── main.rs              # Entry point, arg parsing, runtime setup
    ├── app.rs               # App state machine, event loop
    ├── config.rs            # Config parsing, validation
    ├── api/
    │   ├── mod.rs           # API trait, client factory
    │   ├── coingecko.rs     # CoinGecko implementation
    │   └── types.rs         # API response types
    ├── indicators/
    │   ├── mod.rs           # Indicator trait
    │   ├── rsi.rs           # RSI calculation
    │   ├── ema.rs           # EMA calculation
    │   ├── sma.rs           # SMA calculation
    │   └── macd.rs          # MACD calculation
    ├── ui/
    │   ├── mod.rs           # UI router
    │   ├── overview.rs      # Overview view rendering
    │   ├── details.rs       # Details view rendering
    │   └── widgets.rs       # Shared widgets (sparkline, indicators)
    └── events.rs            # Input handling, tick events
```

---

## Dependencies

```toml
[package]
name = "crypto-dashboard"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.28"
crossterm = "0.28"
tokio = { version = "1", features = ["rt", "macros", "time", "sync"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
```

---

## Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  API Fetch  │────▶│  Indicator  │────▶│   App State │
│  (async)    │     │  Calculate  │     │   Update    │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
       ┌───────────────────────────────────────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  UI Render  │────▶│  Terminal   │────▶│   Display   │
│  (ratatui)  │     │  (crossterm)│     │             │
└─────────────┘     └─────────────┘     └─────────────┘
```

**Event Loop:**

1. Poll for input events (non-blocking)
2. Poll for API updates (async channel)
3. Update state if dirty
4. Render frame if state changed
5. Sleep until next tick

---

## Technical Indicators

### RSI (Relative Strength Index)

```rust
pub struct RsiConfig {
    pub period: usize,  // default: 14
}

// Display: 0-100 scale
// Colors: <30 green (oversold), >70 red (overbought)
```

### EMA (Exponential Moving Average)

```rust
pub struct EmaConfig {
    pub periods: Vec<usize>,  // e.g., [9, 21, 50]
}

// Display: Price level for each period
// Colors: Above price = green, below = red
```

### SMA (Simple Moving Average)

```rust
pub struct SmaConfig {
    pub periods: Vec<usize>,  // e.g., [20, 50, 200]
}

// Display: Price level for each period
```

### MACD (Moving Average Convergence Divergence)

```rust
pub struct MacdConfig {
    pub fast: usize,    // default: 12
    pub slow: usize,    // default: 26
    pub signal: usize,  // default: 9
}

// Display: MACD line, Signal line, Histogram
// Colors: Histogram positive = green, negative = red
```

---

## API Integration

### CoinGecko (Default)

Endpoints used:
- `/simple/price` - Current prices
- `/coins/{id}/market_chart` - Historical data for indicators

Rate limits: 10-30 calls/min (free tier)

### Response Caching

- Cache last successful response
- Serve stale data on network failure
- Show staleness indicator in UI

---

## Error Handling

| Error | UI Behavior |
|-------|-------------|
| Network timeout | Show cached data + "Stale" indicator |
| API rate limit | Back off, show countdown |
| Invalid config | Exit with clear error message |
| Parse error | Log, continue with partial data |

---

## Milestones

### v0.1 - MVP
- [ ] Config loading
- [ ] CoinGecko API integration
- [ ] Overview view with basic stats
- [ ] Keyboard navigation

### v0.2 - Details View
- [ ] Detail view layout
- [ ] RSI indicator
- [ ] EMA indicator
- [ ] Sparkline charts

### v0.3 - Polish
- [ ] All indicators (SMA, MACD)
- [ ] Error handling + staleness
- [ ] Config validation
- [ ] Cross-compile testing on Pi

### v0.4 - Extras
- [ ] Multiple API sources
- [ ] Alerts (optional)
- [ ] Local price history cache

---

## Usage Examples

```bash
# Run with default config
./crypto-dashboard

# Run with custom config
./crypto-dashboard --config ~/my-config.json

# Enable debug logging
RUST_LOG=debug ./crypto-dashboard
```

---

## License

MIT