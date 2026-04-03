# rway-tiling 实现分析

> 基于 rway-tiling crate 源码的深度分析，为后续与 Sway 行为对比做准备。
> 文件：`tree.rs` (2686 行), `container.rs`, `layout.rs`, `workspace.rs`, `commands.rs`, `error.rs`

---

## 1. 数据结构概览

### 1.1 Node 与 NodeData

```
Tree
├── nodes: Vec<Option<Node>>    // Arena 存储，slot 可复用
├── free_list: Vec<NodeId>      // 已释放的 slot 索引
├── root: NodeId                // 永远是 NodeId(0)
└── focus_node: Option<NodeId>  // focus_parent/focus_child 用的层级焦点
```

**Node** 结构：
- `parent: Option<NodeId>` — 父节点（Root 为 None）
- `children: Vec<NodeId>` — 有序子节点列表
- `data: NodeData` — 业务数据枚举

**NodeData 变体：**

| 变体 | 字段 | 角色 |
|------|------|------|
| `Root` | 无 | 虚拟根节点，唯一 |
| `Output` | `name`, `geometry: Rect` | 物理显示器 |
| `Workspace` | `name`, `output: NodeId`, `is_visible` | 工作区 |
| `Container` | `layout: Layout`, `sizes: Vec<f64>`, `focused_child: usize` | 分割/标签/堆叠容器 |
| `Window` | `window_id: u64`, `floating`, `fullscreen`, `fullscreen_global`, `sticky`, `marks`, `geometry`, `saved_geometry` | 实际窗口 |

**Layout 枚举：** `SplitH`, `SplitV`, `Tabbed`, `Stacked`

### 1.2 树的层级结构

```
Root
└── Output("eDP-1", geometry)
    ├── Workspace("1", visible=true)
    │   └── Container(SplitH, sizes=[1.0, 1.0], focused=1)
    │       ├── Window(id=1)
    │       └── Container(SplitV, sizes=[1.0, 1.0], focused=0)
    │           ├── Window(id=2)
    │           └── Window(id=3)
    └── Workspace("2", visible=false)
```

### 1.3 Arena 分配机制

- `add_node()`: 优先从 `free_list` 复用 slot，否则 `push` 新位置
- `remove_node()`: 迭代收集所有后代，从父节点的 children 中移除，slot 置 None 并加入 `free_list`
- 压力测试验证了 1000 节点的分配/释放和跨多轮的 slot 复用

### 1.4 焦点模型

焦点通过两个机制追踪：

1. **Container.focused_child** — 每个容器记录当前聚焦的子节点索引，形成从 workspace 到叶子窗口的焦点路径
2. **Tree.focus_node** — 用于 `focus_parent`/`focus_child` 命令的层级焦点。`None` 表示焦点在叶子窗口上

`focus_window(window_id)` 从窗口节点向上遍历到 workspace，更新沿途每个 Container 的 `focused_child`，保持焦点路径一致。

---

## 2. insert_window 分析

### 2.1 入口

```rust
pub fn insert_window(&mut self, window_id: u64) -> NodeId
    → insert_window_with_layout(window_id, Layout::SplitH)
        → insert_window_into(ws_id, window_id, layout)
```

默认布局为 `SplitH`（水平分割）。

### 2.2 核心逻辑 (insert_window_into)

**分支 1 — 空工作区/容器（children 为空）：**
- 如果 parent 是 Workspace：创建一个 Container(layout=split_layout, sizes=[1.0], focused=0)，再将窗口添加到容器中
- 否则：直接添加为子节点

**分支 2 — 聚焦子节点是窗口：**
- 如果 parent 是 Container：在 focused_idx + 1 位置插入新窗口作为兄弟节点（使用 `insert_child_at`），同时在 sizes 中插入 1.0，更新 focused_child 指向新窗口
- 如果 parent 是 Workspace（直接包含窗口）：调用 `wrap_workspace_children_in_container()` 将所有子节点包裹进新容器

**分支 3 — 聚焦子节点是容器：** 递归进入该容器

### 2.3 Sway 兼容性评估

**正确行为：**
- 三个窗口在同方向插入后成为平级兄弟（不嵌套）— 测试 `three_windows_same_direction_are_siblings` 通过
- 新窗口插入在 focused 位置之后（非末尾追加）— 测试 `insert_after_focused_not_at_end` 通过
- 新窗口获得 1.0 的 size 权重，与既有窗口等比分配

**潜在问题：**
- `split_layout` 参数仅在首个容器创建时使用。当容器已存在时，新窗口继承父容器的布局方向，这是正确的 Sway 行为
- `wrap_workspace_children_in_container` 路径（workspace 直接包含窗口的情况）在正常流程中不应触发，因为第一个窗口插入时就会创建容器。但如果外部代码直接操作树结构可能触发
- **insert_window 不调用 focus_window**：新窗口被设为 Container 的 focused_child，但不更新上层的焦点路径。这在大多数情况下工作正常（因为递归插入会进入最深层），但可能在复杂嵌套场景下导致焦点路径不一致

---

## 3. move_focus 分析

### 3.1 入口

```rust
pub fn move_focus(&mut self, direction: Direction) -> bool
    → move_focus_in(ws_id, direction)  // 递归
    → 成功后调用 focus_window() 重新同步焦点路径
```

### 3.2 核心逻辑 (move_focus_in)

**递归下降策略：**

1. 对 Container 节点：先递归进入 focused_child 子树尝试移动
2. 如果子树内移动失败，检查当前容器的轴是否匹配方向：
   - SplitH 匹配 Left/Right
   - SplitV 匹配 Up/Down
3. 轴匹配时，计算 new_focus = focused ± 1，检查边界
4. 更新 `focused_child` 为新索引

**对 Workspace 节点：** 遍历所有子节点，递归尝试移动

### 3.3 Sway 兼容性评估

**正确行为：**
- 水平容器中 Left/Right 移动焦点 — 测试通过
- 垂直容器中 Up/Down 移动焦点 — 测试通过
- 移动后调用 `focus_window()` 同步焦点路径

**已识别问题：**

1. **跨容器焦点移动受限**：`move_focus_in` 是纯递归下降，从 workspace 到叶子。当焦点到达子容器边界时，算法尝试在父容器中移动。但如果父容器轴不匹配，焦点无法穿越。例如：
   ```
   SplitH [ SplitV[A*, B], C ]
   ```
   焦点在 A，按 Right：`move_focus_in` 递归进入 SplitV，方向 Right 与 SplitV 不匹配，递归返回 false。然后在 SplitH 层级，focused_child=0(SplitV)，尝试 focused+1=1(C)，成功。**这个场景是正确的。**

2. **Tabbed/Stacked 容器不响应任何方向**：代码只匹配 `SplitH + Left/Right` 和 `SplitV + Up/Down`。Tabbed 和 Stacked 布局的焦点移动完全被忽略。Sway 中 Tabbed 容器响应 Left/Right，Stacked 容器响应 Up/Down。**这是一个差异。**

3. **focus_wrapping 未实现**：到达边界时直接返回 false，不支持环绕。Sway 的 `focus_wrapping yes` 会环绕到另一端。

4. **跨 Output 焦点移动未实现**：只在单个 workspace 内移动。Sway 支持 `focus output left/right` 跨显示器。

---

## 4. move_window 分析

### 4.1 入口

```rust
pub fn move_window(&mut self, direction: Direction) -> bool
```

### 4.2 核心逻辑 — 向上遍历策略

1. **找到聚焦的叶子窗口节点** (`find_focused_leaf_node`)
2. **从叶子向上遍历**，寻找轴匹配的祖先容器：
   - SplitH 匹配 Left/Right
   - SplitV 匹配 Up/Down
3. 找到匹配容器后，计算 `current` 在该容器中的索引和边界状态

**三种情况：**

**Case A — 同级交换（current == leaf, 非边界）：**
- 与邻居交换 children 和 sizes 中的位置
- 调用 `focus_window()` 重新同步

**Case B — 跨容器移动（current != leaf, 非边界）：**
- 从原父容器中提取叶子窗口（移除 children 引用 + sizes 条目）
- 在祖先容器中目标位置插入叶子（插入 children + sizes 条目 1.0）
- 更新叶子的 parent 指针
- 清理空容器 (`cleanup_empty_container`)
- 重新同步焦点

**Case C — 到达边界：** 继续向上遍历寻找更高层级的匹配容器。如果到达 Workspace 仍未找到，返回 false。

### 4.3 Sway 兼容性评估

**正确行为：**
- 同级交换工作正常 — 测试 `move_swaps_siblings_same_container` 通过
- 2x2 网格中垂直交换工作正常 — 测试 `move_down_in_2x2_swaps_within_column` 通过
- 跨容器提取工作正常 — 测试 `move_right_in_2x2_extracts_window` 通过
- 边界返回 false — 测试 `move_at_boundary_returns_false` 通过

**已识别问题：**

1. **边界处不创建新容器**：Sway 在移动到容器边界之外时，会将窗口从当前容器提取出来并插入到父容器（或在必要时创建新容器）。当前实现在到达 workspace 级别时直接返回 false。Sway 的实际行为更复杂——当移动到最外层边界时，可能会尝试移动到相邻 output。

2. **Tabbed/Stacked 容器中的移动**：move_window 只匹配 SplitH/SplitV 的轴，Tabbed 和 Stacked 布局完全不响应移动。Sway 中 Tabbed 容器允许 Left/Right 移动，Stacked 容器允许 Up/Down 移动。

3. **move_window 后焦点应留在被移动的窗口上**：当前通过 `focus_window(window_id)` 实现，这是正确的。

4. **sizes 不归一化**：跨容器移动时在目标容器插入 1.0 的 size，但不归一化。如果原有 sizes 总和已经归一化为 1.0，新增的 1.0 会使总和变为 2.0。虽然 layout 计算时会按比例分配，但这导致 sizes 语义不一致。

---

## 5. resize_container 分析

### 5.1 逻辑

```rust
pub fn resize_container(&mut self, node_id: NodeId, axis: ResizeAxis, delta_ppt: f64) -> bool
```

1. 找到 node_id 的父容器
2. 检查父容器布局是否匹配 axis（Width → SplitH, Height → SplitV）
3. 确定 sibling_index：优先取 my_index + 1，否则 my_index - 1
4. 调整 sizes：`sizes[my] += delta/100`, `sizes[sibling] -= delta/100`
5. 最小值钳制：任一 size < 0.05 时钳制并从对方补偿

### 5.2 Sway 兼容性评估

**正确行为：**
- 基本的 resize 操作在匹配轴上工作正常
- 最小值保护（0.05 = 5%）防止窗口缩到不可见

**已识别问题：**

1. **只调整相邻兄弟**：Sway 的 resize 调整的是当前节点和它的 **下一个兄弟**（或上一个，如果是最后一个）。当前实现也是这样的，这是正确的。

2. **不支持递归 resize**：如果 node_id 的直接父容器轴不匹配，直接返回 false。Sway 会向上查找轴匹配的祖先容器来执行 resize。例如在嵌套布局中想调整宽度，但直接父容器是 SplitV，应该向上找到 SplitH 祖先。

3. **delta_ppt 单位**：参数名暗示 "percentage points"，实际除以 100 转为小数。但 sizes 的绝对值不一定归一化为 1.0（初始值都是 1.0），所以 "10 ppt" 的实际效果取决于 sizes 总和。

4. **钳制逻辑可能产生负值**：如果两个窗口的 size 都接近 0.05，双重钳制可能导致一个 size 变为负值。不过在实际使用中不太可能触发。

---

## 6. compute_layout 分析

### 6.1 核心逻辑

递归自顶向下的布局计算。使用 `LayoutInfo` 枚举避免借用冲突。

**各节点类型的处理：**

| 节点类型 | 行为 |
|----------|------|
| Root | PassThrough — 传递 available rect 给所有子节点 |
| Output | PassThrough — 使用 output 自身的 geometry |
| Workspace | PassThrough — 应用 outer gap 收缩后传递 |
| Container(SplitH) | 按 sizes 比例水平分割 available rect |
| Container(SplitV) | 按 sizes 比例垂直分割 available rect |
| Container(Tabbed/Stacked) | 只对 focused_child 应用 available rect（全尺寸） |
| Window(tiling) | 应用 inner gap/2 收缩，写入 geometry |
| Window(floating) | 不修改 geometry |

### 6.2 Split 布局算法 (layout_split_h/v)

```
total = sum(sizes)
for each child i:
    ratio = sizes[i] / total      // 如果 total > 0 且 i < sizes.len
    width = available.width * ratio  // 四舍五入
    // 最后一个 child 取剩余宽度（消除舍入误差）
    child_rect = Rect { x: x_offset, y: available.y, width, height: available.height }
    recursive compute_layout(child, child_rect)
    x_offset += width
```

### 6.3 Sway 兼容性评估

**正确行为：**
- 等比分配工作正常 — 测试 `three_siblings_get_equal_layout` 通过
- 2x2 网格布局正确 — 测试 `build_2x2_grid` 通过
- 最后一个子节点使用剩余空间消除舍入误差

**已识别问题：**

1. **Gap 模型简化**：Sway 的 gap 模型更复杂，区分 inner/outer/top/right/bottom/left。当前实现只有 inner 和 outer 两个值。Workspace 应用 outer gap，Window 应用 inner/2 gap。Sway 的 inner gap 是相邻窗口之间的间距（每侧 inner/2），outer gap 是窗口与屏幕边缘的间距。当前实现基本正确但缺少精细控制。

2. **Tabbed/Stacked 只渲染聚焦窗口**：代码只对 `focused_child` 调用 `compute_layout`。Sway 中 Tabbed/Stacked 容器会为所有子窗口计算 geometry（用于标题栏渲染），但只显示聚焦的那个。当前实现会导致非聚焦窗口的 geometry 为 (0,0,0,0)。

3. **sizes 与 children 数量不匹配的防护**：当 `i >= sizes.len()` 时回退到 `1.0/n` 等分。这是合理的防御性处理。

4. **Output geometry 硬编码传递**：Output 节点直接使用自己的 geometry，忽略传入的 available rect。这在单 output 场景下正确，但多 output 场景需要确保每个 output 有正确的 geometry。

---

## 7. 其他重要方法分析

### 7.1 split (immediate wrapping)

```rust
pub fn split(&mut self, layout: Layout)
```

- 找到聚焦叶子窗口，检查父容器是否已有匹配布局且只有一个子节点（no-op 条件）
- 调用 `wrap_leaf_in_container()` 在原位创建新容器，将窗口重新挂载为其子节点
- `wrap_leaf_in_container` 保持窗口在父容器中的原始位置索引

**正确行为：** 测试 `wrap_container_preserves_position` 和 `split_then_insert_creates_nesting` 通过。

### 7.2 remove_window

- 找到窗口节点，记录其在父容器中的索引
- 调用 `remove_node()` 移除窗口
- 从父容器的 sizes 中移除对应条目
- 调用 `cleanup_empty_container()` 递归清理空容器

**正确行为：** 测试 `remove_all_windows_cleans_containers` 和 `remove_from_2x2_preserves_structure` 通过。

**潜在问题：** 移除窗口后不更新焦点。如果移除的是当前聚焦窗口，`focused_child` 指针会被 `remove_container_size` 调整（钳制到 `sizes.len()-1`），但不会触发 `focus_window()` 同步。这可能导致上层容器的焦点路径不一致。

### 7.3 focus_parent / focus_child

使用 `Tree.focus_node` 字段跟踪层级焦点。`focus_parent` 向上移动（排除 Root），`focus_child` 向下进入（到叶子时清除为 None）。

**问题：** `focus_node` 与 `focused_child` 路径是独立的两套焦点系统，可能产生不一致。

### 7.4 switch_workspace

遍历目标 workspace 所在 output 的所有 workspace，设置 `is_visible`（目标为 true，其余为 false）。

**问题：** 不支持多 output 场景下只切换某个 output 的 workspace。当前是全局切换。

### 7.5 move_to_workspace

先 `remove_window`，再 `insert_window_into(target_ws)`。简单直接。

**问题：** 使用 `focused_window_id()` 获取窗口，但这返回的是 `u64` (window_id)，而不是 NodeId。随后 `remove_window` 和 `insert_window_into` 都需要重新查找窗口位置，有性能开销但逻辑正确。

---

## 8. 已识别的问题清单

### 8.1 关键问题（影响 Sway 兼容性）

| # | 问题 | 位置 | 严重度 |
|---|------|------|--------|
| 1 | **Tabbed/Stacked 容器不响应焦点移动** | `move_focus_in()` | HIGH — Sway 中 Tabbed 响应 Left/Right，Stacked 响应 Up/Down |
| 2 | **Tabbed/Stacked 容器不响应窗口移动** | `move_window()` | HIGH — 同上 |
| 3 | **focus_wrapping 未实现** | `move_focus_in()` | MEDIUM — Sway 默认 `focus_wrapping yes`，到达边界时环绕 |
| 4 | **resize 不递归查找匹配轴的祖先** | `resize_container()` | MEDIUM — 嵌套布局中 resize 可能无法工作 |
| 5 | **Tabbed/Stacked 只计算聚焦窗口布局** | `compute_layout()` | MEDIUM — 非聚焦窗口 geometry 为零 |
| 6 | **remove_window 后不同步焦点路径** | `remove_window()` | MEDIUM — 可能导致焦点状态不一致 |

### 8.2 次要问题（功能限制）

| # | 问题 | 位置 | 严重度 |
|---|------|------|--------|
| 7 | 跨 Output 焦点/移动未实现 | `move_focus`, `move_window` | LOW — P1 功能 |
| 8 | Gap 模型简化（只有 inner/outer） | `compute_layout()` | LOW — 缺少精细控制 |
| 9 | switch_workspace 是全局切换 | `switch_workspace()` | LOW — 多 output 不友好 |
| 10 | sizes 不自动归一化 | 跨容器移动时 | LOW — 语义不一致但不影响布局 |
| 11 | insert_window 不同步上层焦点路径 | `insert_window_into()` | LOW — 大多场景下焦点正确 |
| 12 | 双焦点系统 (focus_node vs focused_child) 可能不一致 | `focus_parent/child` | LOW — 边界情况 |

### 8.3 架构优点

- **Arena 分配**：高效的内存管理，slot 复用避免碎片
- **LayoutInfo 枚举**：巧妙地解决了 Rust 借用检查器的问题，避免 `data.clone()`
- **Smithay 风格的 impl 方法**：所有操作是 `impl Tree` 方法，干净的 API
- **完善的错误类型**：9 个 `TilingError` 变体覆盖了主要错误场景
- **测试覆盖良好**：46+ 单元测试 + 18 个 Sway 兼容性集成测试，全部通过

---

## 9. 方法清单摘要

### 公开 API（~40 个方法）

| 分类 | 方法 |
|------|------|
| **基础树操作** | `new()`, `root()`, `add_node()`, `remove_node()`, `get()`, `get_mut()`, `children()`, `parent()`, `node_count()` |
| **容器** | `add_container_size()`, `remove_container_size()`, `set_layout()`, `focused_child()`, `set_focused_child()` |
| **工作区** | `add_output()`, `add_workspace()`, `focused_workspace()`, `switch_workspace()`, `workspaces()`, `span_workspace()`, `rename_workspace()` |
| **窗口插入/删除** | `insert_window()`, `insert_window_with_layout()`, `remove_window()` |
| **焦点** | `move_focus()`, `focus_window()`, `focused_window_id()`, `focus_parent()`, `focus_child()`, `focus_next_sibling()`, `focus_prev_sibling()` |
| **窗口移动** | `move_window()`, `move_to_workspace()`, `swap_containers()` |
| **布局** | `split()`, `layout_toggle()`, `compute_layout()`, `window_geometries()` |
| **窗口属性** | `toggle_floating()`, `set_floating()`, `set_fullscreen()`, `toggle_fullscreen()`, `set_fullscreen_global()`, `toggle_fullscreen_global()`, `set_sticky()`, `toggle_sticky()`, `add_mark()`, `remove_mark()`, `get_marks()`, `find_node_by_window_id()` |
| **Resize** | `resize_container()` |

### 委托包装模块

`commands.rs`, `layout.rs`, `workspace.rs`, `container.rs` 提供 free function 包装，向后兼容旧的模块级 API。所有实际逻辑已迁移到 `impl Tree`。
