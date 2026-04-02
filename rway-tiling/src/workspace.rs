// Workspace management: create, switch, move windows between workspaces

//! 工作区管理：输出（显示器）注册、工作区创建与切换。

use crate::tree::{NodeData, NodeId, Rect, Tree};

// ── 公开函数 ─────────────────────────────────────────────────

/// 向 Root 节点注册一个物理输出（显示器）。
pub fn add_output(tree: &mut Tree, name: &str, geometry: Rect) -> NodeId {
    let root = tree.root();
    tree.add_node(
        root,
        NodeData::Output {
            name: name.to_string(),
            geometry,
        },
    )
}

/// 在指定输出下创建工作区，并设为该输出的唯一可见工作区（初始可见）。
///
/// 若已存在同名工作区，则直接返回已有的节点 ID。
pub fn add_workspace(tree: &mut Tree, output_id: NodeId, name: &str) -> NodeId {
    // 检查是否已存在同名工作区
    let existing: Vec<NodeId> = tree.children(output_id).to_vec();
    for child_id in existing {
        if let Some(node) = tree.get(child_id) {
            if let NodeData::Workspace { name: ws_name, .. } = &node.data {
                if ws_name == name {
                    return child_id;
                }
            }
        }
    }

    // 新建工作区，默认可见
    tree.add_node(
        output_id,
        NodeData::Workspace {
            name: name.to_string(),
            output: output_id,
            is_visible: true,
        },
    )
}

/// 获取当前（第一个）可见工作区 ID。
///
/// 遍历所有输出的所有工作区，返回第一个 `is_visible == true` 的工作区。
pub fn get_focused_workspace(tree: &Tree) -> Option<NodeId> {
    let root = tree.root();
    for &output_id in tree.children(root) {
        for &ws_id in tree.children(output_id) {
            if let Some(node) = tree.get(ws_id) {
                if let NodeData::Workspace { is_visible, .. } = &node.data {
                    if *is_visible {
                        return Some(ws_id);
                    }
                }
            }
        }
    }
    None
}

/// 切换到指定名称的工作区：隐藏该输出上的其他工作区，显示目标工作区。
///
/// 如果找到目标工作区，返回 `true`；否则 `false`。
pub fn switch_workspace(tree: &mut Tree, workspace_name: &str) -> bool {
    // 先找目标工作区及其所属输出
    let target = find_workspace_by_name(tree, workspace_name);
    let (target_id, output_id) = match target {
        Some(v) => v,
        None => return false,
    };

    // 同一输出下：隐藏所有工作区，再显示目标
    let siblings: Vec<NodeId> = tree.children(output_id).to_vec();
    for ws_id in siblings {
        if let Some(node) = tree.get_mut(ws_id) {
            if let NodeData::Workspace {
                ref mut is_visible, ..
            } = node.data
            {
                *is_visible = ws_id == target_id;
            }
        }
    }
    true
}

/// 返回所有工作区的 `(id, name, is_visible)` 列表，按创建顺序排列。
pub fn get_workspaces(tree: &Tree) -> Vec<(NodeId, String, bool)> {
    let root = tree.root();
    let mut result = Vec::new();

    for &output_id in tree.children(root) {
        for &ws_id in tree.children(output_id) {
            if let Some(node) = tree.get(ws_id) {
                if let NodeData::Workspace {
                    name, is_visible, ..
                } = &node.data
                {
                    result.push((ws_id, name.clone(), *is_visible));
                }
            }
        }
    }
    result
}

/// 为工作区添加跨输出支持：将工作区关联到多个输出上。
///
/// 实现方式：在每个额外的输出节点下创建同名工作区（如已存在则跳过）。
/// 返回 `true` 表示至少有一个输出成功关联。
pub fn span_workspace_outputs(tree: &mut Tree, workspace: NodeId, outputs: &[NodeId]) -> bool {
    let ws_name = match tree.get(workspace) {
        Some(node) => match &node.data {
            NodeData::Workspace { name, .. } => name.clone(),
            _ => return false,
        },
        None => return false,
    };

    let mut any_added = false;
    for &output_id in outputs {
        // 确认目标是 Output 节点
        let is_output = tree
            .get(output_id)
            .map(|n| matches!(n.data, NodeData::Output { .. }))
            .unwrap_or(false);
        if !is_output {
            continue;
        }

        // 跳过与工作区同一个输出
        let same_output = tree
            .get(workspace)
            .and_then(|n| {
                if let NodeData::Workspace { output, .. } = &n.data {
                    Some(*output == output_id)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        if same_output {
            continue;
        }

        // 在该输出下添加同名工作区（add_workspace 内部会去重）
        let children_before = tree.children(output_id).len();
        let _new_ws = add_workspace(tree, output_id, &ws_name);
        let children_after = tree.children(output_id).len();
        if children_after > children_before {
            any_added = true;
        }
    }
    any_added
}

/// 重命名工作区。
pub fn rename_workspace(tree: &mut Tree, old_name: &str, new_name: &str) -> bool {
    let root = tree.root();
    for &output_id in tree.children(root).to_vec().iter() {
        for &ws_id in tree.children(output_id).to_vec().iter() {
            if let Some(node) = tree.get_mut(ws_id) {
                if let NodeData::Workspace { ref mut name, .. } = node.data {
                    if name.as_str() == old_name {
                        *name = new_name.to_string();
                        return true;
                    }
                }
            }
        }
    }
    false
}

// ── 私有辅助 ─────────────────────────────────────────────────

fn find_workspace_by_name(tree: &Tree, name: &str) -> Option<(NodeId, NodeId)> {
    let root = tree.root();
    for &output_id in tree.children(root) {
        for &ws_id in tree.children(output_id) {
            if let Some(node) = tree.get(ws_id) {
                if let NodeData::Workspace { name: ws_name, .. } = &node.data {
                    if ws_name == name {
                        return Some((ws_id, output_id));
                    }
                }
            }
        }
    }
    None
}

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Rect, Tree};

    fn screen() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    fn second_screen() -> Rect {
        Rect::new(1920, 0, 2560, 1440)
    }

    // ── add_output ──────────────────────────────────────────

    #[test]
    fn add_output_creates_output_node() {
        let mut tree = Tree::new();
        let id = add_output(&mut tree, "eDP-1", screen());
        let node = tree.get(id).unwrap();
        assert!(matches!(node.data, NodeData::Output { .. }));
    }

    #[test]
    fn add_output_sets_name_and_geometry() {
        let mut tree = Tree::new();
        let id = add_output(&mut tree, "HDMI-1", screen());
        if let NodeData::Output { name, geometry } = &tree.get(id).unwrap().data {
            assert_eq!(name, "HDMI-1");
            assert_eq!(*geometry, screen());
        } else {
            panic!();
        }
    }

    #[test]
    fn add_output_is_child_of_root() {
        let mut tree = Tree::new();
        let id = add_output(&mut tree, "eDP-1", screen());
        assert_eq!(tree.parent(id), Some(tree.root()));
    }

    #[test]
    fn add_multiple_outputs() {
        let mut tree = Tree::new();
        let o1 = add_output(&mut tree, "eDP-1", screen());
        let o2 = add_output(&mut tree, "HDMI-1", second_screen());
        let children = tree.children(tree.root());
        assert!(children.contains(&o1));
        assert!(children.contains(&o2));
    }

    // ── add_workspace ───────────────────────────────────────

    #[test]
    fn add_workspace_creates_workspace_node() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let ws = add_workspace(&mut tree, out, "1");
        assert!(matches!(
            tree.get(ws).unwrap().data,
            NodeData::Workspace { .. }
        ));
    }

    #[test]
    fn add_workspace_sets_name_and_visible() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let ws = add_workspace(&mut tree, out, "work");
        if let NodeData::Workspace {
            name, is_visible, ..
        } = &tree.get(ws).unwrap().data
        {
            assert_eq!(name, "work");
            assert!(*is_visible);
        } else {
            panic!();
        }
    }

    #[test]
    fn add_workspace_deduplicates_same_name() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let ws1 = add_workspace(&mut tree, out, "1");
        let ws2 = add_workspace(&mut tree, out, "1");
        assert_eq!(ws1, ws2);
        assert_eq!(tree.children(out).len(), 1);
    }

    #[test]
    fn add_workspace_different_names() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let ws1 = add_workspace(&mut tree, out, "1");
        let ws2 = add_workspace(&mut tree, out, "2");
        assert_ne!(ws1, ws2);
        assert_eq!(tree.children(out).len(), 2);
    }

    // ── get_focused_workspace ───────────────────────────────

    #[test]
    fn get_focused_workspace_returns_visible() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let ws = add_workspace(&mut tree, out, "1");
        let focused = get_focused_workspace(&tree);
        assert_eq!(focused, Some(ws));
    }

    #[test]
    fn get_focused_workspace_returns_none_when_empty() {
        let tree = Tree::new();
        assert!(get_focused_workspace(&tree).is_none());
    }

    #[test]
    fn get_focused_workspace_returns_none_when_all_hidden() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let ws = add_workspace(&mut tree, out, "1");

        // 手动隐藏
        if let Some(node) = tree.get_mut(ws) {
            if let NodeData::Workspace {
                ref mut is_visible, ..
            } = node.data
            {
                *is_visible = false;
            }
        }

        assert!(get_focused_workspace(&tree).is_none());
    }

    // ── switch_workspace ────────────────────────────────────

    #[test]
    fn switch_workspace_makes_target_visible() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let _ws1 = add_workspace(&mut tree, out, "1");
        let ws2 = add_workspace(&mut tree, out, "2");

        // 手动隐藏 ws2
        if let Some(n) = tree.get_mut(ws2) {
            if let NodeData::Workspace {
                ref mut is_visible, ..
            } = n.data
            {
                *is_visible = false;
            }
        }

        let ok = switch_workspace(&mut tree, "2");
        assert!(ok);

        if let Some(n) = tree.get(ws2) {
            if let NodeData::Workspace { is_visible, .. } = &n.data {
                assert!(*is_visible);
            }
        }
    }

    #[test]
    fn switch_workspace_hides_other_workspaces_on_same_output() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        let ws1 = add_workspace(&mut tree, out, "1");
        let ws2 = add_workspace(&mut tree, out, "2");

        // 初始状态 ws1, ws2 都可见；切换到 ws2 后 ws1 应隐藏
        switch_workspace(&mut tree, "2");

        let ws_list = get_workspaces(&tree);
        let vis: Vec<(String, bool)> = ws_list.into_iter().map(|(_, n, v)| (n, v)).collect();

        assert!(vis.contains(&("2".into(), true)));
        assert!(vis.contains(&("1".into(), false)));
        let _ = (ws1, ws2);
    }

    #[test]
    fn switch_workspace_returns_false_for_nonexistent() {
        let mut tree = Tree::new();
        let _ = add_output(&mut tree, "eDP-1", screen());
        let ok = switch_workspace(&mut tree, "nonexistent");
        assert!(!ok);
    }

    #[test]
    fn switch_workspace_does_not_affect_other_outputs() {
        let mut tree = Tree::new();
        let out1 = add_output(&mut tree, "eDP-1", screen());
        let out2 = add_output(&mut tree, "HDMI-1", second_screen());

        let _ws1 = add_workspace(&mut tree, out1, "1");
        let ws_other = add_workspace(&mut tree, out2, "other");

        switch_workspace(&mut tree, "1");

        // out2 上的工作区可见性不受影响
        if let Some(n) = tree.get(ws_other) {
            if let NodeData::Workspace { is_visible, .. } = &n.data {
                assert!(*is_visible, "另一个输出上的工作区不应被隐藏");
            }
        }
    }

    // ── get_workspaces ──────────────────────────────────────

    #[test]
    fn get_workspaces_returns_all() {
        let mut tree = Tree::new();
        let out = add_output(&mut tree, "eDP-1", screen());
        add_workspace(&mut tree, out, "1");
        add_workspace(&mut tree, out, "2");
        add_workspace(&mut tree, out, "3");

        let ws_list = get_workspaces(&tree);
        assert_eq!(ws_list.len(), 3);
    }

    #[test]
    fn get_workspaces_empty_when_no_outputs() {
        let tree = Tree::new();
        assert!(get_workspaces(&tree).is_empty());
    }

    #[test]
    fn get_workspaces_empty_when_no_workspaces() {
        let mut tree = Tree::new();
        let _ = add_output(&mut tree, "eDP-1", screen());
        assert!(get_workspaces(&tree).is_empty());
    }

    #[test]
    fn get_workspaces_across_multiple_outputs() {
        let mut tree = Tree::new();
        let out1 = add_output(&mut tree, "eDP-1", screen());
        let out2 = add_output(&mut tree, "HDMI-1", second_screen());
        add_workspace(&mut tree, out1, "1");
        add_workspace(&mut tree, out2, "2");

        let ws_list = get_workspaces(&tree);
        assert_eq!(ws_list.len(), 2);
        let names: Vec<String> = ws_list.into_iter().map(|(_, n, _)| n).collect();
        assert!(names.contains(&"1".to_string()));
        assert!(names.contains(&"2".to_string()));
    }

    // ── span_workspace_outputs ──────────────────────────────

    #[test]
    fn span_workspace_adds_to_extra_output() {
        let mut tree = Tree::new();
        let out1 = add_output(&mut tree, "eDP-1", screen());
        let out2 = add_output(&mut tree, "HDMI-1", second_screen());
        let ws = add_workspace(&mut tree, out1, "shared");

        let ok = span_workspace_outputs(&mut tree, ws, &[out2]);
        assert!(ok);

        // out2 下应有同名工作区
        let names: Vec<String> = tree
            .children(out2)
            .iter()
            .filter_map(|&id| {
                if let NodeData::Workspace { name, .. } = &tree.get(id)?.data {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        assert!(names.contains(&"shared".to_string()));
    }

    #[test]
    fn span_workspace_skips_same_output() {
        let mut tree = Tree::new();
        let out1 = add_output(&mut tree, "eDP-1", screen());
        let ws = add_workspace(&mut tree, out1, "1");

        let before = tree.children(out1).len();
        let ok = span_workspace_outputs(&mut tree, ws, &[out1]);
        // 同一输出不算成功添加
        assert!(!ok);
        assert_eq!(tree.children(out1).len(), before);
    }

    #[test]
    fn span_workspace_skips_non_output_node() {
        let mut tree = Tree::new();
        let out1 = add_output(&mut tree, "eDP-1", screen());
        let ws = add_workspace(&mut tree, out1, "1");

        // ws 不是 Output 节点
        let ok = span_workspace_outputs(&mut tree, ws, &[ws]);
        assert!(!ok);
    }

    #[test]
    fn span_workspace_no_duplicate_on_same_span_twice() {
        let mut tree = Tree::new();
        let out1 = add_output(&mut tree, "eDP-1", screen());
        let out2 = add_output(&mut tree, "HDMI-1", second_screen());
        let ws = add_workspace(&mut tree, out1, "shared");

        span_workspace_outputs(&mut tree, ws, &[out2]);
        span_workspace_outputs(&mut tree, ws, &[out2]); // 再次调用

        let count = tree
            .children(out2)
            .iter()
            .filter(|&&id| {
                tree.get(id)
                    .map(|n| matches!(n.data, NodeData::Workspace { .. }))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(count, 1);
    }
}
