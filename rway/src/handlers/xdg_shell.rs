// handlers/xdg_shell.rs — XdgShellHandler：处理顶层窗口和弹窗

use smithay::{
    delegate_xdg_shell,
    desktop::{
        find_popup_root_surface, get_popup_toplevel_coords, PopupKind, PopupManager, Space, Window,
    },
    input::{
        pointer::{Focus, GrabStartData as PointerGrabStartData},
        Seat,
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            protocol::{wl_seat, wl_surface::WlSurface},
            Resource,
        },
    },
    utils::{Rectangle, Serial},
    wayland::{
        compositor::with_states,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
    },
};

use crate::{
    grabs::{MoveSurfaceGrab, ResizeSurfaceGrab},
    state::RwayState,
};

impl XdgShellHandler for RwayState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let wl_surface = surface.wl_surface().clone();
        let window = Window::new_wayland_window(surface);

        // 确保有活跃输出上的工作区可用
        self.ensure_active_workspace();

        // Allocate window ID and insert into tiling tree.
        // The parent container's layout determines split direction (no pending_split).
        let window_id = self.alloc_window_id();
        self.tiling.insert_window(window_id);
        self.window_map.insert(window_id, window.clone());

        // 映射到 Space 并提升到顶层
        self.space.map_element(window, (0, 0), true);

        // 重新计算布局
        self.relayout();

        // 新窗口自动获得键盘焦点（与 Sway 行为一致）
        let serial = smithay::utils::SERIAL_COUNTER.next_serial();
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.set_focus(self, Some(wl_surface), serial);
        }
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        self.unconstrain_popup(&surface);
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            let geometry = positioner.get_geometry();
            state.geometry = geometry;
            state.positioner = positioner;
        });
        self.unconstrain_popup(&surface);
        surface.send_repositioned(token);
    }

    fn move_request(&mut self, surface: ToplevelSurface, seat: wl_seat::WlSeat, serial: Serial) {
        let Some(seat) = Seat::from_resource(&seat) else {
            return;
        };
        let wl_surface = surface.wl_surface();

        if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
            let Some(pointer) = seat.get_pointer() else {
                return;
            };

            let Some(window) = self
                .space
                .elements()
                .find(|w| {
                    w.toplevel()
                        .map(|t| t.wl_surface() == wl_surface)
                        .unwrap_or(false)
                })
                .cloned()
            else {
                return;
            };
            let Some(initial_window_location) = self.space.element_location(&window) else {
                return;
            };

            let grab = MoveSurfaceGrab {
                start_data,
                window,
                initial_window_location,
            };

            pointer.set_grab(self, grab, serial, Focus::Clear);
        }
    }

    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        seat: wl_seat::WlSeat,
        serial: Serial,
        edges: xdg_toplevel::ResizeEdge,
    ) {
        let Some(seat) = Seat::from_resource(&seat) else {
            return;
        };
        let wl_surface = surface.wl_surface();

        if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
            let Some(pointer) = seat.get_pointer() else {
                return;
            };

            let Some(window) = self
                .space
                .elements()
                .find(|w| {
                    w.toplevel()
                        .map(|t| t.wl_surface() == wl_surface)
                        .unwrap_or(false)
                })
                .cloned()
            else {
                return;
            };
            let Some(initial_window_location) = self.space.element_location(&window) else {
                return;
            };
            let initial_window_size = window.geometry().size;

            surface.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Resizing);
            });
            surface.send_pending_configure();

            let Some(grab) = ResizeSurfaceGrab::start(
                start_data,
                window,
                edges.into(),
                Rectangle::new(initial_window_location, initial_window_size),
            ) else {
                return;
            };

            pointer.set_grab(self, grab, serial, Focus::Clear);
        }
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // TODO：弹窗抓取
    }
}

delegate_xdg_shell!(RwayState);

/// 检查座位上的指针是否持有对应序列号的抓取
fn check_grab(
    seat: &Seat<RwayState>,
    surface: &WlSurface,
    serial: Serial,
) -> Option<PointerGrabStartData<RwayState>> {
    let pointer = seat.get_pointer()?;

    // 确认该表面有点击抓取
    if !pointer.has_grab(serial) {
        return None;
    }

    let start_data = pointer.grab_start_data()?;
    let (focus, _) = start_data.focus.as_ref()?;

    // 焦点必须属于同一客户端
    if !focus.id().same_client_as(&surface.id()) {
        return None;
    }

    Some(start_data)
}

/// Called on `WlSurface::commit`: handle initial configure and popup commits
pub(crate) fn handle_commit(popups: &mut PopupManager, space: &Space<Window>, surface: &WlSurface) {
    // 处理顶层窗口的初始 configure
    if let Some(window) = space
        .elements()
        .find(|w| {
            w.toplevel()
                .map(|t| t.wl_surface() == surface)
                .unwrap_or(false)
        })
        .cloned()
    {
        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .expect("XdgToplevelSurfaceData missing")
                .lock()
                .expect("XdgToplevelSurfaceData poisoned")
                .initial_configure_sent
        });

        if !initial_configure_sent {
            if let Some(toplevel) = window.toplevel() {
                toplevel.send_configure();
            }
        }
    }

    // 处理弹窗提交
    popups.commit(surface);
    if let Some(popup) = popups.find_popup(surface) {
        match popup {
            PopupKind::Xdg(ref xdg) => {
                if !xdg.is_initial_configure_sent() {
                    xdg.send_configure().expect("弹窗初始配置失败");
                }
            }
            PopupKind::InputMethod(ref _input_method) => {}
        }
    }
}

impl RwayState {
    /// 将弹窗约束在父窗口所在的输出范围内
    fn unconstrain_popup(&self, popup: &PopupSurface) {
        let Ok(root) = find_popup_root_surface(&PopupKind::Xdg(popup.clone())) else {
            return;
        };
        let Some(window) = self.space.elements().find(|w| {
            w.toplevel()
                .map(|t| t.wl_surface() == &root)
                .unwrap_or(false)
        }) else {
            return;
        };

        let Some(output) = self.space.outputs().next() else {
            return;
        };
        let Some(output_geo) = self.space.output_geometry(output) else {
            return;
        };
        let Some(window_geo) = self.space.element_geometry(window) else {
            return;
        };

        // 目标几何是相对于父窗口的
        let mut target = output_geo;
        target.loc -= get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));
        target.loc -= window_geo.loc;

        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }
}
