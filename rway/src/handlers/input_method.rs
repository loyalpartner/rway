// handlers/input_method.rs — InputMethodHandler: IME popup surface management

use smithay::{
    desktop::PopupKind,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Rectangle},
    wayland::input_method::{InputMethodHandler, PopupSurface},
};

use crate::state::RwayState;

impl InputMethodHandler for RwayState {
    fn new_popup(&mut self, surface: PopupSurface) {
        // Position the popup below the cursor rectangle from the start.
        let rect = surface.text_input_rectangle();
        surface.set_location((rect.loc.x, rect.loc.y + rect.size.h).into());

        if let Err(e) = self.popups.track_popup(PopupKind::InputMethod(surface)) {
            tracing::warn!("Failed to track IME popup: {:?}", e);
        }
    }

    fn dismiss_popup(&mut self, _surface: PopupSurface) {
        // PopupManager cleans up dead popups automatically.
    }

    fn popup_repositioned(&mut self, surface: PopupSurface) {
        // Position the popup below the cursor rectangle, not overlapping it.
        let rect = surface.text_input_rectangle();
        surface.set_location((rect.loc.x, rect.loc.y + rect.size.h).into());
    }

    fn parent_geometry(&self, parent: &WlSurface) -> Rectangle<i32, Logical> {
        // Return the window's LOCAL geometry (not its Space position).
        //
        // PopupKind::geometry() for InputMethod popups returns this value, and
        // the rendering formula is:
        //   offset = window.geometry().loc + popup_offset - popup.geometry().loc
        //
        // By returning window.geometry() here, window.geometry().loc cancels out
        // with popup.geometry().loc, leaving offset = text_input_rect.loc — which
        // correctly positions the popup relative to the parent surface.
        self.space
            .elements()
            .find(|w| {
                w.toplevel()
                    .map(|t| t.wl_surface() == parent)
                    .unwrap_or(false)
            })
            .map(|w| w.geometry())
            .unwrap_or_default()
    }
}
