// handlers/decoration.rs — XdgDecorationHandler：强制服务端装饰
//
// 平铺窗口管理器由合成器控制边框，客户端不需要画自己的标题栏。
// 所有窗口默认使用 ServerSide 模式。

use smithay::{
    delegate_xdg_decoration,
    reexports::wayland_protocols::xdg::decoration::{
        self as xdg_decoration,
        zv1::server::zxdg_toplevel_decoration_v1::Mode as DecorationMode,
    },
    wayland::shell::xdg::{
        decoration::XdgDecorationHandler,
        ToplevelSurface,
    },
};

use crate::state::RwayState;

impl XdgDecorationHandler for RwayState {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        // 平铺 WM：强制服务端装饰
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, _mode: DecorationMode) {
        // 无论客户端请求什么，都使用服务端装饰
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }
}

delegate_xdg_decoration!(RwayState);
