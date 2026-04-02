// Layout algorithms: all operations migrated to impl Tree methods in tree.rs.
//
// These free functions are kept as thin delegation wrappers for backward
// compatibility with external callers that still use the module-level API.

pub use crate::tree::shrink_rect;

use crate::tree::{GapsConfig, NodeId, Rect, Tree};

pub fn compute_layout(tree: &mut Tree, node_id: NodeId, available: Rect, gaps: &GapsConfig) {
    tree.compute_layout(node_id, available, gaps)
}

pub fn get_window_geometries(tree: &Tree) -> Vec<(u64, Rect)> {
    tree.window_geometries()
}
