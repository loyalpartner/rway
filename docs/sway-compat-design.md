# Sway 兼容性对比与修复方案

> 基于 rway-tiling 实现分析（docs/rway-tiling-analysis.md）与 Sway 行为规范（docs/sway-spec.md）的系统对比。
> 所有修改保持 arena N-ary tree 架构不变。

---

## 不兼容清单

### INC-1: Tabbed/Stacked 容器不响应 focus 方向键 [HIGH]

**Sway 行为：**
- Tabbed 容器：`focus left/right` 在 tab 间切换（等同于 `focus prev/next`）
- Stacked 容器：`focus up/down` 在堆叠项间切换
- 这些容器对不匹配的方向应该透传给父容器处理

**当前 rway 行为：**
`move_focus_in()` (tree.rs:1672) 的 `can_move` 只匹配：
```rust
(Layout::SplitH, Direction::Left | Direction::Right)
| (Layout::SplitV, Direction::Up | Direction::Down)
```
Tabbed 和 Stacked 完全不匹配任何方向，焦点永远无法在 Tabbed/Stacked 容器的子节点间移动。

**修复方案：**
扩展 `can_move` 匹配逻辑：
```rust
let can_move = matches!(
    (layout, direction),
    (Layout::SplitH, Direction::Left | Direction::Right)
    | (Layout::SplitV, Direction::Up | Direction::Down)
    | (Layout::Tabbed, Direction::Left | Direction::Right)
    | (Layout::Stacked, Direction::Up | Direction::Down)
);
```

**影响范围：** `move_focus_in()` 方法，约 2 行代码修改。

---

### INC-2: Tabbed/Stacked 容器不响应 move 方向键 [HIGH]

**Sway 行为：**
- Tabbed 容器中 `move left/right` 重排 tab 顺序
- Stacked 容器中 `move up/down` 重排堆叠顺序
- 不匹配的方向应提取窗口到父容器

**当前 rway 行为：**
`move_window()` (tree.rs:756) 的 `axis_matches` 只匹配：
```rust
(Layout::SplitH, Direction::Left | Direction::Right)
| (Layout::SplitV, Direction::Up | Direction::Down)
```
Tabbed/Stacked 完全不响应。

**修复方案：**
扩展 `axis_matches`：
```rust
let axis_matches = matches!(
    (layout, direction),
    (Layout::SplitH, Direction::Left | Direction::Right)
    | (Layout::SplitV, Direction::Up | Direction::Down)
    | (Layout::Tabbed, Direction::Left | Direction::Right)
    | (Layout::Stacked, Direction::Up | Direction::Down)
);
```

**影响范围：** `move_window()` 方法，约 2 行代码修改。

---

### INC-3: focus_wrapping 未实现 [MEDIUM]

**Sway 行为：**
- `focus_wrapping yes`（默认）：到达容器边界时环绕到对面
- `focus_wrapping no`：到达边界时停止
- `focus_wrapping force`：忽略容器边界，穿透到父/兄弟容器
- `focus_wrapping workspace`：仅在 workspace 级别环绕

**当前 rway 行为：**
到达边界直接返回 `false`，等同于 `focus_wrapping no`。

**修复方案：**

Phase 1 — 实现 `focus_wrapping yes`（默认值）：
在 `move_focus_in()` 中，当 focused 到达边界时环绕：
```rust
let new_focus = match direction {
    Direction::Left | Direction::Up => {
        if focused > 0 {
            focused - 1
        } else if wrapping_enabled {
            children.len() - 1  // wrap to last
        } else {
            return false;
        }
    }
    Direction::Right | Direction::Down => {
        if focused + 1 < children.len() {
            focused + 1
        } else if wrapping_enabled {
            0  // wrap to first
        } else {
            return false;
        }
    }
};
```

Phase 2 — 需要在 Tree 中增加 `focus_wrapping` 配置字段：
```rust
pub struct Tree {
    // ...existing fields...
    pub focus_wrapping: FocusWrapping,
}

pub enum FocusWrapping {
    Yes,    // default
    No,
    Force,
    Workspace,
}
```

**影响范围：** `move_focus_in()` 方法 + Tree 结构新增字段。约 15 行修改。

---

### INC-4: resize 不递归查找匹配轴的祖先 [MEDIUM]

**Sway 行为：**
当 `resize grow width` 时，如果直接父容器是 SplitV（轴不匹配），Sway 会向上查找 SplitH 祖先容器，并调整该祖先中对应子节点的 size。

**当前 rway 行为：**
`resize_container()` (tree.rs:876) 只检查直接父容器，轴不匹配直接返回 `false`。

**修复方案：**
添加递归向上查找逻辑：
```rust
pub fn resize_container(&mut self, node_id: NodeId, axis: ResizeAxis, delta_ppt: f64) -> bool {
    // Walk up from node_id to find an ancestor container whose layout matches axis
    let mut current = node_id;
    loop {
        let parent_id = match self.parent(current) {
            Some(id) => id,
            None => return false,
        };

        let layout_matches = match self.get(parent_id) {
            Some(n) => match &n.data {
                NodeData::Container { layout, .. } => matches!(
                    (axis, layout),
                    (ResizeAxis::Width, Layout::SplitH)
                    | (ResizeAxis::Height, Layout::SplitV)
                ),
                NodeData::Workspace { .. } => return false, // stop at workspace
                _ => false,
            },
            None => return false,
        };

        if layout_matches {
            // Apply resize to `current` within `parent_id`
            return self.apply_resize(parent_id, current, delta_ppt);
        }
        current = parent_id;
    }
}
```

提取当前的 resize 逻辑为 `apply_resize(parent, child, delta)` 私有方法。

**影响范围：** `resize_container()` 重构为向上遍历 + 提取 `apply_resize()`。约 20 行修改。

---

### INC-5: Tabbed/Stacked 只计算聚焦窗口布局 [MEDIUM]

**Sway 行为：**
所有子窗口都获得相同的 geometry（与容器区域相同），只是非聚焦窗口不显示。这对于 IPC GET_TREE 返回正确的 rect 信息很重要。

**当前 rway 行为：**
`compute_layout()` (tree.rs:1896-1900) 对 Tabbed/Stacked 只调用 `compute_layout(focused_id, available, gaps)`。非聚焦子窗口 geometry 保持 (0,0,0,0)。

**修复方案：**
遍历所有子节点计算布局：
```rust
Layout::Tabbed | Layout::Stacked => {
    // All children get the same area (only focused one is displayed)
    for &child in &children {
        self.compute_layout(child, available, gaps);
    }
}
```

**影响范围：** `compute_layout()` 方法 Tabbed/Stacked 分支，约 3 行修改。

---

### INC-6: remove_window 后不同步焦点路径 [MEDIUM]

**Sway 行为：**
移除窗口后，焦点自动转移到相邻窗口，整个焦点路径保持一致。

**当前 rway 行为：**
`remove_window()` (tree.rs:564-583) 移除窗口后：
- `remove_container_size` 会调整 `focused_child`（钳制到 sizes.len()-1）
- 但不调用 `focus_window()` 同步上层容器的焦点路径
- 如果移除的是当前聚焦窗口，上层容器仍指向旧索引

**修复方案：**
在 `remove_window()` 末尾添加焦点同步：
```rust
pub fn remove_window(&mut self, window_id: u64) -> bool {
    // ...existing removal logic...

    // After removal, sync focus path from the new focused leaf
    if let Some(new_focused) = self.focused_window_id() {
        self.focus_window(new_focused);
    }
    true
}
```

**影响范围：** `remove_window()` 末尾添加 3 行。

---

### INC-7: insert_window 后不同步上层焦点路径 [LOW]

**Sway 行为：**
新窗口自动获得焦点，焦点路径从新窗口到 workspace 完全一致。

**当前 rway 行为：**
`insert_window_into()` 设置了直接父容器的 `focused_child`，但不同步上层容器。在深层嵌套中，上层容器的 `focused_child` 可能不指向包含新窗口的子树。

**修复方案：**
在 `insert_window` 返回前调用 `focus_window()`：
```rust
pub fn insert_window(&mut self, window_id: u64) -> NodeId {
    let node_id = self.insert_window_with_layout(window_id, Layout::SplitH);
    self.focus_window(window_id);  // sync full focus path
    node_id
}
```

**影响范围：** `insert_window()` 添加 1 行。

---

### INC-8: 单子节点容器未自动展平 [LOW]

**Sway 行为：**
当容器只剩一个子节点（通过移动或删除其他子节点后），Sway 会自动将该子节点提升到父容器，消除不必要的嵌套层级。

**当前 rway 行为：**
`cleanup_empty_container()` 只处理**空**容器（children 为空），不处理单子节点容器。例如从 `SplitV[A, B]` 中提取 A 后，`SplitV[B]` 会保留而不是展平。

**修复方案：**
扩展清理逻辑处理单子节点容器：
```rust
fn cleanup_single_child_container(&mut self, node_id: NodeId) {
    let (is_single, child_id) = match self.get(node_id) {
        Some(n) => match &n.data {
            NodeData::Container { .. } if n.children.len() == 1 => {
                (true, n.children[0])
            }
            _ => (false, NodeId(0)),
        },
        None => (false, NodeId(0)),
    };

    if !is_single { return; }

    let parent_id = match self.parent(node_id) {
        Some(id) => id,
        None => return,
    };

    // Replace node_id with child_id in parent's children
    let idx = self.children(parent_id).iter().position(|&c| c == node_id);
    if let Some(idx) = idx {
        if let Some(parent) = self.get_mut(parent_id) {
            parent.children[idx] = child_id;
        }
        if let Some(child) = self.get_mut(child_id) {
            child.parent = Some(parent_id);
        }
        // Free the now-orphaned container node
        self.nodes[node_id.0] = None;
        self.free_list.push(node_id);
    }
}
```

在 `cleanup_empty_container` 和 `move_window` 的清理步骤中调用此方法。

**注意：** 这一项需要仔细考虑——Sway 实际上**不总是**展平单子节点容器。如果用户显式 `split` 创建了容器，该容器即使只有一个子节点也应保留。只有通过窗口移除/移动导致的单子节点才应展平。需要进一步验证 Sway 的精确行为。建议标记为 Phase 2。

---

### INC-9: switch_workspace 是全局切换 [LOW]

**Sway 行为：**
`workspace <name>` 只切换当前焦点所在 output 的 workspace。其他 output 保持不变。如果目标 workspace 在另一个 output，焦点移到那个 output。

**当前 rway 行为：**
`switch_workspace()` (tree.rs:415) 切换目标 workspace 所在 output 的所有 workspace 可见性，但不处理焦点 output 的概念。

**修复方案：**
需要在 Tree 中跟踪 "当前焦点 output"，并在切换 workspace 时只影响相关 output。这涉及更多的焦点管理重构，建议标记为 Phase 2。

---

## 修改优先级排序

### Phase 1 — 核心平铺兼容（小改动，高收益）

| 编号 | 修改 | 代码量 | 风险 |
|------|------|--------|------|
| INC-1 | Tabbed/Stacked focus 方向匹配 | ~2 行 | 极低 |
| INC-2 | Tabbed/Stacked move 方向匹配 | ~2 行 | 极低 |
| INC-5 | Tabbed/Stacked 全子节点布局 | ~3 行 | 低 |
| INC-6 | remove_window 焦点同步 | ~3 行 | 低 |
| INC-7 | insert_window 焦点同步 | ~1 行 | 极低 |

**Phase 1 总量：** ~11 行代码修改

### Phase 2 — 增强兼容（中等改动）

| 编号 | 修改 | 代码量 | 风险 |
|------|------|--------|------|
| INC-3 | focus_wrapping yes/no | ~15 行 | 低 |
| INC-4 | resize 递归向上查找 | ~20 行 | 中（需重构） |

**Phase 2 总量：** ~35 行代码修改

### Phase 3 — 进一步优化（需要更多设计）

| 编号 | 修改 | 代码量 | 风险 |
|------|------|--------|------|
| INC-8 | 单子节点容器展平 | ~25 行 | 中（需验证 Sway 精确行为） |
| INC-9 | 多 output workspace 切换 | ~40 行 | 中（焦点管理重构） |

---

## 架构不变性声明

以下架构特性在所有修改中**保持不变**：

1. **Arena 分配**：`Vec<Option<Node>>` + `free_list` 不变
2. **NodeData 枚举**：5 种变体不变（可能增加字段，不增加变体）
3. **impl Tree 方法风格**：所有操作仍是 `impl Tree` 方法
4. **LayoutInfo 枚举**：借用安全的布局信息提取模式不变
5. **错误类型**：`TilingError` 的 9 个变体不变
6. **模块结构**：tree.rs 为核心，delegation wrapper 模块保持

**唯一新增结构：**
- `FocusWrapping` 枚举（Phase 2，INC-3）
- Tree 新增 `focus_wrapping: FocusWrapping` 字段（Phase 2）

---

## 测试策略

每个 INC 修复需要对应的测试用例，建议添加到 `rway-tiling/tests/sway_compat.rs`：

| INC | 需要的测试 |
|-----|----------|
| INC-1 | `focus_left_right_in_tabbed`, `focus_up_down_in_stacked` |
| INC-2 | `move_left_right_in_tabbed`, `move_up_down_in_stacked` |
| INC-3 | `focus_wrapping_yes_cycles`, `focus_wrapping_no_stops` |
| INC-4 | `resize_finds_ancestor_with_matching_axis` |
| INC-5 | `tabbed_all_children_get_geometry` |
| INC-6 | `remove_window_syncs_focus_path` |
| INC-7 | `insert_window_syncs_focus_path` |
| INC-8 | `single_child_container_flattened_after_move` |
