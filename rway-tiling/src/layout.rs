// Layout algorithms: SplitH, SplitV, Tabbed, Stacked

//! 布局算法：自顶向下递归计算树中所有 Window 节点的几何位置。

use crate::tree::{Layout, NodeData, NodeId, Rect, Tree};

/// 自顶向下递归计算从 `node_id` 开始的子树布局。
///
/// - Output/Workspace 节点：将 `available` 传递给唯一或第一个子节点。
/// - Container/SplitH：水平瓜分 `available.width`，各子节点按 `sizes` 比例分配。
/// - Container/SplitV：垂直瓜分 `available.height`，各子节点按 `sizes` 比例分配。
/// - Container/Tabbed|Stacked：只有 `focused_child` 对应的子节点得到完整区域。
/// - Window：直接将 `available` 设为该节点的 geometry。
pub fn compute_layout(tree: &mut Tree, node_id: NodeId, available: Rect) {
    // 先拷贝必要信息（避免同时持有可变/不可变引用）
    let node_data = match tree.get(node_id) {
        Some(n) => n.data.clone(),
        None => return,
    };

    match node_data {
        NodeData::Root => {
            // 根节点：依次递归所有子节点，每个子节点获得完整区域
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for child in children {
                compute_layout(tree, child, available);
            }
        }

        NodeData::Output { geometry, .. } => {
            // 输出节点使用自身的 geometry 作为可用区域
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for child in children {
                compute_layout(tree, child, geometry);
            }
        }

        NodeData::Workspace { .. } => {
            // 工作区：传递 available 给所有子节点
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for child in children {
                compute_layout(tree, child, available);
            }
        }

        NodeData::Container {
            layout,
            sizes,
            focused_child,
        } => {
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            if children.is_empty() {
                return;
            }

            match layout {
                Layout::SplitH => {
                    layout_split_h(tree, &children, &sizes, available);
                }
                Layout::SplitV => {
                    layout_split_v(tree, &children, &sizes, available);
                }
                Layout::Tabbed | Layout::Stacked => {
                    // 仅聚焦子节点可见，获得完整区域
                    let focus_idx = focused_child.min(children.len().saturating_sub(1));
                    let focused_id = children[focus_idx];
                    compute_layout(tree, focused_id, available);
                }
            }
        }

        NodeData::Window { .. } => {
            // 叶节点：直接设置 geometry
            if let Some(node) = tree.get_mut(node_id) {
                if let NodeData::Window { ref mut geometry, .. } = node.data {
                    *geometry = available;
                }
            }
        }
    }
}

/// 水平分割：沿 X 轴按 sizes 比例分配宽度
fn layout_split_h(tree: &mut Tree, children: &[NodeId], sizes: &[f64], available: Rect) {
    let total: f64 = if sizes.is_empty() {
        0.0
    } else {
        sizes.iter().sum()
    };

    let n = children.len();
    let mut x_offset = available.x;

    for (i, &child) in children.iter().enumerate() {
        let ratio = if total > 0.0 && i < sizes.len() {
            sizes[i] / total
        } else {
            1.0 / n as f64
        };

        // 最后一个子节点使用剩余空间，避免浮点舍入误差
        let child_width = if i == n - 1 {
            available.x + available.width - x_offset
        } else {
            (available.width as f64 * ratio).round() as i32
        };

        let child_rect = Rect {
            x: x_offset,
            y: available.y,
            width: child_width.max(1),
            height: available.height,
        };

        compute_layout(tree, child, child_rect);
        x_offset += child_width.max(1);
    }
}

/// 垂直分割：沿 Y 轴按 sizes 比例分配高度
fn layout_split_v(tree: &mut Tree, children: &[NodeId], sizes: &[f64], available: Rect) {
    let total: f64 = if sizes.is_empty() {
        0.0
    } else {
        sizes.iter().sum()
    };

    let n = children.len();
    let mut y_offset = available.y;

    for (i, &child) in children.iter().enumerate() {
        let ratio = if total > 0.0 && i < sizes.len() {
            sizes[i] / total
        } else {
            1.0 / n as f64
        };

        let child_height = if i == n - 1 {
            available.y + available.height - y_offset
        } else {
            (available.height as f64 * ratio).round() as i32
        };

        let child_rect = Rect {
            x: available.x,
            y: y_offset,
            width: available.width,
            height: child_height.max(1),
        };

        compute_layout(tree, child, child_rect);
        y_offset += child_height.max(1);
    }
}

/// 收集树中所有 Window 节点的 (window_id, geometry) 列表，
/// 供合成器渲染使用。
pub fn get_window_geometries(tree: &Tree) -> Vec<(u64, Rect)> {
    let mut result = Vec::new();
    collect_windows(tree, tree.root(), &mut result);
    result
}

fn collect_windows(tree: &Tree, node_id: NodeId, out: &mut Vec<(u64, Rect)>) {
    if let Some(node) = tree.get(node_id) {
        match &node.data {
            NodeData::Window {
                window_id,
                geometry,
                ..
            } => {
                out.push((*window_id, *geometry));
            }
            _ => {
                for &child in &node.children.clone() {
                    collect_windows(tree, child, out);
                }
            }
        }
    }
}

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Layout, NodeData, NodeId, Rect, Tree};

    // ── 辅助函数 ──────────────────────────────────────────────

    fn screen() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    fn add_output(tree: &mut Tree) -> NodeId {
        tree.add_node(
            tree.root(),
            NodeData::Output {
                name: "eDP-1".into(),
                geometry: screen(),
            },
        )
    }

    fn add_workspace(tree: &mut Tree, output: NodeId) -> NodeId {
        tree.add_node(
            output,
            NodeData::Workspace {
                name: "1".into(),
                output,
                is_visible: true,
            },
        )
    }

    fn add_window(tree: &mut Tree, parent: NodeId, id: u64) -> NodeId {
        tree.add_node(
            parent,
            NodeData::Window {
                window_id: id,
                floating: false,
                fullscreen: false,
                geometry: Rect::new(0, 0, 0, 0),
                saved_geometry: None,
            },
        )
    }

    fn add_container(tree: &mut Tree, parent: NodeId, layout: Layout, count: usize) -> NodeId {
        let sizes = vec![1.0; count];
        tree.add_node(
            parent,
            NodeData::Container {
                layout,
                sizes,
                focused_child: 0,
            },
        )
    }

    fn get_window_geometry(tree: &Tree, win_id: NodeId) -> Rect {
        match &tree.get(win_id).unwrap().data {
            NodeData::Window { geometry, .. } => *geometry,
            _ => panic!("非 Window 节点"),
        }
    }

    // ── 单个 Window 直接获得全屏 ──────────────────────────────

    #[test]
    fn single_window_gets_full_area() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let win = add_window(&mut tree, ws, 1);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        assert_eq!(get_window_geometry(&tree, win), screen());
    }

    // ── SplitH 等宽分割 ───────────────────────────────────────

    #[test]
    fn splith_two_equal_windows() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitH, 2);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);

        assert_eq!(g1.x, 0);
        assert_eq!(g1.width, 960);
        assert_eq!(g2.x, 960);
        // 最后一个子节点使用剩余宽度
        assert_eq!(g2.width, 960);
        assert_eq!(g1.height, 1080);
        assert_eq!(g2.height, 1080);
    }

    #[test]
    fn splith_three_equal_windows_cover_full_width() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitH, 3);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);
        let win3 = add_window(&mut tree, container, 3);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);
        let g3 = get_window_geometry(&tree, win3);

        // 三等分：640 + 640 + 640 = 1920
        let total = g1.width + g2.width + g3.width;
        assert_eq!(total, 1920);
        assert_eq!(g1.x, 0);
        assert_eq!(g2.x, g1.x + g1.width);
        assert_eq!(g3.x, g2.x + g2.width);
    }

    // ── SplitH 不等分割 ───────────────────────────────────────

    #[test]
    fn splith_unequal_sizes() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        // 比例 1:3
        let container = tree.add_node(
            ws,
            NodeData::Container {
                layout: Layout::SplitH,
                sizes: vec![1.0, 3.0],
                focused_child: 0,
            },
        );
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);

        // win1 应占 1/4 宽度 = 480
        assert_eq!(g1.width, 480);
        // win2 应占剩余 3/4 = 1440
        assert_eq!(g2.width, 1440);
        assert_eq!(g1.width + g2.width, 1920);
    }

    // ── SplitV 等高分割 ───────────────────────────────────────

    #[test]
    fn splitv_two_equal_windows() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitV, 2);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);

        assert_eq!(g1.y, 0);
        assert_eq!(g1.height, 540);
        assert_eq!(g2.y, 540);
        assert_eq!(g2.height, 540);
        assert_eq!(g1.width, 1920);
        assert_eq!(g2.width, 1920);
    }

    #[test]
    fn splitv_three_equal_windows_cover_full_height() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitV, 3);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);
        let win3 = add_window(&mut tree, container, 3);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);
        let g3 = get_window_geometry(&tree, win3);

        let total = g1.height + g2.height + g3.height;
        assert_eq!(total, 1080);
    }

    // ── Tabbed：只有聚焦子节点可见 ────────────────────────────

    #[test]
    fn tabbed_focused_child_gets_full_area() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = tree.add_node(
            ws,
            NodeData::Container {
                layout: Layout::Tabbed,
                sizes: vec![1.0, 1.0],
                focused_child: 1, // 聚焦第二个
            },
        );
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g2 = get_window_geometry(&tree, win2);
        assert_eq!(g2, screen());

        // win1 因为未被聚焦，geometry 保持初始值 (0,0,0,0)
        let g1 = get_window_geometry(&tree, win1);
        assert_eq!(g1, Rect::new(0, 0, 0, 0));
    }

    // ── Stacked：与 Tabbed 行为相同 ──────────────────────────

    #[test]
    fn stacked_focused_child_gets_full_area() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = tree.add_node(
            ws,
            NodeData::Container {
                layout: Layout::Stacked,
                sizes: vec![1.0, 1.0],
                focused_child: 0,
            },
        );
        let win1 = add_window(&mut tree, container, 1);
        let _win2 = add_window(&mut tree, container, 2);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g1 = get_window_geometry(&tree, win1);
        assert_eq!(g1, screen());
    }

    // ── 嵌套布局 ─────────────────────────────────────────────

    #[test]
    fn nested_splith_inside_splitv() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);

        // 外层垂直分割为上下各 540px
        let outer = add_container(&mut tree, ws, Layout::SplitV, 2);

        // 上半部分：水平分割为左右各 960px
        let top_split = tree.add_node(
            outer,
            NodeData::Container {
                layout: Layout::SplitH,
                sizes: vec![1.0, 1.0],
                focused_child: 0,
            },
        );
        let win_top_left = add_window(&mut tree, top_split, 1);
        let win_top_right = add_window(&mut tree, top_split, 2);

        // 下半部分：单窗口
        let win_bottom = add_window(&mut tree, outer, 3);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let g_tl = get_window_geometry(&tree, win_top_left);
        let g_tr = get_window_geometry(&tree, win_top_right);
        let g_b = get_window_geometry(&tree, win_bottom);

        // 上半区域 y=0, height=540
        assert_eq!(g_tl.y, 0);
        assert_eq!(g_tl.height, 540);
        assert_eq!(g_tr.y, 0);
        assert_eq!(g_tr.height, 540);

        // 左右各半
        assert_eq!(g_tl.width, 960);
        assert_eq!(g_tr.width, 960);
        assert_eq!(g_tl.x, 0);
        assert_eq!(g_tr.x, 960);

        // 下半区域 y=540, height=540
        assert_eq!(g_b.y, 540);
        assert_eq!(g_b.height, 540);
        assert_eq!(g_b.width, 1920);
    }

    // ── get_window_geometries ─────────────────────────────────

    #[test]
    fn get_window_geometries_returns_all_windows() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitH, 2);
        add_window(&mut tree, container, 10);
        add_window(&mut tree, container, 20);

        let root = tree.root();
        compute_layout(&mut tree, root, screen());

        let geoms = get_window_geometries(&tree);
        assert_eq!(geoms.len(), 2);

        let ids: Vec<u64> = geoms.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&10));
        assert!(ids.contains(&20));
    }

    #[test]
    fn get_window_geometries_empty_tree() {
        let tree = Tree::new();
        let geoms = get_window_geometries(&tree);
        assert!(geoms.is_empty());
    }

    // ── 空容器不 panic ────────────────────────────────────────

    #[test]
    fn empty_container_does_not_panic() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let _container = tree.add_node(
            ws,
            NodeData::Container {
                layout: Layout::SplitH,
                sizes: Vec::new(),
                focused_child: 0,
            },
        );
        // 不应 panic
        let root = tree.root();
        compute_layout(&mut tree, root, screen());
    }

    // ── 非正方向 available 宽高 ───────────────────────────────

    #[test]
    fn layout_with_minimum_size() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitH, 2);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let tiny = Rect::new(0, 0, 2, 1);
        compute_layout(&mut tree, ws, tiny);

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);
        // 每个子节点最小宽度为 1
        assert!(g1.width >= 1);
        assert!(g2.width >= 1);
    }

    // ── Output 使用自身 geometry ──────────────────────────────

    #[test]
    fn output_uses_own_geometry_not_available() {
        let mut tree = Tree::new();
        let root = tree.root();
        let output = tree.add_node(
            root,
            NodeData::Output {
                name: "eDP-1".into(),
                geometry: Rect::new(100, 200, 800, 600),
            },
        );
        let ws = add_workspace(&mut tree, output);
        let win = add_window(&mut tree, ws, 1);

        // 传入 available 与 output.geometry 不同
        compute_layout(&mut tree, root, Rect::new(0, 0, 9999, 9999));

        let g = get_window_geometry(&tree, win);
        assert_eq!(g, Rect::new(100, 200, 800, 600));
    }
}
