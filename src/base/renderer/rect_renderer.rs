use glow::HasContext;

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

// Pre-allocate for many rectangles
const MAX_QUADS: usize = 10000;
const MAX_VERTICES: usize = MAX_QUADS * VERTICES_PER_QUAD;
const MAX_FLOATS: usize = MAX_VERTICES * FLOATS_PER_VERTEX;

/// A simple rectangle
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Intersect this rect with another, returning the overlapping region
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right > x && bottom > y {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// Batched rectangle renderer
pub struct RectRenderer {
    program: glow::Program,
    vbo: glow::Buffer,
    projection_loc: Option<glow::UniformLocation>,
    pos_loc: u32,
    color_loc: u32,
    vertex_data: Vec<f32>,
    vertex_count: usize,
}

impl RectRenderer {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        let program = unsafe {
            let vs = gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(|e| format!("Failed to create VS: {}", e))?;
            gl.shader_source(vs, VERTEX_SHADER);
            gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) {
                return Err(format!(
                    "Vertex shader error: {}",
                    gl.get_shader_info_log(vs)
                ));
            }

            let fs = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(|e| format!("Failed to create FS: {}", e))?;
            gl.shader_source(fs, FRAGMENT_SHADER);
            gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) {
                return Err(format!(
                    "Fragment shader error: {}",
                    gl.get_shader_info_log(fs)
                ));
            }

            let program = gl
                .create_program()
                .map_err(|e| format!("Failed to create program: {}", e))?;
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(format!(
                    "Program link error: {}",
                    gl.get_program_info_log(program)
                ));
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

        Ok(RectRenderer {
            program,
            vbo,
            projection_loc,
            pos_loc,
            color_loc,
            vertex_data: Vec::with_capacity(MAX_FLOATS),
            vertex_count: 0,
        })
    }

    pub fn begin(&mut self) {
        self.vertex_data.clear();
        self.vertex_count = 0;
    }

    /// Draw a filled rectangle
    pub fn draw_rect(&mut self, rect: &Rect, color: [f32; 4]) {
        let x0 = rect.x;
        let y0 = rect.y;
        let x1 = rect.right();
        let y1 = rect.bottom();

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

    /// Draw a solid border (4 edge rectangles)
    pub fn draw_border_solid(&mut self, rect: &Rect, width: f32, color: [f32; 4]) {
        // Top edge
        self.draw_rect(&Rect::new(rect.x, rect.y, rect.width, width), color);
        // Bottom edge
        self.draw_rect(
            &Rect::new(rect.x, rect.bottom() - width, rect.width, width),
            color,
        );
        // Left edge (between top and bottom)
        self.draw_rect(
            &Rect::new(rect.x, rect.y + width, width, rect.height - 2.0 * width),
            color,
        );
        // Right edge (between top and bottom)
        self.draw_rect(
            &Rect::new(
                rect.right() - width,
                rect.y + width,
                width,
                rect.height - 2.0 * width,
            ),
            color,
        );
    }

    /// Draw a dashed border
    #[allow(dead_code)]
    pub fn draw_border_dashed(&mut self, rect: &Rect, width: f32, color: [f32; 4]) {
        let dash_len = 10.0;
        let gap_len = 5.0;

        // Top edge
        self.draw_dashed_line_h(rect.x, rect.y, rect.width, width, dash_len, gap_len, color);
        // Bottom edge
        self.draw_dashed_line_h(
            rect.x,
            rect.bottom() - width,
            rect.width,
            width,
            dash_len,
            gap_len,
            color,
        );
        // Left edge
        self.draw_dashed_line_v(rect.x, rect.y, rect.height, width, dash_len, gap_len, color);
        // Right edge
        self.draw_dashed_line_v(
            rect.right() - width,
            rect.y,
            rect.height,
            width,
            dash_len,
            gap_len,
            color,
        );
    }

    /// Draw a dotted border
    #[allow(dead_code)]
    pub fn draw_border_dotted(&mut self, rect: &Rect, width: f32, color: [f32; 4]) {
        let dot_size = width;
        let gap = width;

        // Top edge
        self.draw_dotted_line_h(rect.x, rect.y, rect.width, dot_size, gap, color);
        // Bottom edge
        self.draw_dotted_line_h(
            rect.x,
            rect.bottom() - dot_size,
            rect.width,
            dot_size,
            gap,
            color,
        );
        // Left edge
        self.draw_dotted_line_v(rect.x, rect.y, rect.height, dot_size, gap, color);
        // Right edge
        self.draw_dotted_line_v(
            rect.right() - dot_size,
            rect.y,
            rect.height,
            dot_size,
            gap,
            color,
        );
    }

    fn draw_dashed_line_h(
        &mut self,
        x: f32,
        y: f32,
        length: f32,
        width: f32,
        dash_len: f32,
        gap_len: f32,
        color: [f32; 4],
    ) {
        let mut cursor = 0.0;
        while cursor < length {
            let dash_width = (dash_len).min(length - cursor);
            self.draw_rect(&Rect::new(x + cursor, y, dash_width, width), color);
            cursor += dash_len + gap_len;
        }
    }

    fn draw_dashed_line_v(
        &mut self,
        x: f32,
        y: f32,
        length: f32,
        width: f32,
        dash_len: f32,
        gap_len: f32,
        color: [f32; 4],
    ) {
        let mut cursor = 0.0;
        while cursor < length {
            let dash_height = (dash_len).min(length - cursor);
            self.draw_rect(&Rect::new(x, y + cursor, width, dash_height), color);
            cursor += dash_len + gap_len;
        }
    }

    fn draw_dotted_line_h(
        &mut self,
        x: f32,
        y: f32,
        length: f32,
        dot_size: f32,
        gap: f32,
        color: [f32; 4],
    ) {
        let mut cursor = 0.0;
        while cursor < length {
            self.draw_rect(&Rect::new(x + cursor, y, dot_size, dot_size), color);
            cursor += dot_size + gap;
        }
    }

    fn draw_dotted_line_v(
        &mut self,
        x: f32,
        y: f32,
        length: f32,
        dot_size: f32,
        gap: f32,
        color: [f32; 4],
    ) {
        let mut cursor = 0.0;
        while cursor < length {
            self.draw_rect(&Rect::new(x, y + cursor, dot_size, dot_size), color);
            cursor += dot_size + gap;
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
        2.0 / (right - left),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (top - bottom),
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / (far - near),
        0.0,
        tx,
        ty,
        tz,
        1.0,
    ]
}
