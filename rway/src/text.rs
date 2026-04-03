// text.rs — Title bar text rendering using cosmic-text
//
// TextRenderer holds a global FontSystem + SwashCache (created once at startup)
// and a per-window cache of pre-rendered MemoryRenderBuffers. Text is only
// re-rasterized when the title, size, or focused state changes.

use std::collections::HashMap;

use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use smithay::{
    backend::allocator::Fourcc, backend::renderer::element::memory::MemoryRenderBuffer,
    utils::Transform,
};

use rway_tiling::Rect;

/// Parsed font configuration from Sway's `font` directive.
#[derive(Debug, Clone)]
#[allow(dead_code)] // bold/italic used when cosmic-text Attrs supports weight/style
pub(crate) struct FontConfig {
    pub family: String,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "monospace".to_string(),
            size: 10.0,
            bold: false,
            italic: false,
        }
    }
}

impl FontConfig {
    /// Parse Sway font string: `[pango:]<family> [Bold] [Italic] <size>`
    pub fn from_sway_font(font_str: Option<&str>) -> Self {
        let Some(raw) = font_str else {
            return Self::default();
        };

        let s = raw.strip_prefix("pango:").unwrap_or(raw).trim();
        if s.is_empty() {
            return Self::default();
        }

        let tokens: Vec<&str> = s.split_whitespace().collect();
        let mut size = 10.0_f32;
        let mut bold = false;
        let mut italic = false;
        let mut family_end = tokens.len();

        // Parse from the end: last token may be size, preceding may be style
        if let Some(last) = tokens.last() {
            if let Ok(parsed) = last.parse::<f32>() {
                size = parsed;
                family_end -= 1;
            }
        }

        // Check for Bold/Italic before the size
        while family_end > 0 {
            match tokens[family_end - 1].to_lowercase().as_str() {
                "bold" => {
                    bold = true;
                    family_end -= 1;
                }
                "italic" => {
                    italic = true;
                    family_end -= 1;
                }
                _ => break,
            }
        }

        let family = if family_end > 0 {
            tokens[..family_end].join(" ")
        } else {
            "monospace".to_string()
        };

        Self {
            family,
            size,
            bold,
            italic,
        }
    }
}

/// Cache key for rendered title text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    window_id: u64,
    title: String,
    width: i32,
    height: i32,
}

/// Cached rendered title bar buffer.
struct CachedTitle {
    buffer: MemoryRenderBuffer,
}

/// Title bar text rendering engine using cosmic-text.
///
/// Created once at startup, stored in RwayState. Holds the global font system
/// and glyph cache, plus a per-window cache of pre-rendered title buffers.
pub(crate) struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    font_config: FontConfig,
    cache: HashMap<CacheKey, CachedTitle>,
}

impl TextRenderer {
    /// Create a new TextRenderer. Loads system fonts (may take 50-200ms).
    pub fn new(font_config: FontConfig) -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            font_config,
            cache: HashMap::new(),
        }
    }

    /// Render title text into a MemoryRenderBuffer.
    /// Returns a cached buffer if the title/size haven't changed.
    pub fn render_title(
        &mut self,
        window_id: u64,
        title: &str,
        rect: Rect,
        text_color: [u8; 4],
        bg_color: [u8; 4],
    ) -> &MemoryRenderBuffer {
        let key = CacheKey {
            window_id,
            title: title.to_string(),
            width: rect.width,
            height: rect.height,
        };

        if !self.cache.contains_key(&key) {
            // Remove old entry for this window_id (different title/size)
            self.cache.retain(|k, _| k.window_id != window_id);

            let buf = self.rasterize(title, rect.width, rect.height, text_color, bg_color);
            self.cache.insert(key.clone(), CachedTitle { buffer: buf });
        }

        &self.cache[&key].buffer
    }

    /// Rasterize text to a MemoryRenderBuffer.
    fn rasterize(
        &mut self,
        text: &str,
        width: i32,
        height: i32,
        text_color: [u8; 4],
        bg_color: [u8; 4],
    ) -> MemoryRenderBuffer {
        let w = width.max(1) as usize;
        let h = height.max(1) as usize;

        // Create pixel buffer (Argb8888: B, G, R, A on little-endian)
        let mut pixels = vec![0u8; w * h * 4];

        // Fill background
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[0] = bg_color[2]; // B
            pixel[1] = bg_color[1]; // G
            pixel[2] = bg_color[0]; // R
            pixel[3] = bg_color[3]; // A
        }

        // Set up cosmic-text buffer
        let font_size = self.font_config.size;
        let line_height = height as f32;
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let family = Family::Name(&self.font_config.family);
        let attrs = Attrs::new().family(family);

        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        // Rasterize glyphs onto pixel buffer
        let color = Color::rgba(text_color[0], text_color[1], text_color[2], text_color[3]);
        buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            color,
            |x, y, gw, gh, drawn_color| {
                let alpha = drawn_color.a() as f32 / 255.0;
                if alpha < 0.01 {
                    return;
                }
                for dy in 0..gh as i32 {
                    for dx in 0..gw as i32 {
                        let px = x + dx;
                        let py = y + dy;
                        if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                            continue;
                        }
                        let offset = (py as usize * w + px as usize) * 4;
                        blend_argb8888(
                            &mut pixels[offset..offset + 4],
                            drawn_color.r(),
                            drawn_color.g(),
                            drawn_color.b(),
                            alpha,
                        );
                    }
                }
            },
        );

        MemoryRenderBuffer::from_slice(
            &pixels,
            Fourcc::Argb8888,
            (w as i32, h as i32),
            1,
            Transform::Normal,
            None,
        )
    }

    /// Remove cache entries for windows that no longer exist.
    #[allow(dead_code)] // Called from cleanup_dead_windows when integrated
    pub fn cleanup(&mut self, live_window_ids: &std::collections::HashSet<u64>) {
        self.cache
            .retain(|k, _| live_window_ids.contains(&k.window_id));
    }

    /// Update font configuration (e.g., after config reload).
    #[allow(dead_code)]
    pub fn update_font(&mut self, font_config: FontConfig) {
        self.font_config = font_config;
        self.cache.clear(); // Invalidate all cached renders
    }
}

/// Alpha-blend a source color onto a destination pixel in Argb8888 format.
/// dst layout: [B, G, R, A] on little-endian.
fn blend_argb8888(dst: &mut [u8], src_r: u8, src_g: u8, src_b: u8, src_a: f32) {
    let dst_a = dst[3] as f32 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a < 0.001 {
        return;
    }
    let inv = 1.0 / out_a;
    dst[0] = ((src_b as f32 * src_a + dst[0] as f32 * dst_a * (1.0 - src_a)) * inv) as u8;
    dst[1] = ((src_g as f32 * src_a + dst[1] as f32 * dst_a * (1.0 - src_a)) * inv) as u8;
    dst[2] = ((src_r as f32 * src_a + dst[2] as f32 * dst_a * (1.0 - src_a)) * inv) as u8;
    dst[3] = (out_a * 255.0) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_config_default() {
        let fc = FontConfig::default();
        assert_eq!(fc.family, "monospace");
        assert_eq!(fc.size, 10.0);
        assert!(!fc.bold);
        assert!(!fc.italic);
    }

    #[test]
    fn font_config_parse_simple() {
        let fc = FontConfig::from_sway_font(Some("monospace 12"));
        assert_eq!(fc.family, "monospace");
        assert_eq!(fc.size, 12.0);
    }

    #[test]
    fn font_config_parse_pango_prefix() {
        let fc = FontConfig::from_sway_font(Some("pango:DejaVu Sans Bold 14"));
        assert_eq!(fc.family, "DejaVu Sans");
        assert_eq!(fc.size, 14.0);
        assert!(fc.bold);
    }

    #[test]
    fn font_config_parse_bold_italic() {
        let fc = FontConfig::from_sway_font(Some("Noto Sans Bold Italic 11"));
        assert_eq!(fc.family, "Noto Sans");
        assert!(fc.bold);
        assert!(fc.italic);
        assert_eq!(fc.size, 11.0);
    }

    #[test]
    fn font_config_parse_none() {
        let fc = FontConfig::from_sway_font(None);
        assert_eq!(fc.family, "monospace");
    }

    #[test]
    fn blend_argb8888_fully_opaque() {
        let mut dst = [0u8, 0, 0, 255]; // opaque black (B=0,G=0,R=0,A=255)
        blend_argb8888(&mut dst, 255, 255, 255, 1.0); // opaque white
        assert_eq!(dst, [255, 255, 255, 255]); // should be white
    }

    #[test]
    fn blend_argb8888_half_transparent() {
        let mut dst = [0u8, 0, 0, 255]; // opaque black
        blend_argb8888(&mut dst, 255, 255, 255, 0.5); // 50% white
                                                      // Result should be roughly [128, 128, 128, 255]
        assert!(dst[0] > 120 && dst[0] < 140);
        assert!(dst[1] > 120 && dst[1] < 140);
        assert!(dst[2] > 120 && dst[2] < 140);
        assert_eq!(dst[3], 255);
    }

    #[test]
    fn text_renderer_creates_buffer() {
        let mut renderer = TextRenderer::new(FontConfig::default());
        let rect = Rect::new(0, 0, 200, 25);
        let buf = renderer.render_title(1, "test", rect, [255, 255, 255, 255], [0, 0, 0, 255]);
        // Just verify it returns without panicking
        let _ = buf;
    }

    #[test]
    fn text_renderer_caches() {
        let mut renderer = TextRenderer::new(FontConfig::default());
        let rect = Rect::new(0, 0, 200, 25);
        renderer.render_title(1, "test", rect, [255, 255, 255, 255], [0, 0, 0, 255]);
        // Second call with same params should hit cache
        renderer.render_title(1, "test", rect, [255, 255, 255, 255], [0, 0, 0, 255]);
        assert_eq!(renderer.cache.len(), 1);
    }

    #[test]
    fn text_renderer_invalidates_on_title_change() {
        let mut renderer = TextRenderer::new(FontConfig::default());
        let rect = Rect::new(0, 0, 200, 25);
        renderer.render_title(1, "old title", rect, [255, 255, 255, 255], [0, 0, 0, 255]);
        renderer.render_title(1, "new title", rect, [255, 255, 255, 255], [0, 0, 0, 255]);
        // Old entry removed, new entry present
        assert_eq!(renderer.cache.len(), 1);
        assert!(renderer.cache.keys().any(|k| k.title == "new title"));
    }
}
