// handlers/layer_shell.rs — WlrLayerShellHandler：处理 layer shell 客户端（waybar、swaybg 等）

use smithay::{
    delegate_layer_shell,
    desktop::{layer_map_for_output, LayerSurface},
    output::Output,
    reexports::wayland_server::protocol::wl_output,
    wayland::shell::wlr_layer::{
        Layer, LayerSurface as WlrLayerSurface, WlrLayerShellHandler, WlrLayerShellState,
    },
};

use crate::state::RwayState;

impl WlrLayerShellHandler for RwayState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        output: Option<wl_output::WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        tracing::info!("New layer surface: namespace={}", namespace);
        // 确定目标输出：使用请求的输出，或默认使用第一个输出
        let target_output = output
            .as_ref()
            .and_then(Output::from_resource)
            .or_else(|| self.space.outputs().next().cloned());

        let Some(output) = target_output else {
            tracing::warn!("layer surface 请求无可用输出: {}", namespace);
            return;
        };

        // Map layer surface and arrange to send initial configure
        let mut layer_map = layer_map_for_output(&output);
        let layer_surface = LayerSurface::new(surface, namespace);
        if let Err(e) = layer_map.map_layer(&layer_surface) {
            tracing::warn!("Failed to map layer surface: {}", e);
        }
        layer_map.arrange();
        drop(layer_map);

        // 重新布局（exclusive zone 可能变化）
        self.relayout();
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        // 从所有输出的 layer map 中查找并移除
        for output in self.space.outputs().cloned().collect::<Vec<_>>() {
            let layer_map = layer_map_for_output(&output);
            // 先查找目标 layer surface 并克隆
            let found = layer_map
                .layers()
                .find(|l| l.layer_surface() == &surface)
                .cloned();
            drop(layer_map);

            if let Some(layer_surface) = found {
                let mut layer_map = layer_map_for_output(&output);
                layer_map.unmap_layer(&layer_surface);
                drop(layer_map);
                self.relayout();
                return;
            }
        }
    }
}

delegate_layer_shell!(RwayState);
