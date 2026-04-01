# rway 架构文档

## Crate 依赖图

```
                    ┌─────────────┐
                    │    rway     │  Layer 2 (smithay 合成器)
                    └──┬──┬──┬───┘
                       │  │  │
          ┌────────────┘  │  └────────────┐
          ▼               ▼               ▼
   ┌────────────┐  ┌────────────┐  ┌────────────┐
   │ rway-tiling│  │ rway-config│  │  rway-ipc  │  Layer 0-1
   │  (Layer 0) │  │  (Layer 0) │  │  (Layer 1) │
   └────────────┘  └────────────┘  └──┬──┬──────┘
                                      │  │
                               calloop┘  └serde/serde_json

   ┌──────────────────────────────────────────────┐
   │              rway-harness                     │  Layer 3 (测试框架)
   │   依赖: rway-tiling, rway-config, rway-ipc   │
   │   不依赖 rway（可独立编译和运行）               │
   └──────────────────────────────────────────────┘
```

## 数据流

### 配置加载

```
~/.config/sway/config
        │
        ▼
rway_config::parse_file()          # rway-config/src/parser.rs
        │
        ▼
    Config struct                   # rway-config/src/types.rs
        │
        ▼
RwayState.config                   # rway/src/state.rs
        │
        ├─→ keybindings[]          # 快捷键列表
        ├─→ outputs[]              # 输出配置
        ├─→ gaps                   # 间距配置
        └─→ window_rules[]         # 窗口规则
```

### 窗口生命周期

```
客户端创建 surface
        │
        ▼
  XDG Shell 协议处理               # rway/src/handlers/ (Smithay 协议委托)
        │
        ▼
  new_toplevel 回调
        │
        ▼
  insert_window(&mut Tree, id)     # rway-tiling/src/commands.rs
        │  └─ 找到聚焦工作区 → 创建/复用 Container → 添加 Window 节点
        ▼
  compute_layout(tree, root, rect, gaps)   # rway-tiling/src/layout.rs
        │  └─ 自顶向下递归计算每个节点的 geometry
        ▼
  Space::map_element()             # Smithay Space (2D 窗口平面)
        │
        ▼
  渲染到 Output
```

**窗口状态转换**:

```
新建(new) ──→ 映射(mapped) ──→ 获焦(focused) ──→ 关闭(destroyed)
                  │                    │
                  │                    ├──→ 移动/调整大小
                  │                    ├──→ 浮动(floating)
                  │                    └──→ 全屏(fullscreen)
                  │
                  └──→ 应用窗口规则(for_window)
```

### IPC 消息流

```
swaymsg / waybar
        │
        ▼
  Unix domain socket               # $XDG_RUNTIME_DIR/rway-ipc.$WAYLAND_DISPLAY.sock
        │
        ▼
  IpcServer::try_accept()          # rway-ipc/src/server.rs
        │  (calloop 定时器每 50ms 轮询)
        ▼
  decode_header() → (len, type)    # rway-ipc/src/protocol.rs
        │
        ▼
  dispatch_ipc_message()           # rway/src/ipc.rs
        │
        ├─ type=0  → handle_run_command()     # RUN_COMMAND
        ├─ type=1  → handle_get_workspaces()  # GET_WORKSPACES
        ├─ type=3  → handle_get_outputs()     # GET_OUTPUTS
        ├─ type=4  → handle_get_tree()        # GET_TREE
        ├─ type=7  → handle_get_version()     # GET_VERSION
        └─ 其他   → {"success": false}
        │
        ▼
  encode_message(type, json_bytes) # rway-ipc/src/protocol.rs
        │
        ▼
  write_all → 客户端
```

### 按键处理

```
键盘事件 (Smithay input)
        │
        ▼
  find_matching_binding()          # rway/src/input/keybindings.rs
        │  └─ 遍历 config.keybindings, 匹配修饰键+按键名
        ▼
  execute_action(state, &Action)   # rway/src/input/keybindings.rs
        │
        ├─ Action::Exec(cmd)       → std::process::Command::spawn()
        ├─ Action::Focus(dir)      → tiling::commands::move_focus()
        ├─ Action::Workspace(name) → tiling::workspace::switch_workspace()
        ├─ Action::Split(dir)      → tiling::commands::split()
        ├─ Action::Layout(type)    → tiling::commands::split()
        ├─ Action::Kill            → 关闭聚焦窗口
        ├─ Action::ToggleFloating  → tiling::commands::toggle_floating()
        ├─ Action::Fullscreen      → tiling::commands::toggle_fullscreen()
        ├─ Action::Reload          → 重新加载配置
        └─ Action::Exit            → loop_signal.stop()
```

## 平铺树模型

### 节点层次

```
Root (NodeData::Root)
 └─ Output[] (NodeData::Output { name, geometry })
     └─ Workspace[] (NodeData::Workspace { name, output, is_visible })
         ├─ Container (NodeData::Container { layout, sizes, focused_child })
         │   ├─ Window (NodeData::Window { window_id, floating, fullscreen, geometry })
         │   └─ Window
         └─ Container
             ├─ Container (嵌套)
             │   ├─ Window
             │   └─ Window
             └─ Window
```

### 与 Sway 容器树的对应关系

| Sway 概念 | i3 IPC type 字段 | rway 类型 | 文件 |
|-----------|------------------|----------|------|
| 根节点 | `"root"` | `NodeData::Root` | `rway-tiling/src/tree.rs` |
| 输出/显示器 | `"output"` | `NodeData::Output` | `rway-tiling/src/tree.rs` |
| 工作区 | `"workspace"` | `NodeData::Workspace` | `rway-tiling/src/tree.rs` |
| 分割容器 | `"con"` | `NodeData::Container` | `rway-tiling/src/tree.rs` |
| 窗口 | `"con"` (叶子) | `NodeData::Window` | `rway-tiling/src/tree.rs` |
| 浮动窗口 | `"floating_con"` | `NodeData::Window { floating: true }` | `rway-tiling/src/tree.rs` |

### 布局类型

| Sway 布局名 | rway 枚举值 | 行为 |
|-------------|------------|------|
| `splith` | `Layout::SplitH` | 水平分割，按 `sizes[]` 比例分配宽度 |
| `splitv` | `Layout::SplitV` | 垂直分割，按 `sizes[]` 比例分配高度 |
| `tabbed` | `Layout::Tabbed` | 共享区域，仅 `focused_child` 可见 |
| `stacked` | `Layout::Stacked` | 共享区域，仅 `focused_child` 可见 |

### 布局计算（`compute_layout`）

`compute_layout(tree, node_id, available, gaps)` 自顶向下递归：

1. **Root** → 将 `available` 传递给每个 Output 子节点
2. **Output** → 使用自身 `geometry` 作为可用区域
3. **Workspace** → 应用 `gaps.outer` 收缩四边
4. **Container/SplitH** → 按 `sizes[]` 比例水平分配宽度
5. **Container/SplitV** → 按 `sizes[]` 比例垂直分配高度
6. **Container/Tabbed|Stacked** → 仅聚焦子节点获得完整区域
7. **Window** → 应用 `gaps.inner / 2` 收缩四边（浮动窗口跳过）

## 关键类型映射表

| Sway 概念 | rway 类型 | 文件 |
|-----------|----------|------|
| Container | `NodeData::Container` | `rway-tiling/src/tree.rs` |
| Window | `NodeData::Window` | `rway-tiling/src/tree.rs` |
| Workspace | `NodeData::Workspace` | `rway-tiling/src/tree.rs` |
| Layout | `Layout` 枚举 | `rway-tiling/src/tree.rs` |
| Node ID | `NodeId(usize)` | `rway-tiling/src/tree.rs` |
| Rect | `Rect { x, y, width, height }` | `rway-tiling/src/tree.rs` |
| Gaps | `GapsConfig { inner, outer }` | `rway-tiling/src/tree.rs` |
| Config | `Config` struct | `rway-config/src/types.rs` |
| Keybinding | `Keybinding { modifiers, key, action }` | `rway-config/src/types.rs` |
| Action | `Action` 枚举 | `rway-config/src/types.rs` |
| IPC 命令结果 | `CommandResult` | `rway-ipc/src/commands.rs` |
| IPC 树节点 | `TreeNode` | `rway-ipc/src/commands.rs` |
| IPC 工作区 | `WorkspaceInfo` | `rway-ipc/src/commands.rs` |
| IPC 输出 | `OutputInfo` | `rway-ipc/src/commands.rs` |
| IPC 消息类型 | `MessageType` 枚举 | `rway-ipc/src/protocol.rs` |
| IPC 事件类型 | `EventType` 枚举 | `rway-ipc/src/protocol.rs` |
| IPC 订阅 | `SubscriptionType` 枚举 | `rway-ipc/src/events.rs` |
| 核心状态 | `RwayState` | `rway/src/state.rs` |
