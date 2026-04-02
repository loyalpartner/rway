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
        element::solid::SolidColorRenderElement, gles::GlesRenderer, ImportAll, ImportDmaWl,
        ImportEgl, ImportMem, ImportMemWl, Renderer,
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

/// Generate all overlay render elements (borders + cursor) for one frame.
///
/// Both winit and udev backends call this to get non-Space elements.
/// Returns SolidColorRenderElements that work with any Renderer.
///
/// Elements are in z-order: cursor first (highest), then borders.
pub(crate) fn overlay_elements(
    state: &RwayState,
    scale: Scale<f64>,
    border_config: &BorderConfig,
) -> Vec<SolidColorRenderElement> {
    let window_count = state.space.elements().count();
    let mut elements = Vec::with_capacity(window_count * 4 + 1);

    // Cursor (highest z-order)
    elements.extend(cursor_elements(state, scale));

    // Borders
    elements.extend(border_elements(state, border_config, scale));

    elements
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
