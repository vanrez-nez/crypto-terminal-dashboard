/// Horizontal text alignment
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum HAlign {
    Left,
    #[default]
    Center,
    Right,
}

/// Vertical text alignment
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VAlign {
    Top,
    #[default]
    Center,
    Bottom,
}

/// Border style for panels
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
}

/// Border properties
#[derive(Clone, Copy, Debug, Default)]
pub struct Border {
    pub style: BorderStyle,
    pub width: f32,
    pub color: [f32; 4],
}

impl Border {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn solid(width: f32, color: [f32; 4]) -> Self {
        Self {
            style: BorderStyle::Solid,
            width,
            color,
        }
    }

    pub fn dashed(width: f32, color: [f32; 4]) -> Self {
        Self {
            style: BorderStyle::Dashed,
            width,
            color,
        }
    }

    pub fn dotted(width: f32, color: [f32; 4]) -> Self {
        Self {
            style: BorderStyle::Dotted,
            width,
            color,
        }
    }
}

/// Content that can be rendered inside a panel
#[derive(Clone, Debug, Default)]
pub enum Content {
    #[default]
    None,
    Text {
        text: String,
        color: [f32; 4],
        scale: f32,
    },
    WrappedTextBox {
        text: String,
        color: [f32; 4],
        scale: f32,
        scroll_offset: usize,
        line_gap: f32,
        indicator_color: [f32; 4],
        indicator_scale: f32,
    },
}

/// Visual style properties for a panel (not layout - Taffy handles layout)
#[derive(Clone, Debug, Default)]
pub struct PanelStyle {
    pub background_color: Option<[f32; 4]>,
    pub border: Border,
    pub content: Content,
    pub clip_overflow: bool,
    pub text_align_h: HAlign,
    pub text_align_v: VAlign,
    /// Whether this panel can receive focus
    pub focusable: bool,
    /// Panel ID for focus tracking
    pub panel_id: Option<String>,
    /// Border color when focused
    pub focus_border_color: Option<[f32; 4]>,
    /// Scroll offset for scrollable content
    pub scroll_offset: f32,
    /// Whether this panel is scrollable
    pub scrollable: bool,
}

impl PanelStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_background(mut self, color: [f32; 4]) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn with_border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>, color: [f32; 4], scale: f32) -> Self {
        self.content = Content::Text {
            text: text.into(),
            color,
            scale,
        };
        self
    }

    pub fn with_clip(mut self, clip: bool) -> Self {
        self.clip_overflow = clip;
        self
    }
}
