/// Scroll state for a scrollable panel
#[derive(Clone, Debug)]
pub struct ScrollState {
    /// Current scroll offset (pixels from top)
    pub offset: f32,
    /// Total height of content
    pub content_height: f32,
    /// Height of the visible viewport
    pub viewport_height: f32,
    /// Line height for line-based scrolling
    pub line_height: f32,
}

impl ScrollState {
    pub fn new(viewport_height: f32) -> Self {
        Self {
            offset: 0.0,
            content_height: 0.0,
            viewport_height,
            line_height: 30.0, // Default line height
        }
    }

    /// Set the content height
    pub fn set_content_height(&mut self, height: f32) {
        self.content_height = height;
        // Clamp offset if content shrunk
        self.clamp_offset();
    }

    /// Set the viewport height
    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_height = height;
        self.clamp_offset();
    }

    /// Set line height for line-based scrolling
    pub fn set_line_height(&mut self, height: f32) {
        self.line_height = height;
    }

    /// Scroll up by a number of lines
    pub fn scroll_up(&mut self, lines: f32) {
        self.offset -= lines * self.line_height;
        self.clamp_offset();
    }

    /// Scroll down by a number of lines
    pub fn scroll_down(&mut self, lines: f32) {
        self.offset += lines * self.line_height;
        self.clamp_offset();
    }

    /// Scroll up by one page
    pub fn page_up(&mut self) {
        self.offset -= self.viewport_height * 0.9;
        self.clamp_offset();
    }

    /// Scroll down by one page
    pub fn page_down(&mut self) {
        self.offset += self.viewport_height * 0.9;
        self.clamp_offset();
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.offset = 0.0;
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        self.offset = self.max_offset();
    }

    /// Get the maximum scroll offset
    pub fn max_offset(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    /// Clamp offset to valid range
    fn clamp_offset(&mut self) {
        self.offset = self.offset.clamp(0.0, self.max_offset());
    }

    /// Get the visible range (start, end) in content coordinates
    pub fn visible_range(&self) -> (f32, f32) {
        (self.offset, self.offset + self.viewport_height)
    }

    /// Check if scrolling is needed (content exceeds viewport)
    pub fn can_scroll(&self) -> bool {
        self.content_height > self.viewport_height
    }

    /// Get scroll progress as 0.0-1.0
    pub fn scroll_progress(&self) -> f32 {
        if self.max_offset() <= 0.0 {
            0.0
        } else {
            self.offset / self.max_offset()
        }
    }

    /// Get the scrollbar thumb position and size (relative 0.0-1.0)
    pub fn scrollbar_thumb(&self) -> (f32, f32) {
        if self.content_height <= 0.0 {
            return (0.0, 1.0);
        }

        let thumb_size = (self.viewport_height / self.content_height).min(1.0);
        let thumb_pos = self.scroll_progress() * (1.0 - thumb_size);

        (thumb_pos, thumb_size)
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// Collection of scroll states indexed by panel ID
pub struct ScrollStates {
    states: std::collections::HashMap<String, ScrollState>,
}

impl ScrollStates {
    pub fn new() -> Self {
        Self {
            states: std::collections::HashMap::new(),
        }
    }

    /// Get or create a scroll state for a panel
    pub fn get_or_create(&mut self, id: &str, viewport_height: f32) -> &mut ScrollState {
        self.states
            .entry(id.to_string())
            .or_insert_with(|| ScrollState::new(viewport_height))
    }

    /// Get a scroll state by panel ID
    pub fn get(&self, id: &str) -> Option<&ScrollState> {
        self.states.get(id)
    }

    /// Get a mutable scroll state by panel ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ScrollState> {
        self.states.get_mut(id)
    }

    /// Get the scroll offset for a panel (0.0 if not scrollable)
    pub fn offset(&self, id: &str) -> f32 {
        self.states.get(id).map(|s| s.offset).unwrap_or(0.0)
    }
}

impl Default for ScrollStates {
    fn default() -> Self {
        Self::new()
    }
}
