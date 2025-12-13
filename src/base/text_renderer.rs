use crate::base::font_atlas::FontAtlas;
use glow::HasContext;

const VERTEX_SHADER: &str = r#"
    attribute vec2 a_pos;
    attribute vec2 a_uv;
    attribute vec4 a_color;
    varying vec2 v_uv;
    varying vec4 v_color;
    uniform mat4 u_projection;

    void main() {
        gl_Position = u_projection * vec4(a_pos, 0.0, 1.0);
        v_uv = a_uv;
        v_color = a_color;
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    precision mediump float;
    varying vec2 v_uv;
    varying vec4 v_color;
    uniform sampler2D u_atlas;

    void main() {
        float alpha = texture2D(u_atlas, v_uv).r;
        gl_FragColor = vec4(v_color.rgb, v_color.a * alpha);
    }
"#;

// Vertex: x, y, u, v, r, g, b, a = 8 floats = 32 bytes
const FLOATS_PER_VERTEX: usize = 8;
const VERTICES_PER_QUAD: usize = 6;

// Pre-allocate for max text (320 elements * ~50 chars * 6 vertices)
const MAX_VERTICES: usize = 320 * 50 * VERTICES_PER_QUAD;
const MAX_FLOATS: usize = MAX_VERTICES * FLOATS_PER_VERTEX;

pub struct TextRenderer {
    program: glow::Program,
    vbo: glow::Buffer,
    projection_loc: Option<glow::UniformLocation>,
    atlas_loc: Option<glow::UniformLocation>,
    pos_loc: u32,
    uv_loc: u32,
    color_loc: u32,
    vertex_data: Vec<f32>,
    vertex_count: usize,
}

impl TextRenderer {
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
            // Pre-allocate buffer
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (MAX_FLOATS * std::mem::size_of::<f32>()) as i32,
                glow::DYNAMIC_DRAW,
            );
            vbo
        };

        let projection_loc = unsafe { gl.get_uniform_location(program, "u_projection") };
        let atlas_loc = unsafe { gl.get_uniform_location(program, "u_atlas") };

        let pos_loc = unsafe { gl.get_attrib_location(program, "a_pos").unwrap_or(0) };
        let uv_loc = unsafe { gl.get_attrib_location(program, "a_uv").unwrap_or(1) };
        let color_loc = unsafe { gl.get_attrib_location(program, "a_color").unwrap_or(2) };

        Ok(TextRenderer {
            program,
            vbo,
            projection_loc,
            atlas_loc,
            pos_loc,
            uv_loc,
            color_loc,
            vertex_data: Vec::with_capacity(MAX_FLOATS),
            vertex_count: 0,
        })
    }

    pub fn begin(&mut self) {
        self.vertex_data.clear();
        self.vertex_count = 0;
    }

    pub fn draw_text(
        &mut self,
        atlas: &FontAtlas,
        text: &str,
        mut x: f32,
        y: f32,
        scale: f32,
        color: [f32; 4],
    ) {
        for c in text.chars() {
            if let Some(glyph) = atlas.get_glyph(c) {
                let x0 = x + glyph.bearing.0 * scale;
                let y0 = y - (glyph.size.1 + glyph.bearing.1) * scale;
                let x1 = x0 + glyph.size.0 * scale;
                let y1 = y0 + glyph.size.1 * scale;

                let (u0, v0) = glyph.uv_min;
                let (u1, v1) = glyph.uv_max;

                // Triangle 1 - UV coords flipped vertically for OpenGL texture orientation
                self.push_vertex(x0, y1, u0, v1, &color);
                self.push_vertex(x1, y1, u1, v1, &color);
                self.push_vertex(x0, y0, u0, v0, &color);

                // Triangle 2
                self.push_vertex(x1, y1, u1, v1, &color);
                self.push_vertex(x1, y0, u1, v0, &color);
                self.push_vertex(x0, y0, u0, v0, &color);

                self.vertex_count += 6;
                x += glyph.advance * scale;
            }
        }
    }

    /// Draw text with vertical offset for scrolling
    pub fn draw_text_offset(
        &mut self,
        atlas: &FontAtlas,
        text: &str,
        x: f32,
        y: f32,
        y_offset: f32,
        scale: f32,
        color: [f32; 4],
    ) {
        self.draw_text(atlas, text, x, y + y_offset, scale, color);
    }

    fn push_vertex(&mut self, x: f32, y: f32, u: f32, v: f32, color: &[f32; 4]) {
        self.vertex_data.push(x);
        self.vertex_data.push(y);
        self.vertex_data.push(u);
        self.vertex_data.push(v);
        self.vertex_data.push(color[0]);
        self.vertex_data.push(color[1]);
        self.vertex_data.push(color[2]);
        self.vertex_data.push(color[3]);
    }

    pub fn end(
        &mut self,
        gl: &glow::Context,
        atlas: &FontAtlas,
        screen_width: u32,
        screen_height: u32,
    ) {
        if self.vertex_count == 0 {
            return;
        }

        unsafe {
            gl.use_program(Some(self.program));

            // Orthographic projection - standard top-left origin
            let projection = ortho_projection(0.0, screen_width as f32, screen_height as f32, 0.0);
            gl.uniform_matrix_4_f32_slice(self.projection_loc.as_ref(), false, &projection);

            // Bind atlas texture
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(atlas.texture));
            gl.uniform_1_i32(self.atlas_loc.as_ref(), 0);

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

            // UV: offset 8 (2 floats)
            gl.enable_vertex_attrib_array(self.uv_loc);
            gl.vertex_attrib_pointer_f32(self.uv_loc, 2, glow::FLOAT, false, stride, 8);

            // Color: offset 16 (4 floats)
            gl.enable_vertex_attrib_array(self.color_loc);
            gl.vertex_attrib_pointer_f32(self.color_loc, 4, glow::FLOAT, false, stride, 16);

            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count as i32);

            gl.disable_vertex_attrib_array(self.pos_loc);
            gl.disable_vertex_attrib_array(self.uv_loc);
            gl.disable_vertex_attrib_array(self.color_loc);
        }
    }

    pub fn measure_text(&self, atlas: &FontAtlas, text: &str, scale: f32) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for c in text.chars() {
            if let Some(glyph) = atlas.get_glyph(c) {
                width += glyph.advance * scale;
                // Include bearing to match draw_text positioning
                let glyph_height = (glyph.size.1 + glyph.bearing.1) * scale;
                height = height.max(glyph_height);
            }
        }

        // Return actual text bounds, not clamped to line_height
        // (line_height clamping was causing VAlign::Center to miscalculate)
        (width, height)
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
