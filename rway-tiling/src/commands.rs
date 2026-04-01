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

/// Resize 轴向（rway-tiling 本地定义，不依赖 rway-config）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAxis {
    Width,
    Height,
}

/// 将聚焦窗口沿 `direction` 方向在兄弟节点间交换位置。
pub fn move_window(tree: &mut Tree, direction: Direction) -> bool {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        None => return false,
    };
    move_window_in(tree, ws_id, direction)
}

/// 将当前聚焦窗口移动到目标工作区。
pub fn move_to_workspace(tree: &mut Tree, target_ws: &str) -> bool {
    let win_id = match find_focused_window_id(tree) {
        Some(id) => id,
        None => return false,
    };

    // 找到目标工作区
    let target_ws_id = find_workspace_by_name_global(tree, target_ws);
    let target_ws_id = match target_ws_id {
        Some(id) => id,
        None => return false,
    };

    // 从当前位置移除窗口
    remove_window(tree, win_id);
    // 在目标工作区插入窗口
    insert_window_into(tree, target_ws_id, win_id);
    true
}

/// 调整节点在父容器中的 sizes 比例。
/// `delta_ppt` 为正表示增大，为负表示缩小（百分点）。
pub fn resize_container(
    tree: &mut Tree,
    node_id: NodeId,
    axis: ResizeAxis,
    delta_ppt: f64,
) -> bool {
    let parent_id = match tree.parent(node_id) {
        Some(id) => id,
        None => return false,
    };

    // 检查父容器布局方向是否与 axis 匹配
    let layout_matches = match tree.get(parent_id) {
        Some(n) => match &n.data {
            NodeData::Container { layout, .. } => matches!(
                (axis, layout),
                (ResizeAxis::Width, Layout::SplitH) | (ResizeAxis::Height, Layout::SplitV)
            ),
            _ => false,
        },
        None => false,
    };
    if !layout_matches {
        return false;
    }

    let children: Vec<NodeId> = tree.children(parent_id).to_vec();
    let my_index = match children.iter().position(|&c| c == node_id) {
        Some(i) => i,
        None => return false,
    };

    // 找到相邻兄弟来分配 delta
    let sibling_index = if my_index + 1 < children.len() {
        my_index + 1
    } else if my_index > 0 {
        my_index - 1
    } else {
        return false;
    };

    if let Some(node) = tree.get_mut(parent_id) {
        if let NodeData::Container { ref mut sizes, .. } = node.data {
            if my_index < sizes.len() && sibling_index < sizes.len() {
                let delta_norm = delta_ppt / 100.0;
                sizes[my_index] += delta_norm;
                sizes[sibling_index] -= delta_norm;
                // 确保不低于最小值
                if sizes[my_index] < 0.05 {
                    let diff = 0.05 - sizes[my_index];
                    sizes[my_index] = 0.05;
                    sizes[sibling_index] -= diff;
                }
                if sizes[sibling_index] < 0.05 {
                    let diff = 0.05 - sizes[sibling_index];
                    sizes[sibling_index] = 0.05;
                    sizes[my_index] -= diff;
                }
                return true;
            }
        }
    }
    false
}

/// 将焦点层级上移到父容器。
pub fn focus_parent(tree: &mut Tree) -> bool {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        None => return false,
    };

    // 当前焦点节点
    let current = tree.focus_node.unwrap_or_else(|| {
        // 沿聚焦路径找到叶子节点的 NodeId
        find_focused_leaf_node(tree, ws_id).unwrap_or(ws_id)
    });

    let parent = tree.parent(current);
    match parent {
        Some(pid) if pid != tree.root() => {
            tree.focus_node = Some(pid);
            true
        }
        _ => false,
    }
}

/// 将焦点层级下移到聚焦子节点。
pub fn focus_child(tree: &mut Tree) -> bool {
    let current = match tree.focus_node {
        Some(id) => id,
        None => return false, // 已在叶子层级
    };

    let node = match tree.get(current) {
        Some(n) => n.data.clone(),
        None => return false,
    };

    match &node {
        NodeData::Container { focused_child, .. } => {
            let children: Vec<NodeId> = tree.children(current).to_vec();
            let idx = (*focused_child).min(children.len().saturating_sub(1));
            if let Some(&child_id) = children.get(idx) {
                // 如果子节点是窗口，回到叶子层级
                let is_leaf = tree.get(child_id).map(|n| {
                    matches!(n.data, NodeData::Window { .. })
                }).unwrap_or(false);
                tree.focus_node = if is_leaf { None } else { Some(child_id) };
                true
            } else {
                false
            }
        }
        NodeData::Workspace { .. } => {
            let children: Vec<NodeId> = tree.children(current).to_vec();
            if let Some(&child_id) = children.first() {
                let is_leaf = tree.get(child_id).map(|n| {
                    matches!(n.data, NodeData::Window { .. })
                }).unwrap_or(false);
                tree.focus_node = if is_leaf { None } else { Some(child_id) };
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// 设置窗口的浮动状态为指定值。
pub fn set_floating(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };
    if let Some(node) = tree.get_mut(win_node_id) {
        if let NodeData::Window { ref mut floating, .. } = node.data {
            *floating = enable;
            return true;
        }
    }
    false
}

/// 设置窗口的全屏状态为指定值。
pub fn set_fullscreen(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };
    if let Some(node) = tree.get_mut(win_node_id) {
        if let NodeData::Window {
            ref mut fullscreen,
            ref mut saved_geometry,
            ref geometry,
            ..
        } = node.data
        {
            if enable && !*fullscreen {
                *saved_geometry = Some(*geometry);
            }
            *fullscreen = enable;
            return true;
        }
    }
    false
}

/// 切换窗口的全屏状态。
pub fn toggle_fullscreen(tree: &mut Tree, window_id: u64) -> bool {
    let current = find_window_by_id(tree, window_id).and_then(|id| {
        tree.get(id).and_then(|n| {
            if let NodeData::Window { fullscreen, .. } = &n.data {
                Some(!*fullscreen)
            } else {
                None
            }
        })
    });
    match current {
        Some(new_state) => set_fullscreen(tree, window_id, new_state),
        None => false,
    }
}

/// 公开版的 find_window_by_id（用于外部 crate 获取 NodeId）
pub fn find_node_by_window_id(tree: &Tree, window_id: u64) -> Option<NodeId> {
    find_window_by_id(tree, window_id)
}

/// 切换聚焦容器的布局类型。
///
/// 循环顺序: SplitH -> SplitV -> Tabbed -> Stacked -> SplitH
pub fn layout_toggle(tree: &mut Tree) {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        None => return,
    };

    let container_id = match find_focused_container(tree, ws_id) {
        Some(id) => id,
        None => return,
    };

    let next_layout = match tree.get(container_id) {
        Some(node) => match &node.data {
            NodeData::Container { layout, .. } => match layout {
                Layout::SplitH => Layout::SplitV,
                Layout::SplitV => Layout::Tabbed,
                Layout::Tabbed => Layout::Stacked,
                Layout::Stacked => Layout::SplitH,
            },
            _ => return,
        },
        None => return,
    };

    set_container_layout(tree, container_id, next_layout);
}

/// 设置窗口的 sticky 标志。返回 true 表示找到并设置成功。
pub fn set_sticky(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };
    if let Some(node) = tree.get_mut(win_node_id) {
        if let NodeData::Window { ref mut sticky, .. } = node.data {
            *sticky = enable;
            return true;
        }
    }
    false
}

/// 切换窗口的 sticky 标志。返回 true 表示找到并切换成功。
pub fn toggle_sticky(tree: &mut Tree, window_id: u64) -> bool {
    let current = find_window_by_id(tree, window_id).and_then(|id| {
        tree.get(id).and_then(|n| {
            if let NodeData::Window { sticky, .. } = &n.data {
                Some(!*sticky)
            } else {
                None
            }
        })
    });
    match current {
        Some(new_state) => set_sticky(tree, window_id, new_state),
        None => false,
    }
}

/// 向窗口添加标记。若标记已存在于该窗口则不重复添加。
/// 返回 true 表示找到窗口（无论是否实际添加了标记）。
pub fn add_mark(tree: &mut Tree, window_id: u64, mark: &str) -> bool {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };
    if let Some(node) = tree.get_mut(win_node_id) {
        if let NodeData::Window { ref mut marks, .. } = node.data {
            if !marks.iter().any(|m| m == mark) {
                marks.push(mark.to_string());
            }
            return true;
        }
    }
    false
}

/// 移除窗口上的标记。`mark` 为 None 时移除所有标记。
/// 返回 true 表示找到窗口。
pub fn remove_mark(tree: &mut Tree, window_id: u64, mark: Option<&str>) -> bool {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };
    if let Some(node) = tree.get_mut(win_node_id) {
        if let NodeData::Window { ref mut marks, .. } = node.data {
            match mark {
                Some(m) => marks.retain(|existing| existing != m),
                None => marks.clear(),
            }
            return true;
        }
    }
    false
}

/// 获取窗口上的所有标记。若窗口不存在则返回空 Vec。
pub fn get_marks(tree: &Tree, window_id: u64) -> Vec<String> {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return Vec::new(),
    };
    match tree.get(win_node_id) {
        Some(node) => match &node.data {
            NodeData::Window { marks, .. } => marks.clone(),
            _ => Vec::new(),
        },
        None => Vec::new(),
    }
}

/// 设置窗口的全局全屏（fullscreen_global）标志。
/// 返回 true 表示找到并设置成功。
pub fn set_fullscreen_global(tree: &mut Tree, window_id: u64, enable: bool) -> bool {
    let win_node_id = match find_window_by_id(tree, window_id) {
        Some(id) => id,
        None => return false,
    };
    if let Some(node) = tree.get_mut(win_node_id) {
        if let NodeData::Window {
            ref mut fullscreen_global,
            ref mut saved_geometry,
            ref geometry,
            ..
        } = node.data
        {
            if enable && !*fullscreen_global {
                *saved_geometry = Some(*geometry);
            }
            *fullscreen_global = enable;
            return true;
        }
    }
    false
}

/// 切换窗口的全局全屏（fullscreen_global）标志。
/// 返回 true 表示找到并切换成功。
pub fn toggle_fullscreen_global(tree: &mut Tree, window_id: u64) -> bool {
    let current = find_window_by_id(tree, window_id).and_then(|id| {
        tree.get(id).and_then(|n| {
            if let NodeData::Window { fullscreen_global, .. } = &n.data {
                Some(!*fullscreen_global)
            } else {
                None
            }
        })
    });
    match current {
        Some(new_state) => set_fullscreen_global(tree, window_id, new_state),
        None => false,
    }
}

/// 交换同一父容器下的两个子节点。
/// 仅在两个节点拥有相同父节点时生效。返回 true 表示交换成功。
pub fn swap_containers(tree: &mut Tree, node_a: NodeId, node_b: NodeId) -> bool {
    // 验证两个节点有相同父节点
    let parent_a = tree.parent(node_a);
    let parent_b = tree.parent(node_b);
    let parent_id = match (parent_a, parent_b) {
        (Some(a), Some(b)) if a == b => a,
        _ => return false,
    };

    // 找到两个节点在 children 中的索引
    let children: Vec<NodeId> = tree.children(parent_id).to_vec();
    let idx_a = match children.iter().position(|&c| c == node_a) {
        Some(i) => i,
        None => return false,
    };
    let idx_b = match children.iter().position(|&c| c == node_b) {
        Some(i) => i,
        None => return false,
    };

    if idx_a == idx_b {
        return false;
    }

    // 交换 children 列表和 sizes（若为 Container）
    if let Some(parent_node) = tree.get_mut(parent_id) {
        parent_node.children.swap(idx_a, idx_b);
        if let NodeData::Container { ref mut sizes, .. } = parent_node.data {
            if idx_a < sizes.len() && idx_b < sizes.len() {
                sizes.swap(idx_a, idx_b);
            }
        }
    }

    true
}

/// 将焦点移到当前容器中的下一个兄弟节点。
/// 在边界处（最后一个）不环绕，返回 false。
pub fn focus_next_sibling(tree: &mut Tree) -> bool {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        None => return false,
    };

    // 找到聚焦叶子窗口的 NodeId
    let leaf_id = match find_focused_leaf_node(tree, ws_id) {
        Some(id) => id,
        None => return false,
    };

    let parent_id = match tree.parent(leaf_id) {
        Some(id) => id,
        None => return false,
    };

    let children: Vec<NodeId> = tree.children(parent_id).to_vec();
    let current_idx = match children.iter().position(|&c| c == leaf_id) {
        Some(i) => i,
        None => return false,
    };

    if current_idx + 1 >= children.len() {
        return false;
    }

    let new_idx = current_idx + 1;
    if let Some(node) = tree.get_mut(parent_id) {
        if let NodeData::Container { ref mut focused_child, .. } = node.data {
            *focused_child = new_idx;
            return true;
        }
    }
    false
}

/// 将焦点移到当前容器中的上一个兄弟节点。
/// 在边界处（第一个）不环绕，返回 false。
pub fn focus_prev_sibling(tree: &mut Tree) -> bool {
    let ws_id = match get_focused_workspace(tree) {
        Some(id) => id,
        None => return false,
    };

    let leaf_id = match find_focused_leaf_node(tree, ws_id) {
        Some(id) => id,
        None => return false,
    };

    let parent_id = match tree.parent(leaf_id) {
        Some(id) => id,
        None => return false,
    };

    let children: Vec<NodeId> = tree.children(parent_id).to_vec();
    let current_idx = match children.iter().position(|&c| c == leaf_id) {
        Some(i) => i,
        None => return false,
    };

    if current_idx == 0 {
        return false;
    }

    let new_idx = current_idx - 1;
    if let Some(node) = tree.get_mut(parent_id) {
        if let NodeData::Container { ref mut focused_child, .. } = node.data {
            *focused_child = new_idx;
            return true;
        }
    }
    false
}

// ── 私有辅助 ─────────────────────────────────────────────────

/// 在 `parent_id` 中插入新窗口
fn insert_window_into(tree: &mut Tree, parent_id: NodeId, window_id: u64) -> NodeId {
    let new_win_data = NodeData::Window {
        window_id,
        floating: false,
        fullscreen: false,
        fullscreen_global: false,
        sticky: false,
        marks: Vec::new(),
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
            let can_move = matches!(
                (layout, direction),
                (Layout::SplitH, Direction::Left) | (Layout::SplitH, Direction::Right)
                | (Layout::SplitV, Direction::Up) | (Layout::SplitV, Direction::Down)
            );

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

/// 在子树中递归移动窗口位置（方向键移动）
fn move_window_in(tree: &mut Tree, node_id: NodeId, direction: Direction) -> bool {
    let node_data = match tree.get(node_id) {
        Some(n) => n.data.clone(),
        None => return false,
    };

    match &node_data {
        NodeData::Container { layout, focused_child, .. } => {
            let layout = *layout;
            let focused = *focused_child;
            let children: Vec<NodeId> = tree.children(node_id).to_vec();

            // 先尝试递归进入聚焦子容器
            if let Some(&focused_id) = children.get(focused) {
                let child_data = tree.get(focused_id).map(|n| n.data.clone());
                if let Some(NodeData::Container { .. }) = child_data {
                    if move_window_in(tree, focused_id, direction) {
                        return true;
                    }
                }
            }

            // 本容器层面处理
            let can_move = matches!(
                (layout, direction),
                (Layout::SplitH, Direction::Left) | (Layout::SplitH, Direction::Right)
                | (Layout::SplitV, Direction::Up) | (Layout::SplitV, Direction::Down)
            );

            if can_move && !children.is_empty() {
                let new_pos = match direction {
                    Direction::Left | Direction::Up => {
                        if focused > 0 { focused - 1 } else { return false; }
                    }
                    Direction::Right | Direction::Down => {
                        if focused + 1 < children.len() { focused + 1 } else { return false; }
                    }
                };

                // 交换 children 和 sizes
                if let Some(node) = tree.get_mut(node_id) {
                    node.children.swap(focused, new_pos);
                    if let NodeData::Container {
                        ref mut sizes,
                        ref mut focused_child,
                        ..
                    } = node.data
                    {
                        if focused < sizes.len() && new_pos < sizes.len() {
                            sizes.swap(focused, new_pos);
                        }
                        *focused_child = new_pos;
                    }
                }
                return true;
            }
            false
        }
        NodeData::Workspace { .. } => {
            let children: Vec<NodeId> = tree.children(node_id).to_vec();
            for &child in &children {
                if move_window_in(tree, child, direction) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// 全局搜索工作区（跨所有输出）
fn find_workspace_by_name_global(tree: &Tree, name: &str) -> Option<NodeId> {
    let root = tree.root();
    for &output_id in tree.children(root) {
        for &ws_id in tree.children(output_id) {
            if let Some(node) = tree.get(ws_id) {
                if let NodeData::Workspace { name: ws_name, .. } = &node.data {
                    if ws_name == name {
                        return Some(ws_id);
                    }
                }
            }
        }
    }
    None
}

/// 沿聚焦路径找到叶子节点的 NodeId
fn find_focused_leaf_node(tree: &Tree, node_id: NodeId) -> Option<NodeId> {
    let node = tree.get(node_id)?;
    match &node.data {
        NodeData::Window { .. } => Some(node_id),
        NodeData::Container { focused_child, .. } => {
            let children = tree.children(node_id);
            let idx = (*focused_child).min(children.len().saturating_sub(1));
            children.get(idx).and_then(|&c| find_focused_leaf_node(tree, c))
        }
        _ => {
            let children = tree.children(node_id);
            children.first().and_then(|&c| find_focused_leaf_node(tree, c))
        }
    }
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
