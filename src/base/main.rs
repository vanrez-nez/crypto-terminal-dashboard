mod drm_display;
mod focus;
mod font_atlas;
mod input;
mod layout;
mod renderer;
mod scrollable;
mod text_renderer;
mod view;
mod widgets;

use drm_display::Display;
use focus::FocusManager;
use font_atlas::{FontAtlas, GlyphInfo};
use input::{KeyEvent, KeyboardInput};
use layout::{panel, HAlign, LayoutTree, VAlign};
use renderer::{render, RectRenderer, ScissorStack};
use scrollable::ScrollStates;
use text_renderer::TextRenderer;
use view::{View, ViewManager};
use widgets::{generate_mock_logs, generate_mock_processes, LogViewer, ProcessList, StatCard};

use std::time::Instant;
use taffy::prelude::*;

const FONT_DATA: &[u8] = include_bytes!("../assets/Roboto-Medium.ttf");
const FONT_SIZE: f32 = 24.0;

/// Dashboard state
struct DashboardState {
    // System stats (mocked)
    cpu_percent: f32,
    ram_percent: f32,
    disk_percent: f32,
    net_up: f32,
    net_down: f32,

    // Process list
    process_list: ProcessList,

    // Log viewer
    log_viewer: LogViewer,

    // Animation
    frame_count: u64,

    // Glyph debug
    current_glyph_index: usize,
}

/// Characters available for glyph debug (ASCII printable)
const GLYPH_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()[]{}|;:',.<>/?`~-_=+\"\\";

impl DashboardState {
    fn next_glyph(&mut self) {
        self.current_glyph_index = (self.current_glyph_index + 1) % GLYPH_CHARS.len();
    }

    fn current_glyph(&self) -> char {
        GLYPH_CHARS
            .chars()
            .nth(self.current_glyph_index)
            .unwrap_or('A')
    }
}

impl DashboardState {
    fn new() -> Self {
        Self {
            cpu_percent: 45.0,
            ram_percent: 72.0,
            disk_percent: 28.0,
            net_up: 1.2,
            net_down: 0.34,
            process_list: ProcessList::new("processes").with_processes(generate_mock_processes()),
            log_viewer: LogViewer::new("logs").with_entries(generate_mock_logs()),
            frame_count: 0,
            current_glyph_index: 0,
        }
    }

    fn update(&mut self) {
        self.frame_count += 1;

        // Simulate some variation in stats
        let t = (self.frame_count as f32 * 0.02).sin();
        self.cpu_percent = (45.0 + t * 15.0).clamp(5.0, 95.0);
        self.ram_percent = (72.0 + (t * 0.7).sin() * 8.0).clamp(20.0, 95.0);

        // Occasionally add log entries
        if self.frame_count % 300 == 0 {
            use widgets::log_viewer::{LogEntry, LogLevel};
            let levels = [LogLevel::Info, LogLevel::Debug, LogLevel::Warn];
            let level = levels[(self.frame_count / 300) as usize % 3];
            let time = format!(
                "12:{:02}:{:02}",
                30 + (self.frame_count / 3600) % 30,
                (self.frame_count / 60) % 60
            );
            self.log_viewer.add_entry(LogEntry::new(
                time,
                level,
                format!("Periodic update #{}", self.frame_count / 300),
            ));
        }
    }
}

fn main() {
    println!("Dashboard System");
    println!("================");
    println!("Controls:");
    println!("  Left/Right       - Switch between views");
    println!("  1-4              - Jump to view directly");
    println!("  Tab / Shift+Tab  - Switch between panels");
    println!("  Up/Down arrows   - Scroll content");
    println!("  Page Up/Down     - Fast scroll");
    println!("  Home/End         - Scroll to top/bottom");
    println!("  ESC              - Exit");
    println!();

    // Initialize display
    let mut display = match Display::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to initialize display: {}", e);
            return;
        }
    };

    let screen_width = display.width;
    let screen_height = display.height;

    // Create font atlas
    let atlas = match FontAtlas::new(&display.gl, FONT_DATA, FONT_SIZE) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to create font atlas: {}", e);
            return;
        }
    };

    // Create renderers
    let mut text_renderer = match TextRenderer::new(&display.gl) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create text renderer: {}", e);
            return;
        }
    };

    let mut rect_renderer = match RectRenderer::new(&display.gl) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create rect renderer: {}", e);
            return;
        }
    };

    let mut scissor_stack = ScissorStack::new(screen_height);

    // Initialize input and focus
    let mut keyboard = KeyboardInput::new();
    let mut focus = FocusManager::new();
    let mut view_manager = ViewManager::new();

    // Set initial focus order based on view
    focus.set_focus_order(view_manager.focus_order());

    // Initialize scroll states
    let mut scroll_states = ScrollStates::new();

    // Initialize dashboard state
    let mut state = DashboardState::new();

    // FPS tracking
    let mut fps_counter = 0u32;
    let mut fps_timer = Instant::now();
    let mut current_fps = 0.0f32;

    println!("Dashboard started...");

    loop {
        // Handle input
        for event in keyboard.poll_events() {
            match event {
                KeyEvent::Escape => {
                    println!("\nESC pressed, exiting...");
                    std::process::exit(0);
                }
                // View switching with Left/Right
                KeyEvent::Left => {
                    view_manager.previous();
                }
                KeyEvent::Right => {
                    view_manager.next();
                }
                // Direct view access with number keys
                KeyEvent::Num1 => {
                    view_manager.set(View::Overview);
                }
                KeyEvent::Num2 => {
                    view_manager.set(View::Stats);
                }
                KeyEvent::Num3 => {
                    view_manager.set(View::Processes);
                }
                KeyEvent::Num4 => {
                    view_manager.set(View::Logs);
                }
                KeyEvent::Num5 => {
                    view_manager.set(View::GlyphDebug);
                }
                KeyEvent::Space => {
                    // In glyph debug view, cycle to next glyph
                    if view_manager.current() == View::GlyphDebug {
                        state.next_glyph();
                    }
                }
                KeyEvent::Tab => {
                    focus.next();
                }
                KeyEvent::ShiftTab => {
                    focus.previous();
                }
                KeyEvent::Up => {
                    if let Some(id) = focus.current() {
                        if let Some(scroll) = scroll_states.get_mut(id) {
                            scroll.scroll_up(1.0);
                        }
                        // Also handle process selection
                        if id == "processes" || id == "process_list" {
                            state.process_list.select_prev();
                        }
                    }
                }
                KeyEvent::Down => {
                    if let Some(id) = focus.current() {
                        if let Some(scroll) = scroll_states.get_mut(id) {
                            scroll.scroll_down(1.0);
                        }
                        // Also handle process selection
                        if id == "processes" || id == "process_list" {
                            state.process_list.select_next();
                        }
                    }
                }
                KeyEvent::PageUp => {
                    if let Some(id) = focus.current() {
                        if let Some(scroll) = scroll_states.get_mut(id) {
                            scroll.page_up();
                        }
                    }
                }
                KeyEvent::PageDown => {
                    if let Some(id) = focus.current() {
                        if let Some(scroll) = scroll_states.get_mut(id) {
                            scroll.page_down();
                        }
                    }
                }
                KeyEvent::Home => {
                    if let Some(id) = focus.current() {
                        if let Some(scroll) = scroll_states.get_mut(id) {
                            scroll.scroll_to_top();
                        }
                    }
                }
                KeyEvent::End => {
                    if let Some(id) = focus.current() {
                        if let Some(scroll) = scroll_states.get_mut(id) {
                            scroll.scroll_to_bottom();
                        }
                    }
                }
                KeyEvent::Enter => {
                    // Could be used for selection confirmation
                }
            }
        }

        // Update focus order when view changes
        if view_manager.changed() {
            focus.set_focus_order(view_manager.focus_order());
            view_manager.acknowledge_change();
        }

        // Update state
        state.update();

        // Update scroll states with content heights
        let proc_scroll = scroll_states.get_or_create("processes", screen_height as f32 * 0.4);
        proc_scroll.set_content_height(state.process_list.content_height());

        let log_scroll = scroll_states.get_or_create("logs", screen_height as f32 * 0.25);
        log_scroll.set_content_height(state.log_viewer.content_height());

        // Build layout
        let mut tree = LayoutTree::new();

        // Get current glyph info for debug view
        let glyph_char = state.current_glyph();
        let glyph_info = atlas.get_glyph(glyph_char).copied();

        let root = build_dashboard(
            &mut tree,
            &state,
            &focus,
            &view_manager,
            &scroll_states,
            screen_width as f32,
            screen_height as f32,
            current_fps,
            glyph_char,
            glyph_info,
            atlas.line_height,
        );
        tree.compute(root, screen_width as f32, screen_height as f32);

        // Clear screen
        display.clear(0.04, 0.04, 0.06, 1.0);

        // Render
        render(
            &display.gl,
            &tree,
            root,
            &mut rect_renderer,
            &mut text_renderer,
            &atlas,
            &mut scissor_stack,
            &focus,
            screen_width,
            screen_height,
        );

        // Swap buffers
        if let Err(e) = display.swap_buffers() {
            eprintln!("Swap buffers error: {}", e);
            break;
        }

        // Update FPS
        fps_counter += 1;
        if fps_timer.elapsed().as_secs_f32() >= 1.0 {
            current_fps = fps_counter as f32 / fps_timer.elapsed().as_secs_f32();
            fps_counter = 0;
            fps_timer = Instant::now();
        }
    }

    std::process::exit(0);
}

fn build_dashboard(
    tree: &mut LayoutTree,
    state: &DashboardState,
    focus: &FocusManager,
    view_manager: &ViewManager,
    scroll_states: &ScrollStates,
    width: f32,
    height: f32,
    fps: f32,
    glyph_char: char,
    glyph_info: Option<GlyphInfo>,
    line_height: f32,
) -> NodeId {
    // Main container
    panel()
        .size(width, height)
        .flex_direction(FlexDirection::Column)
        .padding_all(10.0)
        .gap(10.0)
        // Tab bar
        .child(build_tab_bar(view_manager, width - 20.0))
        // Main content area - view specific
        .child(match view_manager.current() {
            View::Overview => build_overview_content(state, focus, scroll_states),
            View::Stats => build_stats_content(state, focus),
            View::Processes => build_processes_content(state, focus, scroll_states),
            View::Logs => build_logs_content(state, focus, scroll_states),
            View::GlyphDebug => {
                build_glyph_debug_content(glyph_char, glyph_info, line_height, width - 20.0)
            }
        })
        // Status bar
        .child(
            panel()
                .width(percent(1.0))
                .height(length(30.0))
                .background([0.08, 0.08, 0.1, 1.0])
                .padding(5.0, 10.0, 5.0, 10.0)
                .flex_direction(FlexDirection::Row)
                .justify_content(JustifyContent::SpaceBetween)
                .align_items(AlignItems::Center)
                .child(
                    panel()
                        .width(length(400.0))
                        .height(percent(1.0))
                        .text(
                            format!(
                                "View: {} | Focus: {} | ←→ switch view | Tab switch panel",
                                view_manager.current().label(),
                                focus.current().unwrap_or("none")
                            ),
                            [0.5, 0.55, 0.6, 1.0],
                            0.75,
                        )
                        .text_align(HAlign::Left, VAlign::Center),
                )
                .child(
                    panel()
                        .width(length(150.0))
                        .height(percent(1.0))
                        .text(
                            format!("FPS: {:.0} | ESC exit", fps),
                            [0.5, 0.55, 0.6, 1.0],
                            0.75,
                        )
                        .text_align(HAlign::Right, VAlign::Center),
                ),
        )
        .build(tree)
}

/// Build the tab bar showing all views
fn build_tab_bar(view_manager: &ViewManager, width: f32) -> layout::PanelBuilder {
    let current = view_manager.current();

    let mut tabs = panel()
        .width(length(width))
        .height(length(40.0))
        .background([0.08, 0.08, 0.1, 1.0])
        .flex_direction(FlexDirection::Row)
        .padding(5.0, 10.0, 5.0, 10.0)
        .gap(8.0)
        .align_items(AlignItems::Center);

    for view in View::all() {
        let is_active = *view == current;
        let bg = if is_active {
            [0.25, 0.35, 0.5, 1.0]
        } else {
            [0.12, 0.12, 0.15, 1.0]
        };
        let text_color = if is_active {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            [0.6, 0.65, 0.7, 1.0]
        };
        let border_color = if is_active {
            [0.4, 0.5, 0.7, 1.0]
        } else {
            [0.2, 0.22, 0.25, 1.0]
        };

        let label = format!("[{}] {}", view.index() + 1, view.label());

        tabs = tabs.child(
            panel()
                .width(length(120.0))
                .height(length(28.0))
                .padding(4.0, 8.0, 4.0, 8.0)
                .background(bg)
                .border_solid(1.0, border_color)
                .text(label, text_color, 0.7)
                .text_align(HAlign::Center, VAlign::Center),
        );
    }

    tabs
}

/// Overview view - Combined stats + processes + logs
fn build_overview_content(
    state: &DashboardState,
    focus: &FocusManager,
    scroll_states: &ScrollStates,
) -> layout::PanelBuilder {
    panel()
        .width(percent(1.0))
        .height(percent(1.0)) // Explicit height to constrain children
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Row)
        .gap(10.0)
        // Left sidebar - Stats
        .child(build_stats_sidebar(state, focus))
        // Right side - Process list and logs
        .child(
            panel()
                .flex_grow(1.0)
                .height(percent(1.0))
                .flex_direction(FlexDirection::Column)
                .gap(10.0)
                // Process list (top) - 60% of available space
                // DESIGN: proportion() handles gaps automatically via flex-grow ratios
                .child(
                    panel()
                        .width(percent(1.0))
                        .proportion(60.0) // 60:40 split with log viewer
                        .overflow_scroll() // Children don't affect container size
                        .child(state.process_list.build(
                            focus.is_focused("processes"),
                            scroll_states.offset("processes"),
                        )),
                )
                // Log viewer (bottom) - 40% of available space
                .child(
                    panel()
                        .width(percent(1.0))
                        .proportion(40.0) // 60:40 split with process list
                        .overflow_scroll() // Children don't affect container size
                        .child(
                            state
                                .log_viewer
                                .build(focus.is_focused("logs"), scroll_states.offset("logs")),
                        ),
                ),
        )
}

/// Stats view - Full screen grid of detailed stats
fn build_stats_content(state: &DashboardState, focus: &FocusManager) -> layout::PanelBuilder {
    panel()
        .width(percent(1.0))
        .height(percent(1.0)) // Explicit height for child percentages to work
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Column)
        .gap(15.0)
        .padding_all(10.0)
        // Row 1: CPU and RAM
        .child(
            panel()
                .width(percent(1.0))
                .height(percent(1.0)) // Explicit height for child percentages
                .flex_grow(1.0)
                .flex_direction(FlexDirection::Row)
                .gap(15.0)
                .child(build_large_stat_card(
                    "CPU Usage",
                    state.cpu_percent,
                    "cpu_detail",
                    focus,
                ))
                .child(build_large_stat_card(
                    "RAM Usage",
                    state.ram_percent,
                    "ram_detail",
                    focus,
                )),
        )
        // Row 2: Disk and Network
        .child(
            panel()
                .width(percent(1.0))
                .height(percent(1.0)) // Explicit height for child percentages
                .flex_grow(1.0)
                .flex_direction(FlexDirection::Row)
                .gap(15.0)
                .child(build_large_stat_card(
                    "Disk Usage",
                    state.disk_percent,
                    "disk_detail",
                    focus,
                ))
                .child(build_network_detail_card(state, focus)),
        )
}

/// Processes view - Full screen process list
fn build_processes_content(
    state: &DashboardState,
    focus: &FocusManager,
    scroll_states: &ScrollStates,
) -> layout::PanelBuilder {
    panel()
        .width(percent(1.0))
        .flex_grow(1.0)
        .child(state.process_list.build(
            focus.is_focused("process_list"),
            scroll_states.offset("process_list"),
        ))
}

/// Logs view - Full screen log viewer
fn build_logs_content(
    state: &DashboardState,
    focus: &FocusManager,
    scroll_states: &ScrollStates,
) -> layout::PanelBuilder {
    // Wrapper uses only flex_grow (not height(percent)) to avoid conflicts.
    // The widget inside uses flex_grow + flex_basis(0) to fill this space.
    panel()
        .width(percent(1.0))
        .flex_grow(1.0)
        .child(state.log_viewer.build(
            focus.is_focused("log_viewer"),
            scroll_states.offset("log_viewer"),
        ))
}

/// Build a large stat card for the Stats view
fn build_large_stat_card(
    label: &str,
    value: f32,
    id: &str,
    focus: &FocusManager,
) -> layout::PanelBuilder {
    let is_focused = focus.is_focused(id);
    let border_color = if is_focused {
        [1.0, 0.8, 0.2, 1.0]
    } else {
        [0.3, 0.35, 0.4, 1.0]
    };

    let value_color = if value > 80.0 {
        [0.9, 0.3, 0.3, 1.0]
    } else if value > 60.0 {
        [0.9, 0.7, 0.2, 1.0]
    } else {
        [0.3, 0.8, 0.4, 1.0]
    };

    panel()
        .flex_grow(1.0)
        .height(percent(1.0))
        .background([0.1, 0.12, 0.15, 1.0])
        .border_solid(2.0, border_color)
        .padding_all(20.0)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .gap(15.0)
        .focusable(id)
        .focus_border([1.0, 0.8, 0.2, 1.0])
        .child(
            panel()
                .width(percent(1.0))
                .height(length(40.0))
                .text(label, [0.7, 0.75, 0.8, 1.0], 1.2)
                .text_align(HAlign::Center, VAlign::Center),
        )
        .child(
            panel()
                .width(percent(1.0))
                .height(length(60.0))
                .text(format!("{:.1}%", value), value_color, 2.0)
                .text_align(HAlign::Center, VAlign::Center),
        )
        .child(
            panel()
                .width(percent(0.8))
                .height(length(20.0))
                .background([0.15, 0.15, 0.18, 1.0])
                .child(
                    panel()
                        .width(percent(value / 100.0))
                        .height(percent(1.0))
                        .background(value_color),
                ),
        )
}

/// Build network detail card for Stats view
fn build_network_detail_card(state: &DashboardState, focus: &FocusManager) -> layout::PanelBuilder {
    let is_focused = focus.is_focused("network_detail");
    let border_color = if is_focused {
        [1.0, 0.8, 0.2, 1.0]
    } else {
        [0.3, 0.35, 0.4, 1.0]
    };

    panel()
        .flex_grow(1.0)
        .height(percent(1.0))
        .background([0.1, 0.12, 0.15, 1.0])
        .border_solid(2.0, border_color)
        .padding_all(20.0)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .gap(15.0)
        .focusable("network_detail")
        .focus_border([1.0, 0.8, 0.2, 1.0])
        .child(
            panel()
                .width(percent(1.0))
                .height(length(40.0))
                .text("Network", [0.7, 0.75, 0.8, 1.0], 1.2)
                .text_align(HAlign::Center, VAlign::Center),
        )
        .child(
            panel()
                .width(percent(1.0))
                .height(length(50.0))
                .text(
                    format!("↑ {:.2} MB/s", state.net_up),
                    [0.4, 0.8, 0.5, 1.0],
                    1.5,
                )
                .text_align(HAlign::Center, VAlign::Center),
        )
        .child(
            panel()
                .width(percent(1.0))
                .height(length(50.0))
                .text(
                    format!("↓ {:.2} MB/s", state.net_down),
                    [0.5, 0.7, 0.9, 1.0],
                    1.5,
                )
                .text_align(HAlign::Center, VAlign::Center),
        )
}

fn build_stats_sidebar(state: &DashboardState, focus: &FocusManager) -> layout::PanelBuilder {
    let is_focused = focus.is_focused("stats");

    panel()
        .width(length(180.0))
        .height(percent(1.0))
        .background([0.08, 0.08, 0.1, 1.0])
        .border_solid(
            2.0,
            if is_focused {
                [1.0, 0.8, 0.2, 1.0]
            } else {
                [0.25, 0.28, 0.32, 1.0]
            },
        )
        .padding_all(10.0)
        .flex_direction(FlexDirection::Column)
        .gap(10.0)
        .focusable("stats")
        .focus_border([1.0, 0.8, 0.2, 1.0])
        // Title
        .child(
            panel()
                .width(percent(1.0))
                .height(length(25.0))
                .text("System Stats", [0.7, 0.75, 0.8, 1.0], 0.9)
                .text_align(HAlign::Center, VAlign::Center),
        )
        // CPU
        .child(StatCard::with_percent("CPU", state.cpu_percent, "cpu").build(false))
        // RAM
        .child(StatCard::with_percent("RAM", state.ram_percent, "ram").build(false))
        // Disk
        .child(StatCard::with_percent("Disk", state.disk_percent, "disk").build(false))
        // Network
        .child(
            panel()
                .width(percent(1.0))
                .height(length(70.0))
                .background([0.1, 0.12, 0.15, 1.0])
                .border_solid(1.0, [0.3, 0.35, 0.4, 1.0])
                .padding_all(8.0)
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Center)
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(18.0))
                        .text("Network", [0.6, 0.65, 0.7, 1.0], 0.8)
                        .text_align(HAlign::Center, VAlign::Center),
                )
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(20.0))
                        .text(
                            format!("↑ {:.1} MB/s", state.net_up),
                            [0.4, 0.8, 0.5, 1.0],
                            0.85,
                        )
                        .text_align(HAlign::Center, VAlign::Center),
                )
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(20.0))
                        .text(
                            format!("↓ {:.2} MB/s", state.net_down),
                            [0.5, 0.7, 0.9, 1.0],
                            0.85,
                        )
                        .text_align(HAlign::Center, VAlign::Center),
                ),
        )
}

/// Glyph debug view - Shows glyph metrics with visual lines
fn build_glyph_debug_content(
    glyph_char: char,
    glyph_info: Option<GlyphInfo>,
    line_height: f32,
    width: f32,
) -> layout::PanelBuilder {
    let scale = 6.0; // Large scale for visibility

    // Get glyph metrics or defaults
    let (size_w, size_h, bearing_x, bearing_y, advance) = match glyph_info {
        Some(g) => (g.size.0, g.size.1, g.bearing.0, g.bearing.1, g.advance),
        None => (10.0, 20.0, 0.0, 0.0, 10.0),
    };

    // Scaled values for display
    let scaled_w = size_w * scale;
    let scaled_h = size_h * scale;
    let scaled_bearing_x = bearing_x * scale;
    let scaled_bearing_y = bearing_y * scale;
    let scaled_advance = advance * scale;
    let scaled_line_height = line_height * scale;

    // Display area dimensions
    let display_height = 350.0;
    let baseline_y = display_height * 0.6; // Baseline at 60% from top

    // Glyph positioning (relative to baseline)
    // In draw_text: y0 = y - (size + bearing), y1 = y0 + size
    // So glyph top = baseline - (size + bearing), glyph bottom = baseline - bearing
    let glyph_top = baseline_y - (scaled_h + scaled_bearing_y);
    let glyph_bottom = baseline_y - scaled_bearing_y;
    let glyph_left = 200.0 + scaled_bearing_x;

    // Colors for measurement lines
    let baseline_color = [1.0, 0.3, 0.3, 1.0]; // Red - baseline
    let top_color = [0.3, 1.0, 0.3, 1.0]; // Green - glyph top
    let bottom_color = [0.3, 0.6, 1.0, 1.0]; // Blue - glyph bottom
    let center_color = [1.0, 1.0, 0.3, 1.0]; // Yellow - center
    let advance_color = [1.0, 0.5, 1.0, 1.0]; // Magenta - advance

    panel()
        .width(length(width))
        .flex_grow(1.0)
        .background([0.08, 0.08, 0.1, 1.0])
        .border_solid(1.0, [0.3, 0.35, 0.4, 1.0])
        .padding_all(15.0)
        .flex_direction(FlexDirection::Row)
        .gap(20.0)
        // Left side - Glyph display with measurement lines
        .child(
            panel()
                .width(length(500.0))
                .height(length(display_height))
                .background([0.05, 0.05, 0.07, 1.0])
                .border_solid(1.0, [0.25, 0.28, 0.32, 1.0])
                // Large glyph character
                .child(
                    panel()
                        .width(length(scaled_w.max(50.0)))
                        .height(length(scaled_h.max(50.0)))
                        .absolute(glyph_left, glyph_top)
                        .text(glyph_char.to_string(), [1.0, 1.0, 1.0, 1.0], scale)
                        .text_align(HAlign::Left, VAlign::Top),
                )
                // Baseline (horizontal red line)
                .child(
                    panel()
                        .width(length(400.0))
                        .height(length(2.0))
                        .absolute(50.0, baseline_y)
                        .background(baseline_color),
                )
                // Glyph top line (green)
                .child(
                    panel()
                        .width(length(300.0))
                        .height(length(1.0))
                        .absolute(100.0, glyph_top)
                        .background(top_color),
                )
                // Glyph bottom line (blue)
                .child(
                    panel()
                        .width(length(300.0))
                        .height(length(1.0))
                        .absolute(100.0, glyph_bottom)
                        .background(bottom_color),
                )
                // Center line (yellow)
                .child(
                    panel()
                        .width(length(250.0))
                        .height(length(1.0))
                        .absolute(125.0, (glyph_top + glyph_bottom) / 2.0)
                        .background(center_color),
                )
                // Advance marker (vertical magenta line)
                .child(
                    panel()
                        .width(length(2.0))
                        .height(length(scaled_h + 40.0))
                        .absolute(glyph_left + scaled_advance, glyph_top - 20.0)
                        .background(advance_color),
                )
                // Left edge marker (vertical white line)
                .child(
                    panel()
                        .width(length(1.0))
                        .height(length(scaled_h + 20.0))
                        .absolute(glyph_left, glyph_top - 10.0)
                        .background([0.7, 0.7, 0.7, 1.0]),
                ),
        )
        // Right side - Metrics info
        .child(
            panel()
                .flex_grow(1.0)
                .height(percent(1.0))
                .flex_direction(FlexDirection::Column)
                .gap(8.0)
                .padding_all(10.0)
                // Title
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(40.0))
                        .text(
                            format!("Glyph: '{}' (0x{:02X})", glyph_char, glyph_char as u32),
                            [1.0, 1.0, 1.0, 1.0],
                            1.2,
                        )
                        .text_align(HAlign::Left, VAlign::Center),
                )
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(25.0))
                        .text("Press SPACE for next glyph", [0.6, 0.65, 0.7, 1.0], 0.8)
                        .text_align(HAlign::Left, VAlign::Center),
                )
                // Separator
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(1.0))
                        .background([0.3, 0.35, 0.4, 1.0]),
                )
                // Size
                .child(build_metric_row(
                    "Size (w x h):",
                    format!("{:.1} x {:.1}", size_w, size_h),
                ))
                // Bearing
                .child(build_metric_row(
                    "Bearing (x, y):",
                    format!("{:.1}, {:.1}", bearing_x, bearing_y),
                ))
                // Advance
                .child(build_metric_row("Advance:", format!("{:.1}", advance)))
                // Line height
                .child(build_metric_row(
                    "Line Height:",
                    format!("{:.1}", line_height),
                ))
                // Separator
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(1.0))
                        .background([0.3, 0.35, 0.4, 1.0]),
                )
                // Color legend
                .child(build_legend_row("Baseline", baseline_color))
                .child(build_legend_row("Glyph Top", top_color))
                .child(build_legend_row("Glyph Bottom", bottom_color))
                .child(build_legend_row("Center", center_color))
                .child(build_legend_row("Advance", advance_color))
                // Separator
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(1.0))
                        .background([0.3, 0.35, 0.4, 1.0]),
                )
                // Calculated values for centering
                .child(
                    panel()
                        .width(percent(1.0))
                        .height(length(25.0))
                        .text("For VAlign::Center:", [0.8, 0.8, 0.5, 1.0], 0.9)
                        .text_align(HAlign::Left, VAlign::Center),
                )
                .child(build_metric_row(
                    "  text_height:",
                    format!("{:.1}", size_h + bearing_y),
                ))
                .child(build_metric_row(
                    "  glyph extends:",
                    format!(
                        "{:.1} above, {:.1} below baseline",
                        size_h + bearing_y,
                        -bearing_y
                    ),
                )),
        )
}

fn build_metric_row(label: &str, value: String) -> layout::PanelBuilder {
    panel()
        .width(percent(1.0))
        .height(length(22.0))
        .flex_direction(FlexDirection::Row)
        .child(
            panel()
                .width(length(140.0))
                .height(percent(1.0))
                .text(label, [0.6, 0.65, 0.7, 1.0], 0.8)
                .text_align(HAlign::Left, VAlign::Center),
        )
        .child(
            panel()
                .flex_grow(1.0)
                .height(percent(1.0))
                .text(value, [0.9, 0.95, 1.0, 1.0], 0.85)
                .text_align(HAlign::Left, VAlign::Center),
        )
}

fn build_legend_row(label: &str, color: [f32; 4]) -> layout::PanelBuilder {
    panel()
        .width(percent(1.0))
        .height(length(20.0))
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(8.0)
        .child(
            panel()
                .width(length(20.0))
                .height(length(10.0))
                .background(color),
        )
        .child(
            panel()
                .flex_grow(1.0)
                .height(percent(1.0))
                .text(label, [0.7, 0.75, 0.8, 1.0], 0.75)
                .text_align(HAlign::Left, VAlign::Center),
        )
}
