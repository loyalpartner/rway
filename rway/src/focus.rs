// focus.rs — 焦点管理

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::SERIAL_COUNTER;

use crate::state::RwayState;

/// 根据平铺树的聚焦路径更新键盘焦点
///
/// 从当前聚焦工作区开始，沿 focused_child 链递归找到叶子窗口，
/// 然后将键盘焦点设置到该窗口的 WlSurface。
pub fn update_focus(state: &mut RwayState) {
    let surface = find_focused_surface(state);
    let serial = SERIAL_COUNTER.next_serial();
    if let Some(keyboard) = state.seat.get_keyboard() {
        keyboard.set_focus(state, surface, serial);
    }
}

/// 从平铺树中递归找到当前聚焦的窗口 surface
fn find_focused_surface(state: &RwayState) -> Option<WlSurface> {
    let ws_id = rway_tiling::workspace::get_focused_workspace(&state.tiling)?;
    find_focused_in_node(state, ws_id)
}

fn find_focused_in_node(state: &RwayState, node_id: rway_tiling::NodeId) -> Option<WlSurface> {
    let node = state.tiling.get(node_id)?;
    match &node.data {
        rway_tiling::NodeData::Window { window_id, .. } => {
            let window = state.window_map.get(window_id)?;
            Some(window.toplevel()?.wl_surface().clone())
        }
        rway_tiling::NodeData::Container { focused_child, .. } => {
            let children = state.tiling.children(node_id);
            let idx = (*focused_child).min(children.len().saturating_sub(1));
            children
                .get(idx)
                .and_then(|&c| find_focused_in_node(state, c))
        }
        _ => {
            let children = state.tiling.children(node_id);
            children
                .first()
                .and_then(|&c| find_focused_in_node(state, c))
        }
    }
}
