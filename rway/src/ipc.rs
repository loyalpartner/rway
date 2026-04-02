// ipc.rs — IPC 事件循环集成：将 rway-ipc 的 Unix socket 接入 calloop
//
// 处理 swaymsg/waybar 的 IPC 请求，从合成器状态生成响应。

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use smithay::reexports::calloop::LoopHandle;

use rway_ipc::{
    protocol::{self, HEADER_SIZE},
    IpcMode, IpcRect, OutputInfo, TreeNode, VersionInfo, WorkspaceInfo,
};

use crate::state::RwayState;

/// 将 IPC 服务器注册到 calloop 事件循环
pub fn register_ipc_source(handle: &LoopHandle<'static, RwayState>) {
    // 使用定时器轮询 IPC 连接（简单可靠的方式）
    // 每 50ms 检查一次新连接
    handle
        .insert_source(
            smithay::reexports::calloop::timer::Timer::from_duration(
                std::time::Duration::from_millis(50),
            ),
            |_, _, state| {
                poll_ipc_connections(state);
                smithay::reexports::calloop::timer::TimeoutAction::ToDuration(
                    std::time::Duration::from_millis(50),
                )
            },
        )
        .expect("注册 IPC 轮询源失败");
}

/// 轮询 IPC 连接并处理请求
fn poll_ipc_connections(state: &mut RwayState) {
    let server = match &state.ipc_server {
        Some(s) => s,
        None => return,
    };

    // 处理所有等待的连接
    while let Some(mut stream) = server.try_accept() {
        // 设置读超时以避免阻塞事件循环
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(100)));
        handle_ipc_client(&mut stream, state);
    }
}

/// 处理单个 IPC 客户端连接
fn handle_ipc_client(stream: &mut UnixStream, state: &RwayState) {
    // 读取消息头
    let mut header_buf = [0u8; HEADER_SIZE];
    if stream.read_exact(&mut header_buf).is_err() {
        return;
    }

    let (payload_len, msg_type) = match protocol::decode_header(&header_buf) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("IPC 消息头解析失败: {}", e);
            return;
        }
    };

    // 读取 payload（限制最大 1MB）
    if payload_len > 1_048_576 {
        tracing::warn!("IPC payload 过大: {} bytes", payload_len);
        return;
    }
    let mut payload = vec![0u8; payload_len as usize];
    if payload_len > 0 && stream.read_exact(&mut payload).is_err() {
        return;
    }

    tracing::debug!("IPC 请求: type={} payload_len={}", msg_type, payload_len);

    // 分发处理并写回响应
    let response = dispatch_ipc_message(state, msg_type, &payload);
    let _ = stream.write_all(&response);
}

/// 根据消息类型分发并生成响应
fn dispatch_ipc_message(state: &RwayState, msg_type: u32, payload: &[u8]) -> Vec<u8> {
    let response_json = match msg_type {
        0 => handle_run_command(payload),
        1 => handle_get_workspaces(state),
        3 => handle_get_outputs(state),
        4 => handle_get_tree(state),
        7 => handle_get_version(),
        100 => serde_json::json!([]), // GET_INPUTS: 空列表
        101 => serde_json::json!([]), // GET_SEATS: 空列表
        _ => serde_json::json!([{"success": false, "error": "unsupported"}]),
    };

    let json_bytes = serde_json::to_vec(&response_json).unwrap_or_default();
    protocol::encode_message(msg_type, &json_bytes)
}

// ── 消息处理函数 ──────────────────────────────────────────────

fn handle_run_command(payload: &[u8]) -> serde_json::Value {
    let cmd = String::from_utf8_lossy(payload);
    tracing::info!("IPC run_command: {}", cmd);
    serde_json::json!([{"success": true}])
}

fn handle_get_workspaces(state: &RwayState) -> serde_json::Value {
    let workspaces = rway_tiling::workspace::get_workspaces(&state.tiling);
    let ws_list: Vec<WorkspaceInfo> = workspaces
        .iter()
        .enumerate()
        .map(|(i, (_, name, visible))| WorkspaceInfo {
            id: i as i64 + 1,
            num: name.parse::<i32>().unwrap_or(i as i32 + 1),
            name: name.clone(),
            visible: *visible,
            focused: *visible,
            urgent: false,
            output: "winit".to_string(),
            rect: output_rect(state),
        })
        .collect();
    serde_json::to_value(&ws_list).unwrap_or(serde_json::json!([]))
}

fn handle_get_outputs(state: &RwayState) -> serde_json::Value {
    let rect = output_rect(state);
    let current_ws = rway_tiling::workspace::get_workspaces(&state.tiling)
        .into_iter()
        .find(|(_, _, vis)| *vis)
        .map(|(_, name, _)| name);

    let outputs = vec![OutputInfo {
        id: 0,
        name: "winit".to_string(),
        make: "Smithay".to_string(),
        model: "Winit".to_string(),
        serial: "Unknown".to_string(),
        active: true,
        primary: true,
        scale: 1.0,
        transform: "normal".to_string(),
        current_workspace: current_ws,
        rect: rect.clone(),
        current_mode: IpcMode {
            width: rect.width,
            height: rect.height,
            refresh: 60000,
        },
    }];
    serde_json::to_value(&outputs).unwrap_or(serde_json::json!([]))
}

fn handle_get_tree(state: &RwayState) -> serde_json::Value {
    let root_rect = output_rect(state);
    let tree = build_tree_node(state, state.tiling.root(), &root_rect);
    serde_json::to_value(&tree).unwrap_or(serde_json::json!({}))
}

fn handle_get_version() -> serde_json::Value {
    serde_json::to_value(&VersionInfo {
        major: 0,
        minor: 1,
        patch: 0,
        human_readable: "rway 0.1.0 (sway-compatible)".to_string(),
        loaded_config_file_name: String::new(),
    })
    .unwrap_or(serde_json::json!({}))
}

// ── 辅助函数 ──────────────────────────────────────────────────

fn output_rect(state: &RwayState) -> IpcRect {
    state
        .space
        .outputs()
        .next()
        .and_then(|o| state.space.output_geometry(o))
        .map(|geo| IpcRect {
            x: geo.loc.x,
            y: geo.loc.y,
            width: geo.size.w,
            height: geo.size.h,
        })
        .unwrap_or(IpcRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        })
}

fn zero_rect() -> IpcRect {
    IpcRect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    }
}

/// 递归构建 sway 兼容的 IPC 树节点
fn build_tree_node(
    state: &RwayState,
    node_id: rway_tiling::NodeId,
    parent_rect: &IpcRect,
) -> TreeNode {
    let Some(node) = state.tiling.get(node_id) else {
        return empty_tree_node(node_id);
    };

    let (node_type, layout, name, rect) = match &node.data {
        rway_tiling::NodeData::Root => ("root", "splith", None, parent_rect.clone()),
        rway_tiling::NodeData::Output { name, geometry } => (
            "output",
            "output",
            Some(name.clone()),
            IpcRect {
                x: geometry.x,
                y: geometry.y,
                width: geometry.width,
                height: geometry.height,
            },
        ),
        rway_tiling::NodeData::Workspace { name, .. } => (
            "workspace",
            "splith",
            Some(name.clone()),
            parent_rect.clone(),
        ),
        rway_tiling::NodeData::Container { layout, .. } => (
            "con",
            match layout {
                rway_tiling::Layout::SplitH => "splith",
                rway_tiling::Layout::SplitV => "splitv",
                rway_tiling::Layout::Tabbed => "tabbed",
                rway_tiling::Layout::Stacked => "stacked",
            },
            None,
            parent_rect.clone(),
        ),
        rway_tiling::NodeData::Window { geometry, .. } => (
            "con",
            "none",
            None,
            IpcRect {
                x: geometry.x,
                y: geometry.y,
                width: geometry.width,
                height: geometry.height,
            },
        ),
    };

    let children: Vec<rway_tiling::NodeId> = state.tiling.children(node_id).to_vec();
    let child_nodes: Vec<TreeNode> = children
        .iter()
        .map(|&cid| build_tree_node(state, cid, &rect))
        .collect();
    let focus: Vec<i64> = children.iter().map(|c| c.0 as i64).collect();

    TreeNode {
        id: node_id.0 as i64,
        name,
        node_type: node_type.to_string(),
        layout: layout.to_string(),
        focused: false,
        urgent: false,
        rect,
        window_rect: zero_rect(),
        deco_rect: zero_rect(),
        geometry: zero_rect(),
        nodes: child_nodes,
        floating_nodes: vec![],
        focus,
        app_id: None,
        window: None,
    }
}

fn empty_tree_node(node_id: rway_tiling::NodeId) -> TreeNode {
    TreeNode {
        id: node_id.0 as i64,
        name: None,
        node_type: "con".to_string(),
        layout: "none".to_string(),
        focused: false,
        urgent: false,
        rect: zero_rect(),
        window_rect: zero_rect(),
        deco_rect: zero_rect(),
        geometry: zero_rect(),
        nodes: vec![],
        floating_nodes: vec![],
        focus: vec![],
        app_id: None,
        window: None,
    }
}
