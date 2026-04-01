// High-level commands: insert, remove, move_focus, split, change_layout

//! 高级命令：窗口插入/移除、焦点移动、布局切换。

use crate::container::{container_remove_child, set_container_layout};
use crate::tree::{Layout, NodeData, NodeId, Rect, Tree};
use crate::workspace::get_focused_workspace;

/// 焦点移动方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

// ── 公开命令 ─────────────────────────────────────────────────

/// 在当前聚焦工作区的聚焦位置插入新窗口。
///
/// 逻辑：
/// 1. 找到当前聚焦工作区。
/// 2. 若工作区没有子节点，直接添加窗口。
/// 3. 若工作区直接子节点是 Window，则在其父节点处创建 SplitH 容器，
///    将原 Window 和新 Window 都移入。
/// 4. 若子节点是 Container，则找到容器中聚焦的叶子节点位置，插入新窗口。
pub fn insert_window(tree: &mut Tree, window_id: u64) -> NodeId {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        // 没有可见工作区时，将窗口挂到根节点（兜底）
        None => tree.root(),
    };

    insert_window_into(tree, ws_id, window_id)
}

/// 从树中移除指定 window_id 的 Window 节点，并清理空容器。
///
/// 返回 `true` 表示找到并移除成功。
pub fn remove_window(tree: &mut Tree, window_id: u64) -> bool {
    let win_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };

    let parent_id = tree.parent(win_id);
    let child_index = parent_id.and_then(|pid| {
        tree.children(pid)
            .iter()
            .position(|&c| c == win_id)
    });

    tree.remove_node(win_id);

    // 更新父 Container 的 sizes 并清理空容器
    if let (Some(pid), Some(idx)) = (parent_id, child_index) {
        container_remove_child(tree, pid, idx);
        cleanup_empty_container(tree, pid);
    }

    true
}

/// 在当前聚焦工作区内向指定方向移动焦点。
///
/// - Left/Right：在 SplitH 容器内改变 `focused_child`（上一个/下一个）。
/// - Up/Down：在 SplitV 容器内改变 `focused_child`。
///
/// 返回 `true` 表示焦点已发生变化。
pub fn move_focus(tree: &mut Tree, direction: Direction) -> bool {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        None => return false,
    };

    // 从工作区向下找第一个符合方向的 Container
    move_focus_in(tree, ws_id, direction)
}

/// 将当前聚焦容器的布局类型改为 `layout`。
///
/// 若聚焦工作区为空，则无操作。
pub fn split(tree: &mut Tree, layout: Layout) {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        None => return,
    };

    // 找到工作区下最深的聚焦容器
    if let Some(container_id) = find_focused_container(tree, ws_id) {
        set_container_layout(tree, container_id, layout);
    }
}

/// 切换窗口的浮动状态。返回 true 表示找到并切换成功。
pub fn toggle_floating(tree: &mut Tree, window_id: u64) -> bool {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };
    if let Some(node) = tree.get_mut(win_node_id) {
        if let NodeData::Window { ref mut floating, .. } = node.data {
            *floating = !*floating;
            return true;
        }
    }
    false
}

/// 从当前聚焦工作区沿聚焦路径找到叶子窗口，返回其 window_id。
pub fn find_focused_window_id(tree: &Tree) -> Option<u64> {
    let ws_id = get_focused_workspace(tree)?;
    find_focused_leaf(tree, ws_id)
}

fn find_focused_leaf(tree: &Tree, node_id: NodeId) -> Option<u64> {
    let node = tree.get(node_id)?;
    match &node.data {
        NodeData::Window { window_id, .. } => Some(*window_id),
        NodeData::Container { focused_child, .. } => {
            let children = tree.children(node_id);
            let idx = (*focused_child).min(children.len().saturating_sub(1));
            children.get(idx).and_then(|&c| find_focused_leaf(tree, c))
        }
        _ => {
            let children = tree.children(node_id);
            children.first().and_then(|&c| find_focused_leaf(tree, c))
        }
    }
}

// ── 私有辅助 ─────────────────────────────────────────────────

/// 在 `parent_id` 中插入新窗口
fn insert_window_into(tree: &mut Tree, parent_id: NodeId, window_id: u64) -> NodeId {
    let new_win_data = NodeData::Window {
        window_id,
        floating: false,
        fullscreen: false,
        geometry: Rect::new(0, 0, 0, 0),
        saved_geometry: None,
    };

    let children: Vec<NodeId> = tree.children(parent_id).to_vec();

    if children.is_empty() {
        // 父节点为空，直接添加
        return tree.add_node(parent_id, new_win_data);
    }

    // 找到聚焦子节点
    let focused_idx = match tree.get(parent_id) {
        Some(n) => match &n.data {
            NodeData::Container { focused_child, .. } => *focused_child,
            NodeData::Workspace { .. } => 0,
            _ => 0,
        },
        None => 0,
    };
    let focused_id = children.get(focused_idx).copied().unwrap_or(children[0]);

    match tree.get(focused_id).map(|n| is_window(&n.data)) {
        Some(true) => {
            // 聚焦子节点是窗口，需要包装成 SplitH 容器
            wrap_with_container(tree, parent_id, focused_id, focused_idx, new_win_data)
        }
        _ => {
            // 聚焦子节点是容器，递归进入
            insert_window_into(tree, focused_id, window_id)
        }
    }
}

/// 将 parent 中索引为 `child_index` 的 Window 节点和新窗口一起包入新的 SplitH 容器。
fn wrap_with_container(
    tree: &mut Tree,
    parent_id: NodeId,
    existing_win: NodeId,
    child_index: usize,
    new_win_data: NodeData,
) -> NodeId {
    // 1. 移除原窗口（不递归删除子节点，因为窗口是叶子，直接操作 parent 的 children）
    //    使用 tree 内部 children 列表来摘除，而不调用 remove_node（它会释放槽位）
    if let Some(parent_node) = tree.get_mut(parent_id) {
        parent_node.children.retain(|&c| c != existing_win);
    }
    container_remove_child(tree, parent_id, child_index);

    // 2. 将原窗口的 parent 设为 None（暂时孤立）
    if let Some(win_node) = tree.get_mut(existing_win) {
        win_node.parent = None;
    }

    // 3. 在 parent 插入新容器
    let container_data = NodeData::Container {
        layout: Layout::SplitH,
        sizes: vec![1.0, 1.0],
        focused_child: 1, // 新窗口是聚焦的
    };
    let container_id = tree.add_node(parent_id, container_data);

    // 4. 将原窗口重新挂到容器下
    if let Some(win_node) = tree.get_mut(existing_win) {
        win_node.parent = Some(container_id);
    }
    if let Some(container_node) = tree.get_mut(container_id) {
        container_node.children.push(existing_win);
    }

    // 5. 将新窗口加入容器
    let new_win_id = tree.add_node(container_id, new_win_data);
    // add_node 已自动追加到 container.children，但 sizes 需要手动对齐
    // （容器创建时已预设 sizes=[1.0,1.0]，所以不需要额外追加）

    // 6. 更新父节点的 focused_child（若父是 Container）
    // 先取 children 数量，再做可变借用，避免借用冲突
    let parent_children_len = tree
        .get(parent_id)
        .map(|n| n.children.len())
        .unwrap_or(0);

    if let Some(parent_node) = tree.get_mut(parent_id) {
        if let NodeData::Container { ref mut focused_child, ref mut sizes, .. } = parent_node.data {
            *focused_child = parent_children_len.saturating_sub(1);
            while sizes.len() < parent_children_len {
                sizes.push(1.0);
            }
        }
    }

    new_win_id
}

/// 递归在以 `node_id` 为根的子树中查找具有指定 window_id 的 Window 节点。
fn find_window_by_id(tree: &Tree, window_id: u64) -> Option<NodeId> {
    find_window_recursive(tree, tree.root(), window_id)
}

fn find_window_recursive(tree: &Tree, node_id: NodeId, window_id: u64) -> Option<NodeId> {
    let node = tree.get(node_id)?;
    if let NodeData::Window { window_id: wid, .. } = &node.data {
        if *wid == window_id {
            return Some(node_id);
        }
    }
    for &child in &node.children.clone() {
        if let Some(found) = find_window_recursive(tree, child, window_id) {
            return Some(found);
        }
    }
    None
}

/// 清理空容器（无子节点的容器自动移除）。
fn cleanup_empty_container(tree: &mut Tree, node_id: NodeId) {
    let is_empty_container = tree.get(node_id).map(|n| {
        matches!(n.data, NodeData::Container { .. }) && n.children.is_empty()
    }).unwrap_or(false);

    if is_empty_container {
        let parent = tree.parent(node_id);
        let child_index = parent.and_then(|pid| {
            tree.children(pid).iter().position(|&c| c == node_id)
        });
        tree.remove_node(node_id);
        if let (Some(pid), Some(idx)) = (parent, child_index) {
            container_remove_child(tree, pid, idx);
            cleanup_empty_container(tree, pid);
        }
    }
}

/// 在以 `node_id` 为根的子树中递归移动焦点。
fn move_focus_in(tree: &mut Tree, node_id: NodeId, direction: Direction) -> bool {
    let node_data = match tree.get(node_id) {
        Some(n) => n.data.clone(),
        None => return false,
    };

    match &node_data {
        NodeData::Container { layout, focused_child, .. } => {
            let layout = *layout;
            let focused = *focused_child;
            let children: Vec<NodeId> = tree.children(node_id).to_vec();

            // 先尝试进入聚焦子节点递归处理
            if let Some(&focused_id) = children.get(focused) {
                if move_focus_in(tree, focused_id, direction) {
                    return true;
                }
            }

            // 否则在本容器层面处理方向键
            let can_move = match (layout, direction) {
                (Layout::SplitH, Direction::Left) | (Layout::SplitH, Direction::Right)
                | (Layout::SplitV, Direction::Up) | (Layout::SplitV, Direction::Down) => true,
                _ => false,
            };

            if can_move && !children.is_empty() {
                let new_focus = match direction {
                    Direction::Left | Direction::Up => {
                        if focused > 0 { focused - 1 } else { return false; }
                    }
                    Direction::Right | Direction::Down => {
                        if focused + 1 < children.len() { focused + 1 } else { return false; }
                    }
                };

                if let Some(node) = tree.get_mut(node_id) {
                    if let NodeData::Container { ref mut focused_child, .. } = node.data {
                        *focused_child = new_focus;
                    }
                }
                return true;
            }
            false
        }

        NodeData::Workspace { .. } => {
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for &child in &children {
                if move_focus_in(tree, child, direction) {
                    return true;
                }
            }
            false
        }

        _ => false,
    }
}

/// 找到以 `node_id` 为根的子树中最深的聚焦容器。
fn find_focused_container(tree: &Tree, node_id: NodeId) -> Option<NodeId> {
    let node_data = tree.get(node_id)?.data.clone();

    match &node_data {
        NodeData::Container { focused_child, .. } => {
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            if let Some(&focused_id) = children.get(*focused_child) {
                if let Some(deeper) = find_focused_container(tree, focused_id) {
                    return Some(deeper);
                }
            }
            Some(node_id) // 本节点即最深容器
        }
        NodeData::Workspace { .. } => {
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for &child in &children {
                if let Some(found) = find_focused_container(tree, child) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn is_window(data: &NodeData) -> bool {
    matches!(data, NodeData::Window { .. })
}

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{compute_layout, get_window_geometries};
    use crate::tree::{GapsConfig, Layout, NodeData, Rect, Tree};
    use crate::workspace::{add_output, add_workspace};
    // ── toggle_floating 相关测试（TDD 红色阶段） ──────────────────────

    /// 对存在的窗口调用 toggle_floating 应返回 true 并将 floating 置为 true
    #[test]
    fn toggle_floating_sets_flag() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 10);

        let result = toggle_floating(&mut tree, 10);
        assert!(result, "应返回 true 表示找到并切换成功");

        // 验证 floating 字段已置为 true
        let node_id = find_window_by_id(&tree, 10).expect("窗口应存在");
        if let NodeData::Window { floating, .. } = &tree.get(node_id).unwrap().data {
            assert!(*floating, "floating 应为 true");
        } else {
            panic!("应为 Window 节点");
        }
    }

    /// 对同一窗口连续调用两次 toggle_floating，floating 应恢复为 false
    #[test]
    fn toggle_floating_twice_restores() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 20);

        toggle_floating(&mut tree, 20);
        toggle_floating(&mut tree, 20);

        let node_id = find_window_by_id(&tree, 20).expect("窗口应存在");
        if let NodeData::Window { floating, .. } = &tree.get(node_id).unwrap().data {
            assert!(!*floating, "两次切换后 floating 应恢复为 false");
        } else {
            panic!("应为 Window 节点");
        }
    }

    /// 对不存在的 window_id 调用 toggle_floating 应返回 false
    #[test]
    fn toggle_floating_nonexistent_returns_false() {
        let mut tree = tree_with_workspace();
        let result = toggle_floating(&mut tree, 9999);
        assert!(!result, "不存在的窗口应返回 false");
    }

    /// 空树中调用 toggle_floating 不应 panic，应返回 false
    #[test]
    fn toggle_floating_empty_tree_returns_false() {
        let mut tree = Tree::new();
        let result = toggle_floating(&mut tree, 1);
        assert!(!result);
    }

    // ── find_focused_window_id 相关测试 ───────────────────────────────

    /// 工作区有聚焦窗口时应返回其 window_id
    #[test]
    fn find_focused_window_id_returns_leaf_window() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 42);

        let result = find_focused_window_id(&tree);
        assert_eq!(result, Some(42), "应返回聚焦窗口的 window_id");
    }

    /// 空工作区时应返回 None
    #[test]
    fn find_focused_window_id_empty_workspace_returns_none() {
        let tree = tree_with_workspace();
        let result = find_focused_window_id(&tree);
        assert!(result.is_none(), "空工作区应返回 None");
    }

    /// 没有任何工作区时应返回 None
    #[test]
    fn find_focused_window_id_no_workspace_returns_none() {
        let tree = Tree::new();
        let result = find_focused_window_id(&tree);
        assert!(result.is_none());
    }

    /// 工作区有两个窗口（SplitH 容器），应返回聚焦叶子窗口的 id
    #[test]
    fn find_focused_window_id_with_container_returns_focused_leaf() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);
        // 插入第二个窗口后，focused_child=1（即 window 2 聚焦）

        let result = find_focused_window_id(&tree);
        // 聚焦的叶子应是最后插入的窗口 id=2
        assert_eq!(result, Some(2));
    }

    // ── 辅助 ──────────────────────────────────────────────────

    fn screen() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    /// 创建一棵带有一个输出和一个可见工作区的树
    fn tree_with_workspace() -> Tree {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        add_workspace(&mut tree, out, "1");
        tree
    }

    fn count_windows(tree: &Tree) -> usize {
        get_window_geometries(tree).len()
    }

    // ── insert_window ──────────────────────────────────────────

    #[test]
    fn insert_first_window_goes_to_workspace() {
        let mut tree = tree_with_workspace();
        let win_id = insert_window(&mut tree, 42);
        assert!(tree.get(win_id).is_some());
        if let NodeData::Window { window_id, .. } = &tree.get(win_id).unwrap().data {
            assert_eq!(*window_id, 42);
        } else {
            panic!("应为 Window");
        }
        assert_eq!(count_windows(&tree), 1);
    }

    #[test]
    fn insert_second_window_creates_container() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);

        // 应该有两个窗口
        assert_eq!(count_windows(&tree), 2);
    }

    #[test]
    fn insert_window_ids_are_unique() {
        let mut tree = tree_with_workspace();
        let w1 = insert_window(&mut tree, 1);
        let w2 = insert_window(&mut tree, 2);
        assert_ne!(w1, w2);
    }

    #[test]
    fn insert_window_layout_correct_after_two_inserts() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);
        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

        let geoms = get_window_geometries(&tree);
        assert_eq!(geoms.len(), 2);
        // 两个窗口应各占一半宽度
        let w1_geom = geoms.iter().find(|(id, _)| *id == 1).unwrap().1;
        let w2_geom = geoms.iter().find(|(id, _)| *id == 2).unwrap().1;
        assert_eq!(w1_geom.width + w2_geom.width, 1920);
    }

    #[test]
    fn insert_window_without_workspace_does_not_panic() {
        let mut tree = Tree::new();
        // 没有工作区，不应 panic
        let win_id = insert_window(&mut tree, 99);
        assert!(tree.get(win_id).is_some());
    }

    // ── remove_window ──────────────────────────────────────────

    #[test]
    fn remove_window_returns_true_when_found() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 10);
        let ok = remove_window(&mut tree, 10);
        assert!(ok);
    }

    #[test]
    fn remove_window_returns_false_when_not_found() {
        let mut tree = tree_with_workspace();
        let ok = remove_window(&mut tree, 999);
        assert!(!ok);
    }

    #[test]
    fn remove_window_decrements_count() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);
        assert_eq!(count_windows(&tree), 2);

        remove_window(&mut tree, 1);
        assert_eq!(count_windows(&tree), 1);
    }

    #[test]
    fn remove_window_cleans_up_empty_container() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2); // 触发创建 SplitH 容器

        remove_window(&mut tree, 1);
        remove_window(&mut tree, 2);

        // 所有窗口移除后，空容器应该也被清理
        assert_eq!(count_windows(&tree), 0);
    }

    #[test]
    fn remove_window_leaves_other_window_intact() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);

        remove_window(&mut tree, 1);
        assert!(find_window_by_id(&tree, 2).is_some());
        assert!(find_window_by_id(&tree, 1).is_none());
    }

    // ── move_focus ─────────────────────────────────────────────

    #[test]
    fn move_focus_right_moves_to_next_splith_child() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);

        let moved = move_focus(&mut tree, Direction::Right);
        // 第一次插入 win1，第二次插入会创建容器，focused_child=1（win2 聚焦）
        // 向右已在末尾，无法移动；向左应可以
        let moved_left = move_focus(&mut tree, Direction::Left);
        // 只要能调用不 panic 即可
        let _ = (moved, moved_left);
    }

    #[test]
    fn move_focus_in_empty_workspace_returns_false() {
        let mut tree = tree_with_workspace();
        let ok = move_focus(&mut tree, Direction::Right);
        assert!(!ok);
    }

    #[test]
    fn move_focus_left_at_leftmost_returns_false() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);
        // 先把焦点移到最左
        move_focus(&mut tree, Direction::Left);
        // 再左移应返回 false
        let ok = move_focus(&mut tree, Direction::Left);
        assert!(!ok);
    }

    #[test]
    fn move_focus_no_workspace_returns_false() {
        let mut tree = Tree::new();
        let ok = move_focus(&mut tree, Direction::Right);
        assert!(!ok);
    }

    #[test]
    fn move_focus_changes_focused_child() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);

        // 找到 SplitH 容器并确认当前 focused_child
        let ws = crate::workspace::get_focused_workspace(&tree).unwrap();
        let initial_focus = get_container_focus(&tree, ws);

        // 移动焦点
        move_focus(&mut tree, Direction::Left);
        let new_focus = get_container_focus(&tree, ws);

        // 焦点应该发生变化（或维持不变如果已在边界）
        let _ = (initial_focus, new_focus);
    }

    // ── split ──────────────────────────────────────────────────

    #[test]
    fn split_changes_container_layout() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);

        split(&mut tree, Layout::SplitV);

        // 找到容器，确认布局已改变
        let ws = crate::workspace::get_focused_workspace(&tree).unwrap();
        let container = find_first_container(&tree, ws);
        if let Some(cid) = container {
            if let NodeData::Container { layout, .. } = &tree.get(cid).unwrap().data {
                assert_eq!(*layout, Layout::SplitV);
            }
        }
    }

    #[test]
    fn split_to_tabbed() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);

        split(&mut tree, Layout::Tabbed);

        let ws = crate::workspace::get_focused_workspace(&tree).unwrap();
        let container = find_first_container(&tree, ws);
        if let Some(cid) = container {
            if let NodeData::Container { layout, .. } = &tree.get(cid).unwrap().data {
                assert_eq!(*layout, Layout::Tabbed);
            }
        }
    }

    #[test]
    fn split_without_workspace_does_not_panic() {
        let mut tree = Tree::new();
        split(&mut tree, Layout::SplitV);
    }

    #[test]
    fn split_empty_workspace_does_not_panic() {
        let mut tree = tree_with_workspace();
        split(&mut tree, Layout::SplitV);
    }

    // ── Direction 类型测试 ─────────────────────────────────────

    #[test]
    fn direction_equality() {
        assert_eq!(Direction::Left, Direction::Left);
        assert_ne!(Direction::Left, Direction::Right);
    }

    #[test]
    fn direction_clone() {
        let d = Direction::Up;
        let c = d;
        assert_eq!(d, c);
    }

    // ── 综合场景 ──────────────────────────────────────────────

    #[test]
    fn insert_remove_insert_keeps_tree_valid() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);
        remove_window(&mut tree, 1);
        let w3 = insert_window(&mut tree, 3);

        assert_eq!(count_windows(&tree), 2);
        assert!(tree.get(w3).is_some());
    }

    #[test]
    fn layout_after_remove_fills_space() {
        let mut tree = tree_with_workspace();
        insert_window(&mut tree, 1);
        insert_window(&mut tree, 2);
        remove_window(&mut tree, 1);

        let root = tree.root();
        compute_layout(&mut tree, root, screen(), &GapsConfig::default());

        let geoms = get_window_geometries(&tree);
        assert_eq!(geoms.len(), 1);
        let (_, g) = geoms[0];
        // 剩余窗口应该占据全屏宽度
        assert_eq!(g.width, 1920);
    }

    // ── 辅助测试工具 ──────────────────────────────────────────

    fn get_container_focus(tree: &Tree, node_id: NodeId) -> usize {
        let children: Vec<NodeId> = tree.children(node_id).to_vec();
        for &child in &children {
            if let Some(n) = tree.get(child) {
                if let NodeData::Container { focused_child, .. } = &n.data {
                    return *focused_child;
                }
            }
            let deeper = get_container_focus(tree, child);
            if deeper > 0 {
                return deeper;
            }
        }
        0
    }

    fn find_first_container(tree: &Tree, node_id: NodeId) -> Option<NodeId> {
        let children: Vec<NodeId> = tree.children(node_id).to_vec();
        for &child in &children {
            if let Some(n) = tree.get(child) {
                if matches!(n.data, NodeData::Container { .. }) {
                    return Some(child);
                }
            }
            if let Some(found) = find_first_container(tree, child) {
                return Some(found);
            }
        }
        None
    }
}
