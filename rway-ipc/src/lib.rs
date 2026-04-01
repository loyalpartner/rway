// rway-ipc: Sway 兼容的 IPC 协议实现
//
// 实现 i3/Sway IPC 二进制协议（通过 Unix socket 通信）。
// 确保 swaymsg、waybar 等工具无需修改即可与 rway 合作。

pub mod commands;
pub mod events;
pub mod protocol;
pub mod server;

// 协议核心导出
pub use protocol::{
    decode_header, encode_event, encode_message, encode_reply, EventType, MessageType,
    ProtocolError, HEADER_SIZE, MAGIC,
};

// 命令响应类型导出
pub use commands::{
    CommandResult, IpcMode, IpcRect, OutputInfo, TreeNode, VersionInfo, WorkspaceInfo,
};

// 事件类型导出
pub use events::{parse_subscribe_payload, SubscriptionType, WindowEvent, WorkspaceEvent};

// 服务端导出
pub use server::{default_socket_path, IpcServer};
