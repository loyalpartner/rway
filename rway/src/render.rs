// render.rs — Unified render pipeline
//
// Following cosmic-comp's architecture:
// - RwayRenderer trait for unified renderer capability bounds
// - overlay_elements() generates borders + cursor in one place
// - Each backend calls overlay_elements() then submits via its own method
//
// cosmic-comp calls output_elements() for KMS and render_output() for winit,
// both sharing workspace_elements(). We follow the same pattern: shared element
// generation, backend-specific frame submission.

use smithay::{
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            solid::SolidColorRenderElement,
            Kind,
        },
        gles::GlesRenderer,
        ImportAll, ImportDmaWl, ImportEgl, ImportMem, ImportMemWl, Renderer,
    },
    input::pointer::CursorImageStatus,
    utils::Scale,
};

use crate::{border::BorderConfig, state::RwayState};

/// Background clear color for all outputs (dark gray, fully opaque).
pub(crate) const CLEAR_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

// ── RwayRenderer trait ──

/// Unified renderer trait for rway (modeled after cosmic-comp's AsGlowRenderer).
///
/// All shared rendering functions use `R: RwayRenderer` as their bound.
/// Provides downcast to GlesRenderer for future cursor texture upload / custom shaders.
#[allow(dead_code)] // Used by impls now; will be a function bound when cursor uses textures
pub(crate) trait RwayRenderer:
    Renderer + ImportAll + ImportMem + ImportMemWl + ImportEgl + ImportDmaWl
{
    fn gles_renderer(&self) -> &GlesRenderer;
    fn gles_renderer_mut(&mut self) -> &mut GlesRenderer;
}

impl RwayRenderer for GlesRenderer {
    fn gles_renderer(&self) -> &GlesRenderer {
        self
    }
    fn gles_renderer_mut(&mut self) -> &mut GlesRenderer {
        self
    }
}

// impl RwayRenderer for UdevRenderer<'_> is in backend/udev.rs

// ── Shared element generation ──

/// Pre-rendered overlay data. Solid elements need no renderer to create;
/// text buffers need `&mut Renderer` to materialize into RenderElements.
pub(crate) struct OverlayOutput {
    pub solid: Vec<SolidColorRenderElement>,
    pub text_buffers: Vec<(MemoryRenderBuffer, (i32, i32))>,
}

/// Generate all overlay elements for one frame.
///
/// Solid elements (borders, cursor, title bar backgrounds) are ready to use.
/// Text buffers require `materialize_text_elements()` with a renderer.
pub(crate) fn overlay_elements(
    state: &mut RwayState,
    scale: Scale<f64>,
    border_config: &BorderConfig,
) -> OverlayOutput {
    let window_count = state.space.elements().count();
    let mut solid = Vec::with_capacity(window_count * 4 + 1);

    // Cursor (highest z-order)
    solid.extend(cursor_elements(state, scale));

    // Borders
    solid.extend(border_elements(state, border_config, scale));

    // Title bar text (pre-rendered to MemoryRenderBuffer with background color baked in)
    let text_buffers = title_bar_text_buffers(state, border_config);

    OverlayOutput {
        solid,
        text_buffers,
    }
}

/// Convert pre-rendered text buffers into RenderElements using a renderer.
pub(crate) fn materialize_text_elements<R>(
    renderer: &mut R,
    text_buffers: &[(MemoryRenderBuffer, (i32, i32))],
    _scale: Scale<f64>,
) -> Vec<MemoryRenderBufferRenderElement<R>>
where
    R: Renderer + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    text_buffers
        .iter()
        .filter_map(|(buf, (x, y))| {
            let loc = (*x as f64, *y as f64);
            MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                loc,
                buf,
                None,
                None,
                None,
                Kind::Unspecified,
            )
            .ok()
        })
        .collect()
}

/// Generate cursor placeholder element at current pointer position.
fn cursor_elements(state: &RwayState, scale: Scale<f64>) -> Vec<SolidColorRenderElement> {
    if matches!(state.cursor_status, CursorImageStatus::Hidden) {
        return vec![];
    }

    let Some(pointer) = state.seat.get_pointer() else {
        return vec![];
    };

    vec![crate::cursor::cursor_square_element(
        pointer.current_location(),
        scale,
    )]
}

/// Generate border elements for all windows mapped in the Space.
fn border_elements(
    state: &RwayState,
    config: &BorderConfig,
    scale: Scale<f64>,
) -> Vec<SolidColorRenderElement> {
    let bw = config.width;
    if bw <= 0 {
        return vec![];
    }

    // If any window is fullscreen, skip ALL borders
    let has_fullscreen = state.space.elements().any(|window| {
        window.toplevel().is_some_and(|tl| {
            use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
            tl.with_pending_state(|s| s.states.contains(xdg_toplevel::State::Fullscreen))
        })
    });
    if has_fullscreen {
        return vec![];
    }

    let focused_surface = state.seat.get_keyboard().and_then(|kb| kb.current_focus());

    state
        .space
        .elements()
        .flat_map(|window| {
            let is_focused = focused_surface
                .as_ref()
                .is_some_and(|fs| window.toplevel().is_some_and(|tl| tl.wl_surface() == fs));

            let color = if is_focused {
                config.focused_color
            } else {
                config.unfocused_color
            };

            let Some(content_geo) = state.space.element_geometry(window) else {
                return vec![];
            };

            // Reconstruct container rect (content rect is inset by border_width)
            let container_geo = smithay::utils::Rectangle::new(
                (content_geo.loc.x - bw, content_geo.loc.y - bw).into(),
                (content_geo.size.w + 2 * bw, content_geo.size.h + 2 * bw).into(),
            );

            crate::border::window_borders(container_geo, color, bw, scale)
        })
        .collect()
}

/// Generate title bar text as pre-rendered MemoryRenderBuffers.
fn title_bar_text_buffers(
    state: &mut RwayState,
    config: &BorderConfig,
) -> Vec<(MemoryRenderBuffer, (i32, i32))> {
    let bars = state.cached_title_bars.clone();
    if bars.is_empty() {
        return vec![];
    }

    let text_color_focused = [0xFFu8, 0xFF, 0xFF, 0xFF];
    let text_color_unfocused = [0xCC, 0xCC, 0xCC, 0xFF];
    let fc = config.focused_color;
    let bg_focused = [
        (fc[0] * 255.0) as u8,
        (fc[1] * 255.0) as u8,
        (fc[2] * 255.0) as u8,
        (fc[3] * 255.0) as u8,
    ];
    let uc = config.unfocused_color;
    let bg_unfocused = [
        (uc[0] * 0.7 * 255.0) as u8,
        (uc[1] * 0.7 * 255.0) as u8,
        (uc[2] * 0.7 * 255.0) as u8,
        (uc[3] * 255.0) as u8,
    ];

    let mut result = Vec::with_capacity(bars.len());
    for bar in &bars {
        let title = get_window_title(state, bar.window_id);
        let (text_color, bg_color) = if bar.focused {
            (text_color_focused, bg_focused)
        } else {
            (text_color_unfocused, bg_unfocused)
        };
        let buf = state.text_renderer.render_title(
            bar.window_id,
            &title,
            bar.rect,
            bar.focused,
            text_color,
            bg_color,
        );
        result.push((buf.clone(), (bar.rect.x, bar.rect.y)));
    }
    result
}

/// Get window title from the Wayland toplevel surface.
fn get_window_title(state: &RwayState, window_id: u64) -> String {
    state
        .window_map
        .get(&window_id)
        .and_then(|window| window.toplevel())
        .map(|tl| {
            smithay::wayland::compositor::with_states(tl.wl_surface(), |states| {
                use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .and_then(|data| data.lock().ok())
                    .and_then(|attrs| attrs.title.clone())
                    .unwrap_or_default()
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_color_is_valid_rgba() {
        for &c in &CLEAR_COLOR[..3] {
            assert!((0.0..=1.0).contains(&c));
        }
        assert_eq!(CLEAR_COLOR[3], 1.0);
    }
}
