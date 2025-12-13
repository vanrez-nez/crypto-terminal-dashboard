/// Manages keyboard focus across focusable panels
pub struct FocusManager {
    /// List of focusable panel IDs in navigation order
    focus_order: Vec<String>,
    /// Current focus index
    current_index: usize,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focus_order: Vec::new(),
            current_index: 0,
        }
    }

    /// Register a focusable panel
    pub fn register(&mut self, id: impl Into<String>) {
        let id = id.into();
        if !self.focus_order.contains(&id) {
            self.focus_order.push(id);
        }
    }

    /// Clear all registered panels (call before rebuilding layout)
    pub fn clear(&mut self) {
        self.focus_order.clear();
        self.current_index = 0;
    }

    /// Set the list of focusable panel IDs
    pub fn set_focus_order(&mut self, order: Vec<String>) {
        self.focus_order = order;
        if self.current_index >= self.focus_order.len() {
            self.current_index = 0;
        }
    }

    /// Move focus to the next panel
    pub fn next(&mut self) {
        if !self.focus_order.is_empty() {
            self.current_index = (self.current_index + 1) % self.focus_order.len();
        }
    }

    /// Move focus to the previous panel
    pub fn previous(&mut self) {
        if !self.focus_order.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.focus_order.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }

    /// Get the currently focused panel ID
    pub fn current(&self) -> Option<&str> {
        self.focus_order.get(self.current_index).map(|s| s.as_str())
    }

    /// Check if a specific panel is focused
    pub fn is_focused(&self, id: &str) -> bool {
        self.current().map(|c| c == id).unwrap_or(false)
    }

    /// Set focus to a specific panel by ID
    pub fn set_focus(&mut self, id: &str) {
        if let Some(idx) = self.focus_order.iter().position(|s| s == id) {
            self.current_index = idx;
        }
    }

    /// Get the index of the currently focused panel
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the number of focusable panels
    pub fn count(&self) -> usize {
        self.focus_order.len()
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}
