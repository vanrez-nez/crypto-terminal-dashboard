use crate::base::renderer::rect_renderer::Rect;
use glow::HasContext;

/// Manages a stack of scissor rectangles for nested clipping
pub struct ScissorStack {
    stack: Vec<Rect>,
    screen_height: u32,
}

impl ScissorStack {
    pub fn new(screen_height: u32) -> Self {
        Self {
            stack: Vec::new(),
            screen_height,
        }
    }

    /// Push a new scissor rect, intersecting with current if any
    pub fn push(&mut self, gl: &glow::Context, rect: Rect) {
        let new_rect = if let Some(current) = self.stack.last() {
            // Intersect with current scissor region
            current
                .intersect(&rect)
                .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
        } else {
            rect
        };

        self.stack.push(new_rect);
        self.apply_scissor(gl, &new_rect);
    }

    /// Pop the current scissor rect
    pub fn pop(&mut self, gl: &glow::Context) {
        self.stack.pop();

        if let Some(rect) = self.stack.last() {
            self.apply_scissor(gl, rect);
        } else {
            unsafe {
                gl.disable(glow::SCISSOR_TEST);
            }
        }
    }

    /// Apply scissor rect to OpenGL
    fn apply_scissor(&self, gl: &glow::Context, rect: &Rect) {
        unsafe {
            gl.enable(glow::SCISSOR_TEST);
            // OpenGL scissor uses bottom-left origin, convert from top-left
            let gl_y = self.screen_height as i32 - rect.y as i32 - rect.height as i32;
            gl.scissor(
                rect.x as i32,
                gl_y.max(0),
                rect.width as i32,
                rect.height as i32,
            );
        }
    }

    /// Clear the scissor stack and disable scissor test
    pub fn clear(&mut self, gl: &glow::Context) {
        self.stack.clear();
        unsafe {
            gl.disable(glow::SCISSOR_TEST);
        }
    }

    /// Check if scissor is currently active
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        !self.stack.is_empty()
    }
}
