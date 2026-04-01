// Arena-allocated N-ary tree for i3/Sway-compatible tiling layout

/// 矩形区域，用于描述节点的几何位置与尺寸
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    /// 判断矩形面积是否有效（宽高均为正数）
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// 节点标识符（Arena 索引包装器）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// 布局类型，对应 i3/Sway 的四种容器布局
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    SplitH,
    SplitV,
    Tabbed,
    Stacked,
}

/// 节点携带的业务数据
#[derive(Debug, Clone)]
pub enum NodeData {
    /// 虚拟根节点，整棵树唯一
    Root,

    /// 物理输出（显示器）
    Output {
        name: String,
        geometry: Rect,
    },

    /// 工作区，归属某个输出
    Workspace {
        name: String,
        output: NodeId,
        is_visible: bool,
    },

    /// 分割/标签/堆叠容器
    Container {
        layout: Layout,
        /// 各子节点的比例大小（之和可以为任意正数，计算时按比例分配）
        sizes: Vec<f64>,
        /// 当前聚焦的子节点索引
        focused_child: usize,
    },

    /// 实际窗口
    Window {
        window_id: u64,
        floating: bool,
        fullscreen: bool,
        geometry: Rect,
        /// 进入浮动/全屏前保存的几何信息
        saved_geometry: Option<Rect>,
    },
}

/// Arena 树中的单个节点
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub data: NodeData,
}

/// Arena-allocated N 叉树
///
/// 使用 `Vec<Option<Node>>` 存储节点，配合 free_list 复用已释放的槽位。
pub struct Tree {
    nodes: Vec<Option<Node>>,
    free_list: Vec<NodeId>,
    root: NodeId,
}

impl Tree {
    /// 创建带有根节点的空树
    pub fn new() -> Self {
        let root_node = Node {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: NodeData::Root,
        };
        Tree {
            nodes: vec![Some(root_node)],
            free_list: Vec::new(),
            root: NodeId(0),
        }
    }

    /// 根节点 ID
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// 向 parent 添加子节点，返回新节点的 ID
    pub fn add_node(&mut self, parent: NodeId, data: NodeData) -> NodeId {
        let id = if let Some(reused) = self.free_list.pop() {
            reused
        } else {
            let idx = self.nodes.len();
            self.nodes.push(None);
            NodeId(idx)
        };

        let node = Node {
            id,
            parent: Some(parent),
            children: Vec::new(),
            data,
        };
        self.nodes[id.0] = Some(node);

        // 将自己加入父节点的 children
        if let Some(parent_node) = self.nodes[parent.0].as_mut() {
            parent_node.children.push(id);
        }

        id
    }

    /// 删除节点及其所有后代，释放的槽位加入 free_list
    pub fn remove_node(&mut self, id: NodeId) {
        // 收集所有后代（含自身），使用迭代而非递归避免栈溢出
        let mut to_remove = Vec::new();
        let mut stack = vec![id];
        while let Some(cur) = stack.pop() {
            if let Some(node) = self.nodes[cur.0].as_ref() {
                for &child in &node.children {
                    stack.push(child);
                }
            }
            to_remove.push(cur);
        }

        // 从父节点的 children 列表中移除 id
        if let Some(node) = self.nodes[id.0].as_ref() {
            if let Some(parent_id) = node.parent {
                if let Some(parent_node) = self.nodes[parent_id.0].as_mut() {
                    parent_node.children.retain(|&c| c != id);
                }
            }
        }

        // 置空并回收槽位
        for rid in to_remove {
            self.nodes[rid.0] = None;
            self.free_list.push(rid);
        }
    }

    /// 不可变引用
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)?.as_ref()
    }

    /// 可变引用
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.0)?.as_mut()
    }

    /// 返回子节点 ID 切片
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        match self.nodes.get(id.0) {
            Some(Some(node)) => &node.children,
            _ => &[],
        }
    }

    /// 返回父节点 ID
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes.get(id.0)?.as_ref()?.parent
    }

    /// 当前存活节点数量（用于测试/调试）
    pub fn node_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_some()).count()
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 单元测试（RED → GREEN → IMPROVE）
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── 辅助函数 ────────────────────────────────────────────

    fn sample_rect() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    fn output_data(name: &str) -> NodeData {
        NodeData::Output {
            name: name.to_string(),
            geometry: sample_rect(),
        }
    }

    fn window_data(id: u64) -> NodeData {
        NodeData::Window {
            window_id: id,
            floating: false,
            fullscreen: false,
            geometry: Rect::new(0, 0, 0, 0),
            saved_geometry: None,
        }
    }

    fn container_data(layout: Layout) -> NodeData {
        NodeData::Container {
            layout,
            sizes: Vec::new(),
            focused_child: 0,
        }
    }

    // ── Rect 测试 ──────────────────────────────────────────

    #[test]
    fn rect_new_stores_fields() {
        let r = Rect::new(10, 20, 800, 600);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 800);
        assert_eq!(r.height, 600);
    }

    #[test]
    fn rect_is_valid_positive_dimensions() {
        assert!(Rect::new(0, 0, 1, 1).is_valid());
        assert!(Rect::new(-100, -100, 800, 600).is_valid());
    }

    #[test]
    fn rect_is_invalid_zero_width() {
        assert!(!Rect::new(0, 0, 0, 600).is_valid());
    }

    #[test]
    fn rect_is_invalid_zero_height() {
        assert!(!Rect::new(0, 0, 800, 0).is_valid());
    }

    #[test]
    fn rect_is_invalid_negative_dimensions() {
        assert!(!Rect::new(0, 0, -1, 600).is_valid());
        assert!(!Rect::new(0, 0, 800, -1).is_valid());
    }

    #[test]
    fn rect_equality() {
        let a = Rect::new(0, 0, 100, 100);
        let b = Rect::new(0, 0, 100, 100);
        assert_eq!(a, b);
    }

    // ── Tree::new 测试 ─────────────────────────────────────

    #[test]
    fn tree_new_has_root() {
        let tree = Tree::new();
        let root = tree.root();
        assert!(tree.get(root).is_some());
    }

    #[test]
    fn tree_new_root_is_root_data() {
        let tree = Tree::new();
        let node = tree.get(tree.root()).unwrap();
        assert!(matches!(node.data, NodeData::Root));
    }

    #[test]
    fn tree_new_root_has_no_parent() {
        let tree = Tree::new();
        assert!(tree.parent(tree.root()).is_none());
    }

    #[test]
    fn tree_new_root_has_no_children() {
        let tree = Tree::new();
        assert!(tree.children(tree.root()).is_empty());
    }

    #[test]
    fn tree_new_node_count_is_one() {
        let tree = Tree::new();
        assert_eq!(tree.node_count(), 1);
    }

    // ── add_node 测试 ──────────────────────────────────────

    #[test]
    fn add_node_returns_valid_id() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        assert!(tree.get(child).is_some());
    }

    #[test]
    fn add_node_parent_has_child() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        assert!(tree.children(root).contains(&child));
    }

    #[test]
    fn add_node_child_has_parent() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        assert_eq!(tree.parent(child), Some(root));
    }

    #[test]
    fn add_node_increments_count() {
        let mut tree = Tree::new();
        let root = tree.root();
        tree.add_node(root, output_data("eDP-1"));
        assert_eq!(tree.node_count(), 2);
    }

    #[test]
    fn add_multiple_children() {
        let mut tree = Tree::new();
        let root = tree.root();
        let c1 = tree.add_node(root, output_data("eDP-1"));
        let c2 = tree.add_node(root, output_data("HDMI-1"));
        let children = tree.children(root);
        assert!(children.contains(&c1));
        assert!(children.contains(&c2));
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn add_nested_nodes() {
        let mut tree = Tree::new();
        let root = tree.root();
        let output = tree.add_node(root, output_data("eDP-1"));
        let ws = tree.add_node(output, NodeData::Workspace {
            name: "1".into(),
            output,
            is_visible: true,
        });
        let win = tree.add_node(ws, window_data(42));

        assert_eq!(tree.parent(win), Some(ws));
        assert_eq!(tree.parent(ws), Some(output));
        assert_eq!(tree.parent(output), Some(root));
    }

    // ── remove_node 测试 ───────────────────────────────────

    #[test]
    fn remove_node_leaf_decrements_count() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        tree.remove_node(child);
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn remove_node_leaf_not_found_after_removal() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        tree.remove_node(child);
        assert!(tree.get(child).is_none());
    }

    #[test]
    fn remove_node_removed_from_parent_children() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        tree.remove_node(child);
        assert!(!tree.children(root).contains(&child));
    }

    #[test]
    fn remove_node_also_removes_descendants() {
        let mut tree = Tree::new();
        let root = tree.root();
        let output = tree.add_node(root, output_data("eDP-1"));
        let ws = tree.add_node(output, NodeData::Workspace {
            name: "1".into(),
            output,
            is_visible: true,
        });
        let win = tree.add_node(ws, window_data(1));

        tree.remove_node(output);

        assert!(tree.get(output).is_none());
        assert!(tree.get(ws).is_none());
        assert!(tree.get(win).is_none());
        assert_eq!(tree.node_count(), 1); // 只剩 root
    }

    #[test]
    fn remove_node_recycles_slot_for_new_node() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        let child_idx = child.0;
        tree.remove_node(child);

        // 新添加的节点应复用已释放的槽位
        let new_child = tree.add_node(root, output_data("HDMI-1"));
        assert_eq!(new_child.0, child_idx);
    }

    #[test]
    fn remove_node_sibling_remains() {
        let mut tree = Tree::new();
        let root = tree.root();
        let c1 = tree.add_node(root, output_data("eDP-1"));
        let c2 = tree.add_node(root, output_data("HDMI-1"));
        tree.remove_node(c1);
        assert!(tree.get(c2).is_some());
        assert!(tree.children(root).contains(&c2));
    }

    // ── get / get_mut 测试 ─────────────────────────────────

    #[test]
    fn get_returns_none_for_invalid_id() {
        let tree = Tree::new();
        assert!(tree.get(NodeId(999)).is_none());
    }

    #[test]
    fn get_mut_modifies_node() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, window_data(1));

        if let Some(node) = tree.get_mut(child) {
            if let NodeData::Window { ref mut floating, .. } = node.data {
                *floating = true;
            }
        }

        if let Some(node) = tree.get(child) {
            if let NodeData::Window { floating, .. } = node.data {
                assert!(floating);
            } else {
                panic!("节点数据类型错误");
            }
        }
    }

    // ── children / parent 边界测试 ─────────────────────────

    #[test]
    fn children_returns_empty_for_invalid_id() {
        let tree = Tree::new();
        assert!(tree.children(NodeId(999)).is_empty());
    }

    #[test]
    fn parent_returns_none_for_invalid_id() {
        let tree = Tree::new();
        assert!(tree.parent(NodeId(999)).is_none());
    }

    // ── NodeData 克隆测试 ──────────────────────────────────

    #[test]
    fn node_data_container_clone() {
        let data = container_data(Layout::SplitH);
        let cloned = data.clone();
        assert!(matches!(cloned, NodeData::Container { layout: Layout::SplitH, .. }));
    }

    #[test]
    fn layout_equality() {
        assert_eq!(Layout::SplitH, Layout::SplitH);
        assert_ne!(Layout::SplitH, Layout::SplitV);
    }

    // ── NodeId 哈希/相等 ───────────────────────────────────

    #[test]
    fn node_id_equality() {
        assert_eq!(NodeId(0), NodeId(0));
        assert_ne!(NodeId(0), NodeId(1));
    }

    #[test]
    fn node_id_in_hashset() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NodeId(0));
        set.insert(NodeId(1));
        set.insert(NodeId(0)); // 重复
        assert_eq!(set.len(), 2);
    }

    // ── 大量节点压力测试 ───────────────────────────────────

    #[test]
    fn add_many_nodes_and_remove_all() {
        let mut tree = Tree::new();
        let root = tree.root();
        let mut ids = Vec::new();
        for i in 0..1000u64 {
            ids.push(tree.add_node(root, window_data(i)));
        }
        assert_eq!(tree.node_count(), 1001);

        for id in &ids {
            tree.remove_node(*id);
        }
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn free_list_reuse_across_many_cycles() {
        let mut tree = Tree::new();
        let root = tree.root();

        // 添加 → 删除 → 再添加，验证复用
        for cycle in 0..10u64 {
            let id = tree.add_node(root, window_data(cycle));
            tree.remove_node(id);
        }
        // 最终只剩 root
        assert_eq!(tree.node_count(), 1);
    }
}
