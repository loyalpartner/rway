// Container types: Root, Output, Workspace, Split, Window

//! 容器辅助操作：管理 Container 节点的子节点尺寸比例、聚焦索引等。

use crate::tree::{Layout, NodeData, NodeId, Tree};

// ── 公开辅助函数 ─────────────────────────────────────────────

/// 向容器节点追加一个子节点，并分配均等的初始比例。
///
/// 如果 `container_id` 不是 `NodeData::Container`，则不执行任何操作。
pub fn container_add_child(tree: &mut Tree, container_id: NodeId) {
    if let Some(node) = tree.get_mut(container_id) {
        if let NodeData::Container { ref mut sizes, .. } = node.data {
            // 将 1.0 追加到末尾；compute_layout 会按比例使用
            sizes.push(1.0);
        }
    }
}

/// 从容器的 `sizes` 中移除指定索引对应的元素，并调整 `focused_child`。
///
/// `child_index` 是在父容器 `children` 列表中的下标（0-based）。
pub fn container_remove_child(tree: &mut Tree, container_id: NodeId, child_index: usize) {
    if let Some(node) = tree.get_mut(container_id) {
        if let NodeData::Container {
            ref mut sizes,
            ref mut focused_child,
            ..
        } = node.data
        {
            if child_index < sizes.len() {
                sizes.remove(child_index);
            }
            // 保持 focused_child 有效
            if !sizes.is_empty() && *focused_child >= sizes.len() {
                *focused_child = sizes.len() - 1;
            } else if sizes.is_empty() {
                *focused_child = 0;
            }
        }
    }
}

/// 创建一个包含两个初始等分比例的容器 NodeData。
pub fn make_container(layout: Layout) -> NodeData {
    NodeData::Container {
        layout,
        sizes: vec![1.0, 1.0],
        focused_child: 0,
    }
}

/// 将容器的布局类型替换为新类型。
pub fn set_container_layout(tree: &mut Tree, container_id: NodeId, layout: Layout) {
    if let Some(node) = tree.get_mut(container_id) {
        if let NodeData::Container { layout: ref mut l, .. } = node.data {
            *l = layout;
        }
    }
}

/// 获取容器当前聚焦的子节点 ID（通过 `focused_child` 索引从 `children` 中取值）。
pub fn get_focused_child(tree: &Tree, container_id: NodeId) -> Option<NodeId> {
    let node = tree.get(container_id)?;
    let focused_index = match &node.data {
        NodeData::Container { focused_child, .. } => *focused_child,
        _ => return None,
    };
    let children = tree.children(container_id);
    children.get(focused_index).copied()
}

/// 将容器的 `focused_child` 设置为指定的子节点 ID 对应的索引。
/// 若子节点不存在于 children 列表中，返回 `false`。
pub fn set_focused_child(tree: &mut Tree, container_id: NodeId, child_id: NodeId) -> bool {
    let children = tree.children(container_id).to_vec();
    let Some(index) = children.iter().position(|&c| c == child_id) else {
        return false;
    };

    if let Some(node) = tree.get_mut(container_id) {
        if let NodeData::Container { ref mut focused_child, .. } = node.data {
            *focused_child = index;
            return true;
        }
    }
    false
}

/// 按比例归一化 sizes，使总和为 `target_sum`（默认 1.0 风格，但这里
/// 直接保持原始比例，仅确保没有负值或零值）。
pub fn normalize_sizes(sizes: &mut Vec<f64>) {
    let total: f64 = sizes.iter().sum();
    if total <= 0.0 {
        // 所有尺寸无效，重置为均等
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

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{NodeData, NodeId, Rect, Tree};

    // ── 辅助 ───────────────────────────────────────────────

    fn make_tree_with_container(layout: Layout) -> (Tree, NodeId, NodeId) {
        let mut tree = Tree::new();
        let root = tree.root();
        let output = tree.add_node(
            root,
            NodeData::Output {
                name: "eDP-1".into(),
                geometry: Rect::new(0, 0, 1920, 1080),
            },
        );
        let container = tree.add_node(output, make_container(layout));
        (tree, root, container)
    }

    // ── make_container ──────────────────────────────────────

    #[test]
    fn make_container_creates_splith() {
        let data = make_container(Layout::SplitH);
        match data {
            NodeData::Container { layout, sizes, focused_child } => {
                assert_eq!(layout, Layout::SplitH);
                assert_eq!(sizes, vec![1.0, 1.0]);
                assert_eq!(focused_child, 0);
            }
            _ => panic!("应为 Container 类型"),
        }
    }

    #[test]
    fn make_container_creates_splitv() {
        let data = make_container(Layout::SplitV);
        assert!(matches!(data, NodeData::Container { layout: Layout::SplitV, .. }));
    }

    #[test]
    fn make_container_creates_tabbed() {
        let data = make_container(Layout::Tabbed);
        assert!(matches!(data, NodeData::Container { layout: Layout::Tabbed, .. }));
    }

    // ── container_add_child ─────────────────────────────────

    #[test]
    fn add_child_appends_to_sizes() {
        let (mut tree, _, container) = make_tree_with_container(Layout::SplitH);
        // make_container 初始有 sizes = [1.0, 1.0]
        container_add_child(&mut tree, container);
        if let Some(NodeData::Container { sizes, .. }) = tree.get(container).map(|n| &n.data) {
            assert_eq!(sizes.len(), 3);
        } else {
            panic!("节点不是 Container");
        }
    }

    #[test]
    fn add_child_to_non_container_does_nothing() {
        let mut tree = Tree::new();
        let root = tree.root(); // Root 类型，不是 Container
        // 不应 panic
        container_add_child(&mut tree, root);
    }

    // ── container_remove_child ──────────────────────────────

    #[test]
    fn remove_child_decrements_sizes() {
        let (mut tree, _, container) = make_tree_with_container(Layout::SplitH);
        container_remove_child(&mut tree, container, 0);
        if let Some(NodeData::Container { sizes, .. }) = tree.get(container).map(|n| &n.data) {
            assert_eq!(sizes.len(), 1);
        } else {
            panic!();
        }
    }

    #[test]
    fn remove_child_clamps_focused_child() {
        let (mut tree, _, container) = make_tree_with_container(Layout::SplitH);
        // focused_child = 0, sizes = [1.0, 1.0]
        // 删除最后一个后 sizes = [1.0]，focused_child 应仍为 0
        container_remove_child(&mut tree, container, 1);
        if let Some(NodeData::Container { focused_child, sizes, .. }) =
            tree.get(container).map(|n| &n.data)
        {
            assert_eq!(*focused_child, 0);
            assert_eq!(sizes.len(), 1);
        } else {
            panic!();
        }
    }

    #[test]
    fn remove_child_all_leaves_zero_focused() {
        let (mut tree, _, container) = make_tree_with_container(Layout::SplitH);
        container_remove_child(&mut tree, container, 0);
        container_remove_child(&mut tree, container, 0);
        if let Some(NodeData::Container { focused_child, sizes, .. }) =
            tree.get(container).map(|n| &n.data)
        {
            assert!(sizes.is_empty());
            assert_eq!(*focused_child, 0);
        } else {
            panic!();
        }
    }

    #[test]
    fn remove_child_out_of_bounds_does_nothing() {
        let (mut tree, _, container) = make_tree_with_container(Layout::SplitH);
        container_remove_child(&mut tree, container, 99);
        if let Some(NodeData::Container { sizes, .. }) = tree.get(container).map(|n| &n.data) {
            assert_eq!(sizes.len(), 2);
        } else {
            panic!();
        }
    }

    // ── set_container_layout ────────────────────────────────

    #[test]
    fn set_layout_changes_container_layout() {
        let (mut tree, _, container) = make_tree_with_container(Layout::SplitH);
        set_container_layout(&mut tree, container, Layout::Tabbed);
        if let Some(NodeData::Container { layout, .. }) = tree.get(container).map(|n| &n.data) {
            assert_eq!(*layout, Layout::Tabbed);
        } else {
            panic!();
        }
    }

    #[test]
    fn set_layout_on_non_container_does_nothing() {
        let mut tree = Tree::new();
        let root = tree.root();
        // 不应 panic
        set_container_layout(&mut tree, root, Layout::SplitV);
    }

    // ── get_focused_child ───────────────────────────────────

    #[test]
    fn get_focused_child_returns_first_child() {
        let mut tree = Tree::new();
        let root = tree.root();
        let container = tree.add_node(root, make_container(Layout::SplitH));
        let win1 = tree.add_node(container, NodeData::Window {
            window_id: 1,
            floating: false,
            fullscreen: false,
            geometry: Rect::new(0, 0, 100, 100),
            saved_geometry: None,
        });
        let _win2 = tree.add_node(container, NodeData::Window {
            window_id: 2,
            floating: false,
            fullscreen: false,
            geometry: Rect::new(0, 0, 100, 100),
            saved_geometry: None,
        });
        // focused_child = 0 → 指向 win1
        let focused = get_focused_child(&tree, container);
        assert_eq!(focused, Some(win1));
    }

    #[test]
    fn get_focused_child_returns_none_for_non_container() {
        let tree = Tree::new();
        let root = tree.root();
        assert!(get_focused_child(&tree, root).is_none());
    }

    #[test]
    fn get_focused_child_returns_none_for_empty_children() {
        let mut tree = Tree::new();
        let root = tree.root();
        let container = tree.add_node(root, NodeData::Container {
            layout: Layout::SplitH,
            sizes: Vec::new(),
            focused_child: 0,
        });
        assert!(get_focused_child(&tree, container).is_none());
    }

    // ── set_focused_child ───────────────────────────────────

    #[test]
    fn set_focused_child_updates_index() {
        let mut tree = Tree::new();
        let root = tree.root();
        let container = tree.add_node(root, make_container(Layout::SplitH));
        let win1 = tree.add_node(container, NodeData::Window {
            window_id: 1,
            floating: false,
            fullscreen: false,
            geometry: Rect::new(0, 0, 100, 100),
            saved_geometry: None,
        });
        let win2 = tree.add_node(container, NodeData::Window {
            window_id: 2,
            floating: false,
            fullscreen: false,
            geometry: Rect::new(0, 0, 100, 100),
            saved_geometry: None,
        });

        let ok = set_focused_child(&mut tree, container, win2);
        assert!(ok);
        assert_eq!(get_focused_child(&tree, container), Some(win2));

        // 切回 win1
        set_focused_child(&mut tree, container, win1);
        assert_eq!(get_focused_child(&tree, container), Some(win1));
    }

    #[test]
    fn set_focused_child_returns_false_for_non_member() {
        let mut tree = Tree::new();
        let root = tree.root();
        let container = tree.add_node(root, make_container(Layout::SplitH));
        let stranger = tree.add_node(root, NodeData::Window {
            window_id: 99,
            floating: false,
            fullscreen: false,
            geometry: Rect::new(0, 0, 100, 100),
            saved_geometry: None,
        });
        let ok = set_focused_child(&mut tree, container, stranger);
        assert!(!ok);
    }

    // ── normalize_sizes ─────────────────────────────────────

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
        normalize_sizes(&mut sizes); // 不应 panic
    }
}
