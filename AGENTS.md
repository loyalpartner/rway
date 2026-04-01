# Agent 行为规范

## 角色定义

| 角色 | 权限 | 职责 |
|------|------|------|
| `context-writer` | 读写 `docs/` 和项目根文档 | 维护 Context 层：CLAUDE.md、AGENTS.md、architecture.md、sway-spec.md |
| `spec-tester` | 读写 `rway-harness/` | 扩充兼容性测试用例，确保每个 spec 条目都有对应测试 |
| `rust-dev` | 读写所有 `src/` | 实现功能、修复 bug、性能优化 |
| `sweep-agent` | 只读 + 报告 | 一致性检查：spec ↔ 测试 ↔ 实现三方对齐审计 |

## 通用规则

### 必读文档

修改任何代码前，必须先读 `docs/sway-spec.md` 中对应功能的规格描述，确保实现行为与 Sway 一致。

### 功能实现 Checklist

实现新功能时，按顺序完成以下步骤：

1. [ ] 读 `docs/sway-spec.md` 对应条目，理解 Sway 的精确行为
2. [ ] 确认 `rway-harness/src/categories/` 中有对应测试（没有则先写测试）
3. [ ] 在对应 crate 中实现功能
4. [ ] `cargo clippy --workspace -- -D warnings` 通过
5. [ ] `cargo test --workspace` 通过
6. [ ] `cargo run -p rway-harness -- report` 兼容性不回归
7. [ ] 更新 `docs/sway-spec.md` 中的状态标记（`[MISSING]` → `[IMPLEMENTED]`，或 `[PARTIAL]` → `[IMPLEMENTED]`）

### 禁止行为

- **不得使用 `unwrap()` / `expect()`**（测试代码除外）— 用 `Result` 或 `Option` 链式处理
- **不得跳过测试** — 新功能必须有测试覆盖，不得 `#[ignore]` 已有测试
- **不得引入循环依赖** — 严格遵守分层规则：Layer N 只依赖 Layer < N
- **不得硬编码值** — 使用常量或配置，不在代码中写死数字/字符串
- **不得修改已通过测试的公开 API 签名** — 除非有充分理由并更新所有调用方

### 依赖分层规则

```
Layer 0: rway-tiling, rway-config  (零/极少外部依赖)
Layer 1: rway-ipc                   (serde, calloop)
Layer 2: rway                       (smithay + Layer 0-1)
Layer 3: rway-harness               (Layer 0-1, 不依赖 rway)
```

新增依赖前检查：
- Layer 0 crate 不得添加非必要外部依赖
- 所有新依赖必须在 PR 中说明理由

### Harness 测试分类

| 分类 | 文件 | 覆盖范围 |
|------|------|----------|
| Config | `rway-harness/src/categories/config.rs` | 配置解析：set, bindsym, exec, output, gaps 等 |
| IPC | `rway-harness/src/categories/ipc.rs` | IPC 协议：消息编解码、命令响应、事件 |
| Tiling | `rway-harness/src/categories/tiling.rs` | 平铺引擎：布局算法、gaps 计算 |
| Window | `rway-harness/src/categories/window.rs` | 窗口管理：插入/移除、浮动、全屏 |
| Workspace | `rway-harness/src/categories/workspace.rs` | 工作区：创建、切换、列表 |
| Input | `rway-harness/src/categories/input.rs` | 输入配置 |
| Output | `rway-harness/src/categories/output.rs` | 输出配置 |
| Appearance | `rway-harness/src/categories/appearance.rs` | 外观：边框、颜色 |
