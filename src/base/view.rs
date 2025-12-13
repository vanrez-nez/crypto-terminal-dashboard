/// Represents a distinct screen/tab in the application
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Overview,   // Combined stats + processes + logs
    Stats,      // Full-screen stats panel
    Processes,  // Full-screen process list
    Logs,       // Full-screen log viewer
    GlyphDebug, // Glyph metrics visualization
}

impl View {
    pub fn all() -> &'static [View] {
        &[
            View::Overview,
            View::Stats,
            View::Processes,
            View::Logs,
            View::GlyphDebug,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            View::Overview => "Overview",
            View::Stats => "Stats",
            View::Processes => "Processes",
            View::Logs => "Logs",
            View::GlyphDebug => "Glyphs",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            View::Overview => 0,
            View::Stats => 1,
            View::Processes => 2,
            View::Logs => 3,
            View::GlyphDebug => 4,
        }
    }

    pub fn from_index(i: usize) -> Option<View> {
        match i {
            0 => Some(View::Overview),
            1 => Some(View::Stats),
            2 => Some(View::Processes),
            3 => Some(View::Logs),
            4 => Some(View::GlyphDebug),
            _ => None,
        }
    }
}

/// Manages view switching
pub struct ViewManager {
    current: View,
    previous: View,
}

impl ViewManager {
    pub fn new() -> Self {
        Self {
            current: View::Overview,
            previous: View::Overview,
        }
    }

    pub fn current(&self) -> View {
        self.current
    }

    /// Returns true if the view changed since last check
    pub fn changed(&self) -> bool {
        self.current != self.previous
    }

    /// Call after handling view change
    pub fn acknowledge_change(&mut self) {
        self.previous = self.current;
    }

    pub fn next(&mut self) {
        let idx = (self.current.index() + 1) % View::all().len();
        self.current = View::from_index(idx).unwrap();
    }

    pub fn previous(&mut self) {
        let len = View::all().len();
        let idx = (self.current.index() + len - 1) % len;
        self.current = View::from_index(idx).unwrap();
    }

    pub fn set(&mut self, view: View) {
        self.current = view;
    }

    /// Returns the focus order for the current view
    pub fn focus_order(&self) -> Vec<String> {
        match self.current {
            View::Overview => vec!["stats", "processes", "logs"],
            View::Stats => vec!["cpu_detail", "ram_detail", "disk_detail", "network_detail"],
            View::Processes => vec!["process_list"],
            View::Logs => vec!["log_viewer"],
            View::GlyphDebug => vec!["glyph_display"],
        }
        .into_iter()
        .map(String::from)
        .collect()
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}
