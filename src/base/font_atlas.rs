use fontdue::{Font, FontSettings};
use glow::HasContext;
use std::collections::HashMap;

const EXTRA_CHARS: &[char] = &[
    '●', // black circle (Live status)
    '◐', // quarter circle (Connecting)
    '○', // white circle (Disconnected)
    '◆', // diamond (Mock/flat change)
    '▲', // price up
    '▼', // price down
    '←', // left arrow (scroll hint)
    '→', // right arrow (scroll hint)
];

#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub uv_min: (f32, f32),
    pub uv_max: (f32, f32),
    pub size: (f32, f32),
    pub bearing: (f32, f32),
    pub advance: f32,
}

pub struct FontAtlas {
    pub texture: glow::Texture,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub atlas_size: u32,
    pub line_height: f32,
}

impl FontAtlas {
    pub fn new(gl: &glow::Context, font_data: &[u8], font_size: f32) -> Result<Self, String> {
        let font = Font::from_bytes(font_data, FontSettings::default())
            .map_err(|e| format!("Failed to load font: {}", e))?;

        // ASCII printable characters (32-126) plus explicit extras
        let mut chars: Vec<char> = (32u8..=126u8).map(|c| c as char).collect();
        chars.extend(EXTRA_CHARS.iter().copied());
        chars.sort_unstable();
        chars.dedup();

        // First pass: rasterize all glyphs to determine atlas size
        let mut rasterized: Vec<(char, fontdue::Metrics, Vec<u8>)> = Vec::new();
        let mut max_height: u32 = 0;
        let mut total_width: u32 = 0;

        for &c in &chars {
            let (metrics, bitmap) = font.rasterize(c, font_size);
            max_height = max_height.max(metrics.height as u32);
            total_width += metrics.width as u32 + 2; // padding
            rasterized.push((c, metrics, bitmap));
        }

        // Calculate atlas size (power of 2, try to make it square-ish)
        let row_height = max_height + 2;
        let chars_per_row = ((512.0 / (total_width as f32 / chars.len() as f32)).ceil() as usize)
            .max(1)
            .min(chars.len());
        let num_rows = (chars.len() + chars_per_row - 1) / chars_per_row;

        let atlas_width = 512u32;
        let atlas_height = ((num_rows as u32 * row_height) as u32)
            .next_power_of_two()
            .max(256);
        let atlas_size = atlas_width.max(atlas_height);

        // Create atlas pixel data
        let mut atlas_data = vec![0u8; (atlas_size * atlas_size) as usize];
        let mut glyphs = HashMap::new();

        let mut cursor_x: u32 = 1;
        let mut cursor_y: u32 = 1;
        let mut line_height: f32 = 0.0;

        for (c, metrics, bitmap) in rasterized {
            let glyph_width = metrics.width as u32;
            let glyph_height = metrics.height as u32;

            // Move to next row if needed
            if cursor_x + glyph_width + 1 > atlas_size {
                cursor_x = 1;
                cursor_y += row_height;
            }

            // Copy glyph bitmap to atlas
            for y in 0..glyph_height {
                for x in 0..glyph_width {
                    let src_idx = (y * glyph_width + x) as usize;
                    let dst_x = cursor_x + x;
                    let dst_y = cursor_y + y;
                    let dst_idx = (dst_y * atlas_size + dst_x) as usize;
                    if src_idx < bitmap.len() && dst_idx < atlas_data.len() {
                        atlas_data[dst_idx] = bitmap[src_idx];
                    }
                }
            }

            // Store glyph info with UV coordinates
            let uv_min = (
                cursor_x as f32 / atlas_size as f32,
                cursor_y as f32 / atlas_size as f32,
            );
            let uv_max = (
                (cursor_x + glyph_width) as f32 / atlas_size as f32,
                (cursor_y + glyph_height) as f32 / atlas_size as f32,
            );

            glyphs.insert(
                c,
                GlyphInfo {
                    uv_min,
                    uv_max,
                    size: (glyph_width as f32, glyph_height as f32),
                    bearing: (metrics.xmin as f32, metrics.ymin as f32),
                    advance: metrics.advance_width,
                },
            );

            line_height = line_height.max(metrics.height as f32);
            cursor_x += glyph_width + 2;
        }

        // Create OpenGL texture
        let texture = unsafe {
            let tex = gl
                .create_texture()
                .map_err(|e| format!("Failed to create texture: {}", e))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::LUMINANCE as i32,
                atlas_size as i32,
                atlas_size as i32,
                0,
                glow::LUMINANCE,
                glow::UNSIGNED_BYTE,
                Some(&atlas_data),
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
            tex
        };

        println!(
            "Font atlas created: {}x{} ({} glyphs)",
            atlas_size,
            atlas_size,
            glyphs.len()
        );

        Ok(FontAtlas {
            texture,
            glyphs,
            atlas_size,
            line_height,
        })
    }

    pub fn get_glyph(&self, c: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&c).or_else(|| self.glyphs.get(&'?'))
    }

    /// Measure text dimensions without needing a TextRenderer
    /// Returns (width, height) in pixels at the given scale
    pub fn measure_text(&self, text: &str, scale: f32) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for c in text.chars() {
            if let Some(glyph) = self.get_glyph(c) {
                width += glyph.advance * scale;
                // Include bearing to match draw_text positioning
                let glyph_height = (glyph.size.1 + glyph.bearing.1) * scale;
                height = height.max(glyph_height);
            }
        }

        // Ensure minimum height based on line_height for empty or small text
        height = height.max(self.line_height * scale);

        (width, height)
    }
}
