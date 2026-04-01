/// rway-tiling：纯 Rust 瓦片布局引擎（零外部依赖）
///
/// 模块结构：
/// - [`tree`]      — Arena 分配的 N 叉树（核心数据结构）
/// - [`container`] — 容器辅助操作（sizes、focused_child 管理）
/// - [`layout`]    — 布局算法（SplitH/SplitV/Tabbed/Stacked）
/// - [`workspace`] — 工作区与输出管理
/// - [`commands`]  — 高级命令（插入窗口、移除窗口、移动焦点、切换布局）
pub mod tree;
pub mod container;
pub mod layout;
pub mod workspace;
pub mod commands;

// 重导出最常用的公开类型，方便外部 crate 使用
pub use tree::{Layout, Node, NodeData, NodeId, Rect, Tree};
pub use commands::Direction;
