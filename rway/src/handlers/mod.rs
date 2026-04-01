// handlers/mod.rs — SeatHandler、SelectionHandler、DataDeviceHandler、OutputHandler

pub mod compositor;
pub mod decoration;
pub mod layer_shell;
pub mod xdg_shell;
pub mod output;

use crate::state::RwayState;

//
// Wl Seat
//

use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Resource;
use smithay::wayland::output::OutputHandler;
use smithay::wayland::selection::data_device::{
    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
    ServerDndGrabHandler,
};
use smithay::wayland::selection::SelectionHandler;
use smithay::{delegate_data_device, delegate_output, delegate_seat};

impl SeatHandler for RwayState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<RwayState> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let dh = &self.display_handle;
        let client = focused.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, client);
    }
}

delegate_seat!(RwayState);

//
// Wl Data Device（剪贴板与拖拽）
//

impl SelectionHandler for RwayState {
    type SelectionUserData = ();
}

impl ClientDndGrabHandler for RwayState {}
impl ServerDndGrabHandler for RwayState {}

impl DataDeviceHandler for RwayState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

delegate_data_device!(RwayState);

//
// Wl Output & Xdg Output
//

impl OutputHandler for RwayState {}
delegate_output!(RwayState);
