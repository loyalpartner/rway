// High-level commands: all operations migrated to impl Tree methods in tree.rs.
//
// These free functions are kept as thin delegation wrappers for backward
// compatibility with external callers that still use the module-level API.

pub use crate::tree::Direction;
pub use crate::tree::ResizeAxis;

use crate::tree::{Layout, NodeId, Tree};

pub fn insert_window(tree: &mut Tree, window_id: u64) -> NodeId {
    tree.insert_window(window_id)
}

pub fn remove_window(tree: &mut Tree, window_id: u64) -> bool {
    tree.remove_window(window_id)
}

pub fn move_focus(tree: &mut Tree, direction: Direction) -> bool {
    tree.move_focus(direction)
}

pub fn split(tree: &mut Tree, layout: Layout) {
    tree.split(layout)
}

pub fn toggle_floating(tree: &mut Tree, window_id: u64) -> bool {
    tree.toggle_floating(window_id)
}

pub fn find_focused_window_id(tree: &Tree) -> Option<u64> {
    tree.focused_window_id()
}

pub fn move_window(tree: &mut Tree, direction: Direction) -> bool {
    tree.move_window(direction)
}

pub fn move_to_workspace(tree: &mut Tree, target_ws: &str) -> bool {
    tree.move_to_workspace(target_ws)
}

pub fn resize_container(
    tree: &mut Tree,
    node_id: NodeId,
    axis: ResizeAxis,
    delta_ppt: f64,
) -> bool {
    tree.resize_container(node_id, axis, delta_ppt)
}

pub fn focus_parent(tree: &mut Tree) -> bool {
    tree.focus_parent()
}

pub fn focus_child(tree: &mut Tree) -> bool {
    tree.focus_child()
}

pub fn set_floating(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    tree.set_floating(window_id, enable)
}

pub fn set_fullscreen(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    tree.set_fullscreen(window_id, enable)
}

pub fn toggle_fullscreen(tree: &mut Tree, window_id: u64) -> bool {
    tree.toggle_fullscreen(window_id)
}

pub fn find_node_by_window_id(tree: &Tree, window_id: u64) -> Option<NodeId> {
    tree.find_node_by_window_id(window_id)
}

pub fn layout_toggle(tree: &mut Tree) {
    tree.layout_toggle()
}

pub fn set_focused_layout(tree: &mut Tree, layout: Layout) {
    tree.set_focused_layout(layout)
}

pub fn set_sticky(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    tree.set_sticky(window_id, enable)
}

pub fn toggle_sticky(tree: &mut Tree, window_id: u64) -> bool {
    tree.toggle_sticky(window_id)
}

pub fn add_mark(tree: &mut Tree, window_id: u64, mark: &str) -> bool {
    tree.add_mark(window_id, mark)
}

pub fn remove_mark(tree: &mut Tree, window_id: u64, mark: Option<&str>) -> bool {
    tree.remove_mark(window_id, mark)
}

pub fn get_marks(tree: &Tree, window_id: u64) -> Vec<String> {
    tree.get_marks(window_id)
}

pub fn set_fullscreen_global(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    tree.set_fullscreen_global(window_id, enable)
}

pub fn toggle_fullscreen_global(tree: &mut Tree, window_id: u64) -> bool {
    tree.toggle_fullscreen_global(window_id)
}

pub fn swap_containers(tree: &mut Tree, node_a: NodeId, node_b: NodeId) -> bool {
    tree.swap_containers(node_a, node_b)
}

pub fn focus_next_sibling(tree: &mut Tree) -> bool {
    tree.focus_next_sibling()
}

pub fn focus_prev_sibling(tree: &mut Tree) -> bool {
    tree.focus_prev_sibling()
}
