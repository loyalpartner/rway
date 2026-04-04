// cursor.rs — Cursor rendering with xcursor theme support
//
// Loads the system xcursor theme (respects XCURSOR_THEME and XCURSOR_SIZE).
// Supports multiple cursor shapes (default, text, pointer, etc.) via
// wp_cursor_shape protocol or Named cursor status.

use std::{collections::HashMap, io::Read};

use smithay::{
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            surface::WaylandSurfaceRenderElement,
            Kind,
        },
        ImportAll, ImportMem, Renderer,
    },
    input::pointer::CursorImageStatus,
    render_elements,
    utils::{Physical, Point, Scale},
};

render_elements! {
    pub PointerRenderElement<R> where R: ImportAll + ImportMem;
    Surface=WaylandSurfaceRenderElement<R>,
    Memory=MemoryRenderBufferRenderElement<R>,
}

impl<R: Renderer> std::fmt::Debug for PointerRenderElement<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surface(s) => f.debug_tuple("Surface").field(s).finish(),
            Self::Memory(m) => f.debug_tuple("Memory").field(m).finish(),
            Self::_GenericCatcher(c) => f.debug_tuple("_GenericCatcher").field(c).finish(),
        }
    }
}

/// Xcursor theme with cached shapes.
pub struct XCursor {
    theme: xcursor::CursorTheme,
    cache: HashMap<String, Vec<xcursor::parser::Image>>,
    size: u32,
}

impl XCursor {
    pub fn load() -> Self {
        let theme_name = std::env::var("XCURSOR_THEME")
            .ok()
            .unwrap_or_else(|| "default".into());
        let size = std::env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        let theme = xcursor::CursorTheme::load(&theme_name);
        let mut cursor = XCursor {
            theme,
            cache: HashMap::new(),
            size,
        };
        // Pre-load default cursor
        cursor.ensure_loaded("default");
        cursor
    }

    fn ensure_loaded(&mut self, name: &str) {
        if self.cache.contains_key(name) {
            return;
        }
        let images = load_icon(&self.theme, name).unwrap_or_else(|_| {
            // Try fallback names
            let fallback = cursor_icon_fallback(name);
            for fb in fallback {
                if let Ok(imgs) = load_icon(&self.theme, fb) {
                    return imgs;
                }
            }
            if name != "default" {
                // Fall back to default cursor
                if let Some(default) = self.cache.get("default") {
                    return default.clone();
                }
            }
            fallback_arrow()
        });
        self.cache.insert(name.to_string(), images);
    }

    fn get_frame(&mut self, name: &str, scale: u32, millis: u32) -> xcursor::parser::Image {
        self.ensure_loaded(name);
        let images = self.cache.get(name).unwrap();
        let target_size = self.size * scale;
        let nearest = images
            .iter()
            .min_by_key(|img| (target_size as i32 - img.size as i32).abs())
            .unwrap();
        let matching: Vec<&xcursor::parser::Image> = images
            .iter()
            .filter(|img| img.width == nearest.width && img.height == nearest.height)
            .collect();
        let total: u32 = matching.iter().map(|img| img.delay).sum();
        if total == 0 {
            return matching[0].clone();
        }
        let mut remaining = millis % total;
        for img in &matching {
            if remaining < img.delay {
                return (*img).clone();
            }
            remaining -= img.delay;
        }
        matching[0].clone()
    }

    pub fn render(
        &mut self,
        name: &str,
        scale: u32,
        millis: u32,
    ) -> (MemoryRenderBuffer, Point<i32, Physical>) {
        let img = self.get_frame(name, scale, millis);
        let hotspot: Point<i32, Physical> = (img.xhot as i32, img.yhot as i32).into();
        let buffer = MemoryRenderBuffer::from_slice(
            &img.pixels_rgba,
            smithay::backend::allocator::Fourcc::Abgr8888,
            (img.width as i32, img.height as i32),
            1,
            smithay::utils::Transform::Normal,
            None,
        );
        (buffer, hotspot)
    }
}

/// Map CursorIcon names to xcursor theme icon names.
fn cursor_icon_to_name(icon: &smithay::input::pointer::CursorIcon) -> &'static str {
    use smithay::input::pointer::CursorIcon;
    match icon {
        CursorIcon::Default => "default",
        CursorIcon::Pointer => "pointer",
        CursorIcon::Text => "text",
        CursorIcon::Crosshair => "crosshair",
        CursorIcon::Move => "move",
        CursorIcon::Wait => "wait",
        CursorIcon::Help => "help",
        CursorIcon::Progress => "progress",
        CursorIcon::NotAllowed => "not-allowed",
        CursorIcon::ContextMenu => "context-menu",
        CursorIcon::Cell => "cell",
        CursorIcon::VerticalText => "vertical-text",
        CursorIcon::Alias => "alias",
        CursorIcon::Copy => "copy",
        CursorIcon::NoDrop => "no-drop",
        CursorIcon::Grab => "grab",
        CursorIcon::Grabbing => "grabbing",
        CursorIcon::AllScroll => "all-scroll",
        CursorIcon::ZoomIn => "zoom-in",
        CursorIcon::ZoomOut => "zoom-out",
        CursorIcon::ColResize => "col-resize",
        CursorIcon::RowResize => "row-resize",
        CursorIcon::NResize => "n-resize",
        CursorIcon::EResize => "e-resize",
        CursorIcon::SResize => "s-resize",
        CursorIcon::WResize => "w-resize",
        CursorIcon::NeResize => "ne-resize",
        CursorIcon::NwResize => "nw-resize",
        CursorIcon::SeResize => "se-resize",
        CursorIcon::SwResize => "sw-resize",
        CursorIcon::EwResize => "ew-resize",
        CursorIcon::NsResize => "ns-resize",
        CursorIcon::NeswResize => "nesw-resize",
        CursorIcon::NwseResize => "nwse-resize",
        _ => "default",
    }
}

/// Fallback xcursor names for common cursor types.
fn cursor_icon_fallback(name: &str) -> &[&str] {
    match name {
        "default" => &["left_ptr", "arrow"],
        "pointer" => &["hand2", "hand1", "pointing_hand"],
        "text" => &["xterm", "ibeam"],
        "move" => &["fleur", "grabbing"],
        "crosshair" => &["cross"],
        "wait" => &["watch"],
        "not-allowed" => &["crossed_circle", "forbidden"],
        "grab" => &["openhand", "hand1"],
        "grabbing" => &["closedhand", "fleur"],
        "n-resize" | "s-resize" | "ns-resize" => &["sb_v_double_arrow", "v_double_arrow"],
        "e-resize" | "w-resize" | "ew-resize" => &["sb_h_double_arrow", "h_double_arrow"],
        "ne-resize" | "sw-resize" | "nesw-resize" => &["fd_double_arrow"],
        "nw-resize" | "se-resize" | "nwse-resize" => &["bd_double_arrow"],
        "col-resize" => &["sb_h_double_arrow"],
        "row-resize" => &["sb_v_double_arrow"],
        "all-scroll" => &["fleur"],
        "help" => &["question_arrow"],
        "progress" => &["left_ptr_watch"],
        _ => &["left_ptr"],
    }
}

fn load_icon(
    theme: &xcursor::CursorTheme,
    name: &str,
) -> Result<Vec<xcursor::parser::Image>, CursorError> {
    let path = theme.load_icon(name).ok_or(CursorError::NotFound)?;
    let mut file = std::fs::File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    xcursor::parser::parse_xcursor(&data).ok_or(CursorError::Parse)
}

fn fallback_arrow() -> Vec<xcursor::parser::Image> {
    let size = 16u32;
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let arrow: &[(u32, u32)] = &[
        (0, 0),
        (0, 1),
        (1, 1),
        (0, 2),
        (1, 2),
        (2, 2),
        (0, 3),
        (1, 3),
        (2, 3),
        (3, 3),
        (0, 4),
        (1, 4),
        (2, 4),
        (3, 4),
        (4, 4),
        (0, 5),
        (1, 5),
        (2, 5),
        (3, 5),
        (4, 5),
        (5, 5),
        (0, 6),
        (1, 6),
        (2, 6),
        (3, 6),
        (4, 6),
        (5, 6),
        (6, 6),
        (0, 7),
        (1, 7),
        (2, 7),
        (3, 7),
        (4, 7),
        (5, 7),
        (6, 7),
        (7, 7),
        (0, 8),
        (1, 8),
        (2, 8),
        (3, 8),
        (4, 8),
        (0, 9),
        (1, 9),
        (2, 9),
        (3, 9),
        (0, 10),
        (1, 10),
        (3, 10),
        (4, 10),
        (0, 11),
        (1, 11),
        (4, 11),
        (5, 11),
        (0, 12),
        (5, 12),
        (6, 12),
        (6, 13),
    ];
    for &(x, y) in arrow {
        let idx = ((y * size + x) * 4) as usize;
        if idx + 3 < pixels.len() {
            pixels[idx] = 255;
            pixels[idx + 1] = 255;
            pixels[idx + 2] = 255;
            pixels[idx + 3] = 255;
        }
    }
    vec![xcursor::parser::Image {
        size,
        width: size,
        height: size,
        xhot: 0,
        yhot: 0,
        delay: 0,
        pixels_rgba: pixels,
        pixels_argb: vec![],
    }]
}

#[derive(thiserror::Error, Debug)]
enum CursorError {
    #[error("cursor theme has no cursor")]
    NotFound,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse xcursor file")]
    Parse,
}

/// Render the appropriate cursor element based on current status.
pub fn render_cursor_element<R>(
    renderer: &mut R,
    cursor: &mut XCursor,
    cursor_status: &CursorImageStatus,
    location: Point<f64, smithay::utils::Logical>,
    scale: Scale<f64>,
    millis: u32,
) -> Vec<PointerRenderElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    match cursor_status {
        CursorImageStatus::Hidden => vec![],
        CursorImageStatus::Named(icon) => {
            let name = cursor_icon_to_name(icon);
            let (buffer, hotspot) = cursor.render(name, 1, millis);
            let pos = Point::<i32, Physical>::from((
                (location.x as i32) - hotspot.x,
                (location.y as i32) - hotspot.y,
            ))
            .to_f64();
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                pos,
                &buffer,
                None,
                None,
                None,
                Kind::Cursor,
            ) {
                vec![PointerRenderElement::Memory(elem)]
            } else {
                vec![]
            }
        }
        CursorImageStatus::Surface(surface) => {
            let hotspot = smithay::wayland::compositor::with_states(surface, |states| {
                states
                    .data_map
                    .get::<smithay::input::pointer::CursorImageSurfaceData>()
                    .map(|d| d.lock().unwrap().hotspot)
                    .unwrap_or_default()
            });
            let pos = (location - hotspot.to_f64()).to_physical_precise_round(scale);
            smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
                renderer,
                surface,
                pos,
                scale,
                1.0,
                Kind::Cursor,
            )
            .into_iter()
            .map(PointerRenderElement::Surface)
            .collect()
        }
    }
}
