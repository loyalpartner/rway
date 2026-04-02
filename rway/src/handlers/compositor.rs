// handlers/compositor.rs — CompositorHandler、BufferHandler、ShmHandler

use crate::{grabs::resize_grab, handlers::xdg_shell, state::ClientState, state::RwayState};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_shm,
    desktop::layer_map_for_output,
    reexports::wayland_server::{
        protocol::{wl_buffer, wl_surface::WlSurface},
        Client,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler,
            CompositorState,
        },
        shm::{ShmHandler, ShmState},
    },
};

impl CompositorHandler for RwayState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        // 先尝试普通 Wayland 客户端，再尝试 XWayland 客户端
        if let Some(state) = client.get_data::<ClientState>() {
            return &state.compositor_state;
        }
        #[cfg(feature = "xwayland")]
        if let Some(state) = client.get_data::<smithay::xwayland::XWaylandClientData>() {
            return &state.compositor_state;
        }
        panic!("未知客户端类型：既不是 Wayland 也不是 XWayland 客户端");
    }

    fn commit(&mut self, surface: &WlSurface) {
        // 1. 处理缓冲区提交（释放旧缓冲区等）
        on_commit_buffer_handler::<Self>(surface);

        // 2. 若不是同步子表面，向上查找根表面并触发 on_commit
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self.space.elements().find(|w| {
                w.toplevel()
                    .map(|t| t.wl_surface() == &root)
                    .unwrap_or(false)
            }) {
                window.on_commit();
            }
        }

        // 3. Handle XDG Shell initial configure
        xdg_shell::handle_commit(&mut self.popups, &self.space, surface);

        // 4. Handle resize grab post-commit logic
        resize_grab::handle_commit(&mut self.space, surface);

        // 5. Handle layer surface commit: rearrange layer layout
        if let Some(output) = self.space.outputs().next().cloned() {
            let mut layer_map = layer_map_for_output(&output);
            layer_map.arrange();
        }

        // 6. Schedule redraw — client submitted new content
        self.needs_redraw = true;
    }
}

impl BufferHandler for RwayState {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for RwayState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_compositor!(RwayState);
delegate_shm!(RwayState);
