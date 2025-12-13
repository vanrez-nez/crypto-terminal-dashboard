//! OpenGL renderer for chart primitives (lines, candles, bars)
//!
//! This renderer is optimized for drawing financial charts with batched rendering.

use dashboard_system::glow::{self, HasContext};

const VERTEX_SHADER: &str = r#"
    attribute vec2 a_pos;
    attribute vec4 a_color;
    varying vec4 v_color;
    uniform mat4 u_projection;

    void main() {
        gl_Position = u_projection * vec4(a_pos, 0.0, 1.0);
        v_color = a_color;
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    precision mediump float;
    varying vec4 v_color;

    void main() {
        gl_FragColor = v_color;
    }
"#;

// Vertex: x, y, r, g, b, a = 6 floats = 24 bytes
const FLOATS_PER_VERTEX: usize = 6;
const VERTICES_PER_QUAD: usize = 6;

// Pre-allocate for chart rendering (lines, candles, bars)
const MAX_QUADS: usize = 20000;
const MAX_VERTICES: usize = MAX_QUADS * VERTICES_PER_QUAD;
const MAX_FLOATS: usize = MAX_VERTICES * FLOATS_PER_VERTEX;

/// Batched chart renderer for OpenGL
pub struct ChartRenderer {
    program: glow::Program,
    vbo: glow::Buffer,
    projection_loc: Option<glow::UniformLocation>,
    pos_loc: u32,
    color_loc: u32,
    vertex_data: Vec<f32>,
    vertex_count: usize,
}

impl ChartRenderer {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        let program = unsafe {
            let vs = gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(|e| format!("Failed to create VS: {}", e))?;
            gl.shader_source(vs, VERTEX_SHADER);
            gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) {
                return Err(format!("Vertex shader error: {}", gl.get_shader_info_log(vs)));
            }

            let fs = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(|e| format!("Failed to create FS: {}", e))?;
            gl.shader_source(fs, FRAGMENT_SHADER);
            gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) {
                return Err(format!("Fragment shader error: {}", gl.get_shader_info_log(fs)));
            }

            let program = gl
                .create_program()
                .map_err(|e| format!("Failed to create program: {}", e))?;
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(format!("Program link error: {}", gl.get_program_info_log(program)));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);
            program
        };

        let vbo = unsafe {
            let vbo = gl
                .create_buffer()
                .map_err(|e| format!("Failed to create VBO: {}", e))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (MAX_FLOATS * std::mem::size_of::<f32>()) as i32,
                glow::DYNAMIC_DRAW,
            );
            vbo
        };

        let projection_loc = unsafe { gl.get_uniform_location(program, "u_projection") };
        let pos_loc = unsafe { gl.get_attrib_location(program, "a_pos").unwrap_or(0) };
        let color_loc = unsafe { gl.get_attrib_location(program, "a_color").unwrap_or(1) };

        Ok(ChartRenderer {
            program,
            vbo,
            projection_loc,
            pos_loc,
            color_loc,
            vertex_data: Vec::with_capacity(MAX_FLOATS),
            vertex_count: 0,
        })
    }

    /// Begin a new batch
    pub fn begin(&mut self) {
        self.vertex_data.clear();
        self.vertex_count = 0;
    }

    /// Draw a filled rectangle
    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        let x0 = x;
        let y0 = y;
        let x1 = x + width;
        let y1 = y + height;

        // Triangle 1
        self.push_vertex(x0, y0, &color);
        self.push_vertex(x1, y0, &color);
        self.push_vertex(x0, y1, &color);

        // Triangle 2
        self.push_vertex(x1, y0, &color);
        self.push_vertex(x1, y1, &color);
        self.push_vertex(x0, y1, &color);

        self.vertex_count += 6;
    }

    /// Draw a horizontal line
    pub fn draw_line_h(&mut self, x: f32, y: f32, length: f32, thickness: f32, color: [f32; 4]) {
        self.draw_rect(x, y - thickness * 0.5, length, thickness, color);
    }

    /// Draw a vertical line
    pub fn draw_line_v(&mut self, x: f32, y: f32, length: f32, thickness: f32, color: [f32; 4]) {
        self.draw_rect(x - thickness * 0.5, y, thickness, length, color);
    }

    /// Draw a line between two points
    pub fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        // Perpendicular normalized vector scaled by half thickness
        let px = -dy / len * thickness * 0.5;
        let py = dx / len * thickness * 0.5;

        // Quad corners
        let ax = x1 + px;
        let ay = y1 + py;
        let bx = x1 - px;
        let by = y1 - py;
        let cx = x2 - px;
        let cy = y2 - py;
        let dx = x2 + px;
        let dy = y2 + py;

        // Triangle 1
        self.push_vertex(ax, ay, &color);
        self.push_vertex(bx, by, &color);
        self.push_vertex(cx, cy, &color);

        // Triangle 2
        self.push_vertex(ax, ay, &color);
        self.push_vertex(cx, cy, &color);
        self.push_vertex(dx, dy, &color);

        self.vertex_count += 6;
    }

    /// Draw a candlestick (wick + body)
    ///
    /// * `x` - center x position
    /// * `open`, `high`, `low`, `close` - price values in pixel coordinates (y)
    /// * `body_width` - width of the candle body
    /// * `wick_width` - width of the wick line
    pub fn draw_candle(
        &mut self,
        x: f32,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        body_width: f32,
        wick_width: f32,
        color: [f32; 4],
    ) {
        // Draw wick (vertical line from low to high)
        self.draw_rect(
            x - wick_width * 0.5,
            high.min(low),
            wick_width,
            (high - low).abs(),
            color,
        );

        // Draw body (rectangle from open to close)
        let body_top = open.min(close);
        let body_height = (open - close).abs().max(1.0); // Ensure at least 1px height
        self.draw_rect(
            x - body_width * 0.5,
            body_top,
            body_width,
            body_height,
            color,
        );
    }

    /// Draw a volume bar
    pub fn draw_volume_bar(
        &mut self,
        x: f32,
        bottom: f32,
        height: f32,
        width: f32,
        color: [f32; 4],
    ) {
        self.draw_rect(x - width * 0.5, bottom - height, width, height, color);
    }

    /// Draw a dashed horizontal line
    pub fn draw_dashed_line_h(
        &mut self,
        x: f32,
        y: f32,
        length: f32,
        thickness: f32,
        dash_len: f32,
        gap_len: f32,
        color: [f32; 4],
    ) {
        let mut cursor = 0.0;
        while cursor < length {
            let dash = dash_len.min(length - cursor);
            self.draw_line_h(x + cursor, y, dash, thickness, color);
            cursor += dash_len + gap_len;
        }
    }

    /// Draw a dashed vertical line
    pub fn draw_dashed_line_v(
        &mut self,
        x: f32,
        y: f32,
        length: f32,
        thickness: f32,
        dash_len: f32,
        gap_len: f32,
        color: [f32; 4],
    ) {
        let mut cursor = 0.0;
        while cursor < length {
            let dash = dash_len.min(length - cursor);
            self.draw_line_v(x, y + cursor, dash, thickness, color);
            cursor += dash_len + gap_len;
        }
    }

    /// Draw grid lines for a chart
    pub fn draw_grid(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        h_lines: usize,
        v_lines: usize,
        thickness: f32,
        color: [f32; 4],
    ) {
        // Horizontal lines
        if h_lines > 0 {
            let step = height / (h_lines + 1) as f32;
            for i in 1..=h_lines {
                let ly = y + step * i as f32;
                self.draw_line_h(x, ly, width, thickness, color);
            }
        }

        // Vertical lines
        if v_lines > 0 {
            let step = width / (v_lines + 1) as f32;
            for i in 1..=v_lines {
                let lx = x + step * i as f32;
                self.draw_line_v(lx, y, height, thickness, color);
            }
        }
    }

    /// Draw a polyline (connected line segments)
    pub fn draw_polyline(&mut self, points: &[(f32, f32)], thickness: f32, color: [f32; 4]) {
        if points.len() < 2 {
            return;
        }

        for window in points.windows(2) {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            self.draw_line(x1, y1, x2, y2, thickness, color);
        }
    }

    /// Draw a filled area under a polyline (for area charts)
    pub fn draw_filled_area(
        &mut self,
        points: &[(f32, f32)],
        baseline_y: f32,
        color: [f32; 4],
    ) {
        if points.len() < 2 {
            return;
        }

        for window in points.windows(2) {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];

            // Quad from (x1, y1) to (x2, y2) to (x2, baseline) to (x1, baseline)
            // Triangle 1
            self.push_vertex(x1, y1, &color);
            self.push_vertex(x2, y2, &color);
            self.push_vertex(x1, baseline_y, &color);

            // Triangle 2
            self.push_vertex(x2, y2, &color);
            self.push_vertex(x2, baseline_y, &color);
            self.push_vertex(x1, baseline_y, &color);

            self.vertex_count += 6;
        }
    }

    /// Draw a marker (small filled circle approximated as octagon)
    pub fn draw_marker(&mut self, x: f32, y: f32, size: f32, color: [f32; 4]) {
        // Draw as a filled circle using triangles (8-sided polygon)
        let r = size * 0.5;
        let segments = 8;

        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let x1 = x + angle1.cos() * r;
            let y1 = y + angle1.sin() * r;
            let x2 = x + angle2.cos() * r;
            let y2 = y + angle2.sin() * r;

            self.push_vertex(x, y, &color);
            self.push_vertex(x1, y1, &color);
            self.push_vertex(x2, y2, &color);

            self.vertex_count += 3;
        }
    }

    fn push_vertex(&mut self, x: f32, y: f32, color: &[f32; 4]) {
        self.vertex_data.push(x);
        self.vertex_data.push(y);
        self.vertex_data.push(color[0]);
        self.vertex_data.push(color[1]);
        self.vertex_data.push(color[2]);
        self.vertex_data.push(color[3]);
    }

    /// Flush the batch to the GPU
    pub fn end(&mut self, gl: &glow::Context, screen_width: u32, screen_height: u32) {
        if self.vertex_count == 0 {
            return;
        }

        unsafe {
            gl.use_program(Some(self.program));

            // Orthographic projection - top-left origin
            let projection = ortho_projection(0.0, screen_width as f32, screen_height as f32, 0.0);
            gl.uniform_matrix_4_f32_slice(self.projection_loc.as_ref(), false, &projection);

            // Upload vertex data
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(&self.vertex_data),
            );

            let stride = (FLOATS_PER_VERTEX * std::mem::size_of::<f32>()) as i32;

            // Position: offset 0
            gl.enable_vertex_attrib_array(self.pos_loc);
            gl.vertex_attrib_pointer_f32(self.pos_loc, 2, glow::FLOAT, false, stride, 0);

            // Color: offset 8 (2 floats)
            gl.enable_vertex_attrib_array(self.color_loc);
            gl.vertex_attrib_pointer_f32(self.color_loc, 4, glow::FLOAT, false, stride, 8);

            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count as i32);

            gl.disable_vertex_attrib_array(self.pos_loc);
            gl.disable_vertex_attrib_array(self.color_loc);
        }
    }
}

fn ortho_projection(left: f32, right: f32, bottom: f32, top: f32) -> [f32; 16] {
    let near = -1.0f32;
    let far = 1.0f32;

    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);

    [
        2.0 / (right - left), 0.0, 0.0, 0.0,
        0.0, 2.0 / (top - bottom), 0.0, 0.0,
        0.0, 0.0, -2.0 / (far - near), 0.0,
        tx, ty, tz, 1.0,
    ]
}

/// Chart coordinate system helper
pub struct ChartBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl ChartBounds {
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self { x_min, x_max, y_min, y_max }
    }

    /// Create bounds from a slice of (x, y) points
    pub fn from_points(points: &[(f64, f64)]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        for &(x, y) in points {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }

        Some(Self { x_min, x_max, y_min, y_max })
    }

    /// Add padding to bounds (as a fraction, e.g., 0.05 for 5%)
    pub fn with_padding(mut self, padding: f64) -> Self {
        let x_range = self.x_max - self.x_min;
        let y_range = self.y_max - self.y_min;
        self.x_min -= x_range * padding;
        self.x_max += x_range * padding;
        self.y_min -= y_range * padding;
        self.y_max += y_range * padding;
        self
    }

    /// Map a data point to pixel coordinates
    pub fn to_pixel(&self, data_x: f64, data_y: f64, pixel_rect: &PixelRect) -> (f32, f32) {
        let x_range = self.x_max - self.x_min;
        let y_range = self.y_max - self.y_min;

        let x_norm = if x_range > 0.0 {
            (data_x - self.x_min) / x_range
        } else {
            0.5
        };

        let y_norm = if y_range > 0.0 {
            (data_y - self.y_min) / y_range
        } else {
            0.5
        };

        let px = pixel_rect.x + (x_norm as f32) * pixel_rect.width;
        // Invert Y because pixel coordinates are top-down
        let py = pixel_rect.y + pixel_rect.height - (y_norm as f32) * pixel_rect.height;

        (px, py)
    }

    /// Map pixel coordinates back to data coordinates
    pub fn from_pixel(&self, px: f32, py: f32, pixel_rect: &PixelRect) -> (f64, f64) {
        let x_norm = (px - pixel_rect.x) / pixel_rect.width;
        let y_norm = 1.0 - (py - pixel_rect.y) / pixel_rect.height;

        let data_x = self.x_min + (x_norm as f64) * (self.x_max - self.x_min);
        let data_y = self.y_min + (y_norm as f64) * (self.y_max - self.y_min);

        (data_x, data_y)
    }
}

/// Pixel rectangle for chart area
#[derive(Clone, Copy, Debug)]
pub struct PixelRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PixelRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// Calculate visible range for scrollable charts
pub struct VisibleRange {
    pub start_idx: usize,
    pub end_idx: usize,
    pub empty_right_slots: usize,
    pub clamped_offset: isize,
}

/// Calculate the visible range of candles based on scroll offset
pub fn calculate_visible_range(
    total_candles: usize,
    visible_slots: usize,
    scroll_offset: isize,
) -> VisibleRange {
    if total_candles == 0 {
        return VisibleRange {
            start_idx: 0,
            end_idx: 0,
            empty_right_slots: visible_slots,
            clamped_offset: 0,
        };
    }

    // Clamp scroll offset: negative = showing future (empty), positive = showing history
    // offset 0 = latest candle at right edge
    let max_offset = (total_candles as isize - 1).max(0);
    let min_offset = -(visible_slots as isize - 1);
    let clamped_offset = scroll_offset.clamp(min_offset, max_offset);

    // Calculate which candles are visible
    // With offset=0, the rightmost slot shows candle[total-1]
    // With positive offset, we scroll into history
    let right_idx = (total_candles as isize - 1 - clamped_offset).max(0) as usize;
    let left_idx = right_idx.saturating_sub(visible_slots - 1);

    let empty_right_slots = if clamped_offset < 0 {
        (-clamped_offset) as usize
    } else {
        0
    };

    VisibleRange {
        start_idx: left_idx,
        end_idx: right_idx + 1, // exclusive
        empty_right_slots,
        clamped_offset,
    }
}
