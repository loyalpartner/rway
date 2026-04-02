// Layout algorithms: SplitH, SplitV, Tabbed, Stacked

//! 布局算法：自顶向下递归计算树中所有 Window 节点的几何位置。

use crate::tree::{GapsConfig, Layout, NodeData, NodeId, Rect, Tree};

/// 自顶向下递归计算从 `node_id` 开始的子树布局。
///
/// - Output/Workspace 节点：将 `available` 传递给唯一或第一个子节点。
/// - Container/SplitH：水平瓜分 `available.width`，各子节点按 `sizes` 比例分配。
/// - Container/SplitV：垂直瓜分 `available.height`，各子节点按 `sizes` 比例分配。
/// - Container/Tabbed|Stacked：只有 `focused_child` 对应的子节点得到完整区域。
/// - Window：将 `available` 四边各收缩 `gaps.inner / 2` 后设为该节点的 geometry。
///
/// gaps 语义：
/// - `gaps.outer`：在 Workspace 节点处将可用区域四边各收缩 `outer`。
/// - `gaps.inner`：在 Window 节点处将可用区域四边各收缩 `inner / 2`（相邻两窗口合计 `inner`）。
pub fn compute_layout(tree: &mut Tree, node_id: NodeId, available: Rect, gaps: &GapsConfig) {
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
                compute_layout(tree, child, available, gaps);
            }
        }

        NodeData::Output { geometry, .. } => {
            // 输出节点使用自身的 geometry 作为可用区域
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for child in children {
                compute_layout(tree, child, geometry, gaps);
            }
        }

        NodeData::Workspace { .. } => {
            // 工作区：应用外边距后将收缩后的区域传递给子节点
            let inner_rect = shrink_rect(available, gaps.outer);
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for child in children {
                compute_layout(tree, child, inner_rect, gaps);
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
                    layout_split_h(tree, &children, &sizes, available, gaps);
                }
                Layout::SplitV => {
                    layout_split_v(tree, &children, &sizes, available, gaps);
                }
                Layout::Tabbed | Layout::Stacked => {
                    // 仅聚焦子节点可见，获得完整区域
                    let focus_idx = focused_child.min(children.len().saturating_sub(1));
                    let focused_id = children[focus_idx];
                    compute_layout(tree, focused_id, available, gaps);
                }
            }
        }

        NodeData::Window { .. } => {
            // 叶节点：浮动窗口跳过布局计算，保留现有几何
            let is_floating = matches!(&node_data, NodeData::Window { floating: true, .. });
            if !is_floating {
                let final_rect = shrink_rect(available, gaps.inner / 2);
                if let Some(node) = tree.get_mut(node_id) {
                    if let NodeData::Window {
                        ref mut geometry, ..
                    } = node.data
                    {
                        *geometry = final_rect;
                    }
                }
            }
        }
    }
}

/// 将矩形四边各向内收缩 `amount` 像素，返回新矩形。
///
/// 若收缩后宽度或高度小于 1，则保持最小值 1（防止窗口尺寸变为负数）。
fn shrink_rect(r: Rect, amount: i32) -> Rect {
    if amount == 0 {
        return r;
    }
    Rect {
        x: r.x + amount,
        y: r.y + amount,
        width: (r.width - 2 * amount).max(1),
        height: (r.height - 2 * amount).max(1),
    }
}

/// 水平分割：沿 X 轴按 sizes 比例分配宽度。
///
/// 内边距（inner）由各叶子 Window 节点在 `compute_layout` 中统一收缩，
/// 此函数仅负责按比例切分空间，无需感知 gaps。
fn layout_split_h(
    tree: &mut Tree,
    children: &[NodeId],
    sizes: &[f64],
    available: Rect,
    gaps: &GapsConfig,
) {
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

        compute_layout(tree, child, child_rect, gaps);
        x_offset += child_width.max(1);
    }
}

/// 垂直分割：沿 Y 轴按 sizes 比例分配高度。
///
/// 内边距（inner）由各叶子 Window 节点在 `compute_layout` 中统一收缩，
/// 此函数仅负责按比例切分空间，无需感知 gaps。
fn layout_split_v(
    tree: &mut Tree,
    children: &[NodeId],
    sizes: &[f64],
    available: Rect,
    gaps: &GapsConfig,
) {
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

        compute_layout(tree, child, child_rect, gaps);
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
    use crate::tree::{GapsConfig, Layout, NodeData, NodeId, Rect, Tree};

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
                fullscreen_global: false,
                sticky: false,
                marks: Vec::new(),
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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

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
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());
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
        compute_layout(&mut tree, ws, tiny, &GapsConfig::default());

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
        compute_layout(
            &mut tree,
            root,
            Rect::new(0, 0, 9999, 9999),
            &GapsConfig::default(),
        );

        let g = get_window_geometry(&tree, win);
        assert_eq!(g, Rect::new(100, 200, 800, 600));
    }

    // ── Gaps 测试（RED 阶段：先写测试，再实现）─────────────────

    /// 外边距为 10 时，工作区子节点可用区域四边各缩小 10px
    #[test]
    fn gaps_outer_shrinks_workspace_area() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let win = add_window(&mut tree, ws, 1);

        let gaps = GapsConfig {
            inner: 0,
            outer: 10,
        };
        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &gaps);

        let g = get_window_geometry(&tree, win);
        // 外边距 10：x 和 y 各 +10，宽高各 -20
        assert_eq!(g.x, 10);
        assert_eq!(g.y, 10);
        assert_eq!(g.width, 1920 - 20);
        assert_eq!(g.height, 1080 - 20);
    }

    /// 内边距为 10 时，水平分割的两个窗口之间有 10px 间距
    #[test]
    fn gaps_inner_between_two_splith_windows() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitH, 2);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let gaps = GapsConfig {
            inner: 10,
            outer: 0,
        };
        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &gaps);

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);

        // 每个窗口四边各缩 inner/2 = 5px：
        // - 相邻边：win1 右缩 5 + win2 左缩 5 = 10px 间距
        // - 外侧边：win1 左缩 5，win2 右缩 5（各有 inner/2 的外侧空白）
        let inner = 10_i32;
        let gap_between = g2.x - (g1.x + g1.width);
        assert_eq!(
            gap_between, inner,
            "两窗口之间应有 {}px 间距，实际为 {}",
            inner, gap_between
        );

        // 外侧空白各为 inner/2，两窗口宽度 + 内部间距 + 两侧外部空白 = 总宽度
        assert_eq!(g1.width + g2.width + gap_between + inner, 1920);
        // 验证外侧位置：win1 从 inner/2 开始，win2 在 总宽 - inner/2 结束
        assert_eq!(g1.x, inner / 2, "win1.x 应为 {}", inner / 2);
        assert_eq!(
            g2.x + g2.width,
            1920 - inner / 2,
            "win2 右边界应为 {}",
            1920 - inner / 2
        );
    }

    /// 内边距为 8 时，垂直分割的三个窗口之间各有 8px 间距
    #[test]
    fn gaps_inner_between_three_splitv_windows() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitV, 3);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);
        let win3 = add_window(&mut tree, container, 3);

        let gaps = GapsConfig { inner: 8, outer: 0 };
        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &gaps);

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);
        let g3 = get_window_geometry(&tree, win3);

        // win1 与 win2 之间间距 = 8
        let gap12 = g2.y - (g1.y + g1.height);
        assert_eq!(gap12, 8, "win1-win2 间距应为 8，实际为 {}", gap12);

        // win2 与 win3 之间间距 = 8
        let gap23 = g3.y - (g2.y + g2.height);
        assert_eq!(gap23, 8, "win2-win3 间距应为 8，实际为 {}", gap23);

        // 每个窗口四边各缩 inner/2 = 4px，外侧各有 4px 空白
        // 三窗口高度之和 + 两段内部间距 + 两侧外部空白（各 inner/2）= 总高度
        let inner = 8_i32;
        assert_eq!(
            g1.height + g2.height + g3.height + gap12 + gap23 + inner,
            1080,
            "三窗口高度 + 间距 + 外侧空白应等于总高度"
        );
        // 验证外侧：第一个窗口从 inner/2 开始，最后一个窗口在 总高 - inner/2 结束
        assert_eq!(g1.y, inner / 2, "win1.y 应为 {}", inner / 2);
        assert_eq!(
            g3.y + g3.height,
            1080 - inner / 2,
            "win3 底边应为 {}",
            1080 - inner / 2
        );
    }

    /// GapsConfig::default()（0,0）应产生与旧版完全相同的几何结果
    #[test]
    fn gaps_zero_is_backwards_compatible() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitH, 2);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);

        // 零间距时与原始行为完全一致
        assert_eq!(g1.x, 0);
        assert_eq!(g1.width, 960);
        assert_eq!(g2.x, 960);
        assert_eq!(g2.width, 960);
        assert_eq!(g1.height, 1080);
        assert_eq!(g2.height, 1080);
    }

    /// 外边距 10 + 内边距 5 同时生效
    #[test]
    fn gaps_outer_and_inner_combined() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let container = add_container(&mut tree, ws, Layout::SplitH, 2);
        let win1 = add_window(&mut tree, container, 1);
        let win2 = add_window(&mut tree, container, 2);

        let gaps = GapsConfig {
            inner: 10,
            outer: 20,
        };
        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &gaps);

        let g1 = get_window_geometry(&tree, win1);
        let g2 = get_window_geometry(&tree, win2);

        // 外边距 20 后可用宽度 = 1920 - 40 = 1880
        // 两个等宽窗口各占约 940px，但再减去内边距
        // win1: x = outer(20) + inner/2(5) = 25，不再有左内边距（最左侧只有外边距）
        // 实际：win1 从 outer + inner/2 开始，win2 到 (1920-outer - inner/2) 结束

        // win1.x = outer + inner/2 = 20 + 5 = 25
        assert_eq!(g1.x, 25, "win1.x 应为 25，实际为 {}", g1.x);

        // win2 右边 = 1920 - outer - inner/2 = 1920 - 20 - 5 = 1895
        let win2_right = g2.x + g2.width;
        assert_eq!(
            win2_right, 1895,
            "win2 右边界应为 1895，实际为 {}",
            win2_right
        );

        // 两窗口之间间距 = inner = 10
        let gap = g2.x - (g1.x + g1.width);
        assert_eq!(gap, 10, "两窗口间距应为 10，实际为 {}", gap);

        // win1 顶部 = outer + inner/2 = 25（高度方向也有 inner/2）
        assert_eq!(g1.y, 25, "win1.y 应为 25，实际为 {}", g1.y);
    }

    /// 工作区内只有单个窗口时：只应用外边距，无内边距
    #[test]
    fn gaps_single_window_only_outer() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);
        let win = add_window(&mut tree, ws, 1);

        let gaps = GapsConfig {
            inner: 20,
            outer: 15,
        };
        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &gaps);

        let g = get_window_geometry(&tree, win);

        // 单个窗口：外边距 15 压缩四边，内边距 20 也应用（每边 10px）
        // 实际上单窗口也会经过 Window 节点的 inner/2 收缩
        // x = outer + inner/2 = 15 + 10 = 25
        assert_eq!(g.x, 25, "单窗口 x 应为 25，实际为 {}", g.x);
        assert_eq!(g.y, 25, "单窗口 y 应为 25，实际为 {}", g.y);
        assert_eq!(g.width, 1920 - 2 * 25, "单窗口宽应为 {}", 1920 - 2 * 25);
        assert_eq!(g.height, 1080 - 2 * 25, "单窗口高应为 {}", 1080 - 2 * 25);
    }

    // ── 浮动窗口布局测试 ──────────────────────────────────────

    /// 浮动窗口在 compute_layout 中应被跳过，geometry 保持初始值不变
    #[test]
    fn floating_window_skipped_in_layout() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);

        // 添加一个平铺窗口和一个浮动窗口
        let tiled_win = add_window(&mut tree, ws, 1);
        // 手动创建浮动窗口节点（floating=true）
        let floating_win = tree.add_node(
            ws,
            NodeData::Window {
                window_id: 2,
                floating: true,
                fullscreen: false,
                fullscreen_global: false,
                sticky: false,
                marks: Vec::new(),
                geometry: Rect::new(100, 100, 400, 300), // 预设浮动位置
                saved_geometry: None,
            },
        );

        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

        // 平铺窗口应获得全屏（工作区有两个子节点，各自获得 available）
        // 浮动窗口保持原始 geometry 不变
        let floating_geom = get_window_geometry(&tree, floating_win);
        assert_eq!(
            floating_geom,
            Rect::new(100, 100, 400, 300),
            "浮动窗口 geometry 不应被 compute_layout 修改"
        );

        // 平铺窗口应被赋予布局计算的 geometry（不应保持初始 0,0,0,0）
        let tiled_geom = get_window_geometry(&tree, tiled_win);
        assert!(tiled_geom.width > 0, "平铺窗口宽度应大于 0");
        assert!(tiled_geom.height > 0, "平铺窗口高度应大于 0");
    }

    /// 浮动窗口不应影响平铺兄弟节点的空间分配
    ///
    /// 场景：SplitH 容器有 3 个窗口，中间一个浮动。
    /// 浮动窗口被跳过后，平铺窗口依旧按比例平分全部可用区域。
    #[test]
    fn floating_window_does_not_affect_tiled_siblings() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);

        // SplitH 容器，3 个子节点，sizes=[1.0, 1.0, 1.0]
        let container = add_container(&mut tree, ws, Layout::SplitH, 3);
        let win1 = add_window(&mut tree, container, 1);

        // 中间窗口设为浮动
        let win2 = tree.add_node(
            container,
            NodeData::Window {
                window_id: 2,
                floating: true,
                fullscreen: false,
                fullscreen_global: false,
                sticky: false,
                marks: Vec::new(),
                geometry: Rect::new(200, 200, 300, 200), // 预设浮动位置
                saved_geometry: None,
            },
        );

        let win3 = add_window(&mut tree, container, 3);

        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

        // win1 和 win3 各获得 1/3 宽度（因为容器 sizes=[1.0,1.0,1.0] 平均分配）
        let g1 = get_window_geometry(&tree, win1);
        let g3 = get_window_geometry(&tree, win3);
        // 两者都应有非零宽度
        assert!(g1.width > 0, "win1 宽度应大于 0");
        assert!(g3.width > 0, "win3 宽度应大于 0");

        // 浮动窗口保持预设 geometry
        let g2 = get_window_geometry(&tree, win2);
        assert_eq!(
            g2,
            Rect::new(200, 200, 300, 200),
            "浮动窗口 geometry 不应被修改"
        );
    }

    /// toggle_floating 后再次 compute_layout，浮动窗口重新参与布局
    #[test]
    fn toggled_to_tiled_participates_in_layout() {
        let mut tree = Tree::new();
        let output = add_output(&mut tree);
        let ws = add_workspace(&mut tree, output);

        // 创建一个初始为浮动的窗口
        let win = tree.add_node(
            ws,
            NodeData::Window {
                window_id: 99,
                floating: true,
                fullscreen: false,
                fullscreen_global: false,
                sticky: false,
                marks: Vec::new(),
                geometry: Rect::new(50, 50, 100, 100), // 浮动时的 geometry
                saved_geometry: None,
            },
        );

        // 浮动时 compute_layout 不修改 geometry
        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());
        let g_before = get_window_geometry(&tree, win);
        assert_eq!(
            g_before,
            Rect::new(50, 50, 100, 100),
            "浮动状态下 geometry 不变"
        );

        // 手动切换为平铺
        if let Some(node) = tree.get_mut(win) {
            if let NodeData::Window {
                ref mut floating, ..
            } = node.data
            {
                *floating = false;
            }
        }

        // 再次布局，窗口应获得全屏区域
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());
        let g_after = get_window_geometry(&tree, win);
        assert_eq!(g_after.width, 1920, "切换为平铺后应获得全屏宽度");
        assert_eq!(g_after.height, 1080, "切换为平铺后应获得全屏高度");
    }
}
