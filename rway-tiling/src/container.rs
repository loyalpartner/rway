// Container utilities
//
// Most container operations have been migrated to impl Tree methods in tree.rs.
// Legacy delegation wrappers are provided for backward compatibility.

use crate::tree::{Layout, NodeData, NodeId, Tree};

/// Delegation: append a 1.0 size entry to a container
pub fn container_add_child(tree: &mut Tree, container_id: NodeId) {
    let _ = tree.add_container_size(container_id);
}

/// Delegation: remove a size entry and adjust focused_child
pub fn container_remove_child(tree: &mut Tree, container_id: NodeId, child_index: usize) {
    let _ = tree.remove_container_size(container_id, child_index);
}

/// Create a container NodeData with two equal initial sizes
pub fn make_container(layout: Layout) -> NodeData {
    NodeData::Container {
        layout,
        sizes: vec![1.0, 1.0],
        focused_child: 0,
    }
}

/// Delegation: set container layout type
pub fn set_container_layout(tree: &mut Tree, container_id: NodeId, layout: Layout) {
    let _ = tree.set_layout(container_id, layout);
}

/// Delegation: get focused child of a container
pub fn get_focused_child(tree: &Tree, container_id: NodeId) -> Option<NodeId> {
    tree.focused_child(container_id).ok().flatten()
}

/// Delegation: set focused child of a container
pub fn set_focused_child(tree: &mut Tree, container_id: NodeId, child_id: NodeId) -> bool {
    tree.set_focused_child(container_id, child_id).is_ok()
}

/// Normalize sizes so they sum to 1.0.
/// If total is zero or negative, reset to equal distribution.
pub fn normalize_sizes(sizes: &mut [f64]) {
    let total: f64 = sizes.iter().sum();
    if total <= 0.0 {
        let n = sizes.len();
        if n > 0 {
            let eq = 1.0 / n as f64;
            for s in sizes.iter_mut() {
                *s = eq;
            }
        }
        return;
    }
    for s in sizes.iter_mut() {
        *s /= total;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_makes_sum_one() {
        let mut sizes = vec![1.0, 1.0, 2.0];
        normalize_sizes(&mut sizes);
        let sum: f64 = sizes.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn normalize_equal_splits() {
        let mut sizes = vec![1.0, 1.0];
        normalize_sizes(&mut sizes);
        assert!((sizes[0] - 0.5).abs() < 1e-9);
        assert!((sizes[1] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn normalize_zero_total_resets_to_equal() {
        let mut sizes = vec![0.0, 0.0, 0.0];
        normalize_sizes(&mut sizes);
        for s in &sizes {
            assert!((s - 1.0 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn normalize_single_element_becomes_one() {
        let mut sizes = vec![42.0];
        normalize_sizes(&mut sizes);
        assert!((sizes[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn normalize_empty_does_not_panic() {
        let mut sizes: Vec<f64> = Vec::new();
        normalize_sizes(&mut sizes);
    }
}
