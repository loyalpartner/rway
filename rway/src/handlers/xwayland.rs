// handlers/xwayland.rs — XWayland 窗口管理器处理器
//
// 实现 XwmHandler 和 XWaylandShellHandler，使 X11 应用程序能够
// 在 rway 中无缝运行。X11 窗口通过 Window::new_x11_window() 转换为
// 与 Wayland 窗口统一的 Window 类型，融入平铺布局。

use std::os::unix::io::OwnedFd;

use smithay::{
    delegate_xwayland_shell,
    desktop::Window,
    utils::{Logical, Rectangle},
    wayland::{
        selection::{
            data_device::{
                clear_data_device_selection, current_data_device_selection_userdata,
                request_data_device_client_selection, set_data_device_selection,
            },
            SelectionTarget,
        },
        xwayland_shell::{XWaylandShellHandler, XWaylandShellState},
    },
    xwayland::{
        xwm::{Reorder, ResizeEdge as X11ResizeEdge, X11Window, XwmId},
        X11Surface, X11Wm, XwmHandler,
    },
};
use tracing::{error, trace, warn};

use crate::state::RwayState;

// ─── XWaylandShellHandler ─────────────────────────────────────────────────────

impl XWaylandShellHandler for RwayState {
    fn xwayland_shell_state(&mut self) -> &mut XWaylandShellState {
        self.xwayland_shell_state
            .as_mut()
            .expect("xwayland_shell_state 尚未初始化")
    }
}

delegate_xwayland_shell!(RwayState);

// ─── XwmHandler ───────────────────────────────────────────────────────────────

impl XwmHandler for RwayState {
    fn xwm_state(&mut self, _xwm: XwmId) -> &mut X11Wm {
        self.xwm.as_mut().expect("X11Wm 尚未初始化")
    }

    fn new_window(&mut self, _xwm: XwmId, _window: X11Surface) {
        // X11 窗口创建通知；尚未映射，无需操作
    }

    fn new_override_redirect_window(&mut self, _xwm: XwmId, _window: X11Surface) {
        // override redirect 窗口创建通知（菜单、工具提示等），无需操作
    }

    fn map_window_request(&mut self, _xwm: XwmId, window: X11Surface) {
        // 标记为已映射
        if let Err(e) = window.set_mapped(true) {
            warn!("设置 X11 窗口映射状态失败: {}", e);
            return;
        }

        // Create a unified Window object and insert into tiling tree.
        // The parent container's layout determines split direction (no pending_split).
        let win = Window::new_x11_window(window.clone());
        let window_id = self.alloc_window_id();
        self.tiling.insert_window(window_id);
        self.window_map.insert(window_id, win.clone());
        self.space.map_element(win.clone(), (0, 0), false);

        // 重新计算平铺布局
        self.relayout();

        // 将平铺后的几何位置通知 X11 窗口
        if let Some(bbox) = self.space.element_bbox(&win) {
            let _ = window.configure(bbox);
        }
    }

    fn mapped_override_redirect_window(&mut self, _xwm: XwmId, window: X11Surface) {
        // override redirect 窗口（菜单、弹出窗口等）：不纳入平铺，直接浮动放置
        let location = window.geometry().loc;
        let win = Window::new_x11_window(window);
        self.space.map_element(win, location, true);
    }

    fn unmapped_window(&mut self, _xwm: XwmId, window: X11Surface) {
        // 从平铺树中移除普通 X11 窗口
        if !window.is_override_redirect() {
            let _ = window.set_mapped(false);
            if let Some((&win_id, _)) = self
                .window_map
                .iter()
                .find(|(_, w)| w.x11_surface() == Some(&window))
            {
                rway_tiling::commands::remove_window(&mut self.tiling, win_id);
                self.window_map.remove(&win_id);
                self.relayout();
            }
        }

        // 从 space 中取消映射（同时处理 override redirect 窗口）
        let maybe_elem = self
            .space
            .elements()
            .find(|e| e.x11_surface() == Some(&window))
            .cloned();
        if let Some(elem) = maybe_elem {
            self.space.unmap_elem(&elem);
        }
    }

    fn destroyed_window(&mut self, _xwm: XwmId, _window: X11Surface) {
        // 窗口销毁通知；清理工作已在 unmapped_window 中完成
    }

    fn configure_request(
        &mut self,
        _xwm: XwmId,
        window: X11Surface,
        _x: Option<i32>,
        _y: Option<i32>,
        w: Option<u32>,
        h: Option<u32>,
        _reorder: Option<Reorder>,
    ) {
        // 对于平铺窗口，忽略客户端的位置/大小请求（由合成器控制布局）
        // 对于 override redirect 窗口，允许其自由调整大小
        if window.is_override_redirect() {
            let mut geo = window.geometry();
            if let Some(w) = w {
                geo.size.w = w as i32;
            }
            if let Some(h) = h {
                geo.size.h = h as i32;
            }
            let _ = window.configure(geo);
        }
    }

    fn configure_notify(
        &mut self,
        _xwm: XwmId,
        window: X11Surface,
        geometry: Rectangle<i32, Logical>,
        _above: Option<X11Window>,
    ) {
        // 更新 override redirect 窗口在 space 中的位置
        let maybe_elem = self
            .space
            .elements()
            .find(|e| e.x11_surface() == Some(&window))
            .cloned();
        if let Some(elem) = maybe_elem {
            self.space.map_element(elem, geometry.loc, false);
        }
    }

    fn resize_request(
        &mut self,
        _xwm: XwmId,
        _window: X11Surface,
        _button: u32,
        _resize_edge: X11ResizeEdge,
    ) {
        // 平铺合成器中暂不支持 X11 客户端发起的调整大小请求
    }

    fn move_request(&mut self, _xwm: XwmId, _window: X11Surface, _button: u32) {
        // 平铺合成器中暂不支持 X11 客户端发起的移动请求
    }

    // ─── 剪贴板桥接 ─────────────────────────────────────────────────────────

    fn allow_selection_access(&mut self, xwm: XwmId, _selection: SelectionTarget) -> bool {
        // 当焦点在 X11 窗口上时允许选择访问
        if let Some(keyboard) = self.seat.get_keyboard() {
            if let Some(focused) = keyboard.current_focus() {
                // 检查聚焦的 surface 是否属于此 XWM 管理的 X11 窗口
                for win in self.window_map.values() {
                    if let Some(x11) = win.x11_surface() {
                        if x11.wl_surface().as_ref() == Some(&focused) && x11.xwm_id() == Some(xwm)
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn send_selection(
        &mut self,
        _xwm: XwmId,
        selection: SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
    ) {
        match selection {
            SelectionTarget::Clipboard => {
                if let Err(err) = request_data_device_client_selection(&self.seat, mime_type, fd) {
                    error!(?err, "请求 Wayland 剪贴板给 XWayland 失败");
                }
            }
            SelectionTarget::Primary => {
                // 主选择（中键粘贴）暂未实现 PrimarySelectionHandler，记录日志
                trace!("X11 请求主选择发送，但 PrimarySelectionHandler 尚未实现");
                drop((mime_type, fd));
            }
        }
    }

    fn new_selection(&mut self, _xwm: XwmId, selection: SelectionTarget, mime_types: Vec<String>) {
        trace!(?selection, ?mime_types, "收到来自 X11 的选择");
        match selection {
            SelectionTarget::Clipboard => {
                set_data_device_selection(&self.display_handle, &self.seat, mime_types, ());
            }
            SelectionTarget::Primary => {
                // 主选择（中键粘贴）暂未实现 PrimarySelectionHandler
                trace!("X11 设置主选择，但 PrimarySelectionHandler 尚未实现");
            }
        }
    }

    fn cleared_selection(&mut self, _xwm: XwmId, selection: SelectionTarget) {
        match selection {
            SelectionTarget::Clipboard => {
                if current_data_device_selection_userdata(&self.seat).is_some() {
                    clear_data_device_selection(&self.display_handle, &self.seat);
                }
            }
            SelectionTarget::Primary => {
                // 主选择（中键粘贴）暂未实现 PrimarySelectionHandler
            }
        }
    }
}
