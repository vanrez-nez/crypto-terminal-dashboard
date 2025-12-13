# Phase 3: Implement Details View

## Goal
Create the Details view structure that displays 1-3 selected coins with price panels, chart areas, and indicator panels.

## Files to Create
- `src/views/details.rs`

## Files to Modify
- `src/views/mod.rs` - Add exports
- `src/main.rs` - Wire view and chart rendering

## Implementation Tasks

### Task 3.1: Update views/mod.rs
```rust
pub mod overview;
pub mod details;

pub use overview::build_overview_view;
pub use details::{build_details_view, ChartArea};
```

### Task 3.2: Create views/details.rs structure
```rust
//! Details view - price charts and indicators for selected coins

use dashboard_system::{panel, PanelBuilder, taffy};
use taffy::prelude::*;

use crate::app::{App, View, ChartType};
use crate::mock::CoinData;
use crate::widgets::{
    build_status_header,
    build_details_footer,
    build_price_panel,
    build_indicator_panel,
    GlTheme,
    PixelRect,
};

/// Represents a chart area that needs to be rendered separately
#[derive(Clone, Debug)]
pub struct ChartArea {
    pub coin_index: usize,
    pub bounds: PixelRect,  // Will be filled after layout
}

pub fn build_details_view(
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> (PanelBuilder, Vec<ChartArea>) {
    let selected_coins = app.selected_coins();
    let count = selected_coins.len().max(1);

    let mut chart_areas = Vec::new();

    // Build coin columns
    let columns: Vec<PanelBuilder> = selected_coins
        .iter()
        .enumerate()
        .map(|(i, (idx, coin))| {
            chart_areas.push(ChartArea {
                coin_index: *idx,
                bounds: PixelRect::new(0.0, 0.0, 0.0, 0.0), // Filled after layout
            });
            build_coin_column(coin, count, theme)
        })
        .collect();

    let view = panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
        .background(theme.background)
        // Header
        .child(build_status_header(
            app.view,
            &app.provider,
            app.time_window,
            app.chart_type,
            app.connection_status,
            theme,
        ))
        // Coin columns (horizontal layout)
        .child(
            panel()
                .flex_grow(1.0)
                .flex_direction(FlexDirection::Row)
                .gap(4.0)
                .children(columns)
        )
        // Footer
        .child(build_details_footer(theme));

    (view, chart_areas)
}

fn build_coin_column(
    coin: &CoinData,
    total_columns: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    panel()
        .proportion(1.0 / total_columns as f32)
        .flex_direction(FlexDirection::Column)
        .gap(4.0)
        // Price panel (fixed height ~80px)
        .child(build_price_panel(coin, theme))
        // Chart area (grows to fill, placeholder for ChartRenderer)
        .child(build_chart_placeholder(theme))
        // Indicator panel (fixed height ~120px)
        .child(build_indicator_panel(&coin.indicators, theme))
}

fn build_chart_placeholder(theme: &GlTheme) -> PanelBuilder {
    // This panel reserves space for chart rendering
    // The actual chart is drawn by ChartRenderer after layout
    panel()
        .flex_grow(1.0)
        .background(theme.background_panel)
        .border_solid(1.0, theme.border)
        .id("chart_area") // Mark for identification
}
```

### Task 3.3: Add selected_coins helper to App
In `src/app.rs`:
```rust
impl App {
    /// Returns indices and references to selected (checked) coins
    pub fn selected_coins(&self) -> Vec<(usize, &CoinData)> {
        self.coins
            .iter()
            .enumerate()
            .filter(|(i, _)| self.checked.get(*i).copied().unwrap_or(false))
            .collect()
    }

    /// If no coins selected, return the currently highlighted coin
    pub fn active_coins(&self) -> Vec<(usize, &CoinData)> {
        let selected = self.selected_coins();
        if selected.is_empty() {
            vec![(self.selected_index, &self.coins[self.selected_index])]
        } else {
            selected
        }
    }
}
```

### Task 3.4: Wire details view to main render loop
Update `build_current_view`:
```rust
use crate::views::{build_overview_view, build_details_view, ChartArea};

struct ViewResult {
    root: taffy::NodeId,
    chart_areas: Vec<ChartArea>,
}

fn build_current_view(
    tree: &mut LayoutTree,
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> ViewResult {
    match app.view {
        View::Overview => ViewResult {
            root: tree.build(build_overview_view(app, theme, width, height)),
            chart_areas: vec![],
        },
        View::Details => {
            let (panel, chart_areas) = build_details_view(app, theme, width, height);
            ViewResult {
                root: tree.build(panel),
                chart_areas,
            }
        }
    }
}
```

### Task 3.5: Extract chart bounds after layout
After layout computation, find chart panel positions:
```rust
fn extract_chart_bounds(
    tree: &LayoutTree,
    root: taffy::NodeId,
    chart_areas: &mut [ChartArea],
) {
    // Walk the tree to find panels marked as chart areas
    // and extract their computed positions

    // This requires dashboard-system to expose layout results
    // or we compute bounds based on known layout structure
}
```

Alternative: Calculate bounds from known layout:
```rust
fn calculate_chart_bounds(
    width: f32,
    height: f32,
    num_charts: usize,
    header_height: f32,    // ~50px
    footer_height: f32,    // ~36px
    price_panel_height: f32,   // ~80px
    indicator_height: f32,     // ~120px
    gap: f32,                  // 4px
) -> Vec<PixelRect> {
    let content_height = height - header_height - footer_height;
    let chart_height = content_height - price_panel_height - indicator_height - gap * 2.0;
    let chart_width = (width - gap * (num_charts - 1) as f32) / num_charts as f32;

    (0..num_charts)
        .map(|i| {
            let x = i as f32 * (chart_width + gap);
            let y = header_height + price_panel_height + gap;
            PixelRect::new(x, y, chart_width, chart_height)
        })
        .collect()
}
```

## Layout Structure (1 coin selected)
```
+------------------------------------------+
| Status Header (50px)                     |
| [DETAILS] BTC/USD | Binance | 1h | Line  |
+------------------------------------------+
| Price Panel (80px)                       |
| BTC $67,432.50  +2.34%  H:68102 L:65201 |
+------------------------------------------+
| Chart Area (flex grow)                   |
|  [Chart rendered by ChartRenderer]       |
|                                          |
+------------------------------------------+
| Indicator Panel (120px)                  |
| RSI: 62.5 | EMA-7: 67200 | MACD: +125   |
+------------------------------------------+
| Footer (36px)                            |
| [h/l] Scroll  [w] Window  [Tab] Overview |
+------------------------------------------+
```

## Layout Structure (3 coins selected)
```
+------------------------------------------+
| Status Header (50px)                     |
+------------+------------+----------------+
| BTC Panel  | ETH Panel  | SOL Panel      |
| Price      | Price      | Price          |
+------------+------------+----------------+
| BTC Chart  | ETH Chart  | SOL Chart      |
|            |            |                |
+------------+------------+----------------+
| BTC Indic  | ETH Indic  | SOL Indic      |
+------------+------------+----------------+
| Footer (36px)                            |
+------------------------------------------+
```

## Validation
- Details view renders with proper layout
- Multi-column layout works for 1, 2, and 3 coins
- Price panels show correct data
- Indicator panels show RSI, EMA, MACD values
- Chart area is visible (even if empty)
- Gap spacing is consistent

## Notes
- Actual chart rendering is in Phase 4
- Chart area is a placeholder panel for now
- Volume bars will be added in Phase 4
