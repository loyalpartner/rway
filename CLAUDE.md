# rway — Sway 兼容 Wayland 合成器

rway 是基于 [Smithay](https://github.com/Smithay/smithay) 框架构建的 Wayland 合成器，目标是让用户可以无缝从 Sway 迁移到 rway，复用现有的 Sway 配置文件、快捷键、IPC 工具链（swaymsg、waybar 等）。

## Language

- All code comments and log messages (tracing) must be in **English**.
- Git commit messages in English.
- Respond to the user in Chinese (简体中文).

## 架构概览

### Crate 职责

| Crate | 描述 |
|-------|------|
| `rway` | 合成器主程序：Smithay 集成、后端（winit/udev）、XWayland、渲染、输入处理 |
| `rway-tiling` | 纯 Rust 平铺引擎：Arena N 叉树、布局算法、窗口/工作区管理 |
| `rway-config` | Sway 配置文件解析器：变量替换、按键绑定、输出/输入配置、窗口规则 |
| `rway-ipc` | Sway IPC 协议实现：i3-ipc 二进制协议编解码、Unix socket 服务端 |
| `rway-harness` | 兼容性测试框架：8 分类测试注册/运行/报告生成 |

### 依赖分层

```
Layer 0: rway-tiling          (零外部依赖)
         rway-config          (thiserror)
Layer 1: rway-ipc             (calloop, serde, serde_json, thiserror)
Layer 2: rway                 (smithay + Layer 0-1)
Layer 3: rway-harness         (Layer 0-1, 不依赖 rway)
```

**规则**: Layer N 只能依赖 Layer < N。rway-tiling 和 rway-config 之间不存在依赖关系。

## 编码规范

- **不可变优先**: 返回新对象，不修改已有对象。Tree 操作通过 `&mut Tree` 的专用方法进行。
- **错误处理**: 不使用 `unwrap()` / `expect()`（测试代码除外），用 `Result` 或 `Option` 处理错误路径。
- **函数体积**: 单个函数 < 50 行，单个文件 < 800 行。
- **测试先行**: 新功能必须先写测试（RED → GREEN → REFACTOR）。
- **零循环依赖**: 严格遵守分层规则，`cargo build --workspace` 通过即可验证。

## Sway 兼容性工作流

每次实现新功能前：

1. **读规格**: 打开 `docs/sway-spec.md`，找到对应功能的 Sway 行为描述
2. **写测试 (RED)**: 在 `rway-harness/src/categories/` 对应分类中添加兼容性测试
3. **实现功能 (GREEN)**: 在对应 crate 中实现，使测试通过
4. **过 gates**: 依次运行以下命令，全部通过才可提交

### Gates

```bash
cargo clippy --workspace -- -D warnings   # lint 检查
cargo test --workspace                      # 全部测试
cargo run -p rway-harness -- report         # 兼容性报告（确认不回归）
```

## 常用命令

```bash
cargo build --workspace                     # 编译全部 crate
cargo test --workspace                      # 运行全部测试
cargo test -p rway-tiling                   # 仅平铺引擎测试
cargo test -p rway-harness                  # 兼容性测试（作为单元测试运行）
cargo run -p rway-harness -- report         # 生成兼容性报告
cargo run -p rway-harness -- report window  # 仅 Window 分类报告
cargo clippy --workspace -- -D warnings     # lint
```

## 核心入口文件

| 文件 | 说明 |
|------|------|
| `rway/src/state.rs` | `RwayState` — 合成器核心状态结构体 |
| `rway/src/main.rs` | 启动流程、后端选择（winit/udev） |
| `rway/src/input/keybindings.rs` | 快捷键匹配与动作执行（`execute_action`） |
| `rway/src/ipc.rs` | IPC 消息分发（`dispatch_ipc_message`） |
| `rway-tiling/src/tree.rs` | Arena N 叉树、`NodeData` 枚举 |
| `rway-tiling/src/commands.rs` | 高级命令：insert/remove/move_focus/split |
| `rway-tiling/src/layout.rs` | 布局算法：`compute_layout()` |
| `rway-config/src/types.rs` | 配置类型定义：`Config`, `Action`, `Keybinding` |
| `rway-config/src/parser.rs` | Sway 配置解析器 |
| `rway-ipc/src/protocol.rs` | i3-ipc 二进制协议编解码 |
| `docs/sway-spec.md` | Sway 功能规格与兼容性分析（796 行） |
