// ipc.rs — IPC 事件循环集成：将 rway-ipc 的 Unix socket 接入 calloop
//
// 处理 swaymsg/waybar 的 IPC 请求，从合成器状态生成响应。

use std::collections::HashSet;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use smithay::reexports::calloop::LoopHandle;

use rway_ipc::{
    events::SubscriptionType,
    protocol::{self, EventType, HEADER_SIZE},
    IpcMode, IpcRect, OutputInfo, TreeNode, VersionInfo, WorkspaceInfo,
};

use crate::state::RwayState;

/// A client that has subscribed to IPC events via the Subscribe command.
/// The connection is kept open and events are pushed asynchronously.
pub(crate) struct IpcSubscriber {
    pub stream: UnixStream,
    pub subscriptions: HashSet<SubscriptionType>,
}

impl IpcSubscriber {
    /// Send an event to this subscriber if it matches their subscriptions.
    /// Returns false if the write failed (connection dead).
    pub fn send_event(&mut self, event_type: EventType, payload: &[u8]) -> bool {
        let sub_type = match event_type {
            EventType::Workspace => SubscriptionType::Workspace,
            EventType::Output => SubscriptionType::Output,
            EventType::Mode => SubscriptionType::Mode,
            EventType::Window => SubscriptionType::Window,
            EventType::BarConfigUpdate => SubscriptionType::BarConfigUpdate,
            EventType::Binding => SubscriptionType::Binding,
            EventType::Shutdown => SubscriptionType::Shutdown,
            EventType::Tick => SubscriptionType::Tick,
            EventType::BarStateUpdate => SubscriptionType::BarStateUpdate,
            EventType::Input => SubscriptionType::Input,
        };

        if !self.subscriptions.contains(&sub_type) {
            return true; // Not subscribed, but connection is still alive
        }

        let msg = protocol::encode_message(event_type as u32, payload);
        self.stream.write_all(&msg).is_ok()
    }
}

/// Register the IPC server with the calloop event loop.
/// Polls every 16ms (~1 frame) for new client connections.
pub(crate) fn register_ipc_source(handle: &LoopHandle<'static, RwayState>) {
    handle
        .insert_source(
            smithay::reexports::calloop::timer::Timer::from_duration(
                std::time::Duration::from_millis(16),
            ),
            |_, _, state| {
                poll_ipc_connections(state);
                smithay::reexports::calloop::timer::TimeoutAction::ToDuration(
                    std::time::Duration::from_millis(16),
                )
            },
        )
        .expect("Failed to register IPC poll source");
}

/// 轮询 IPC 连接并处理请求
fn poll_ipc_connections(state: &mut RwayState) {
    let server = match &state.ipc_server {
        Some(s) => s,
        None => return,
    };

    // Accept all pending connections
    let mut new_streams: Vec<UnixStream> = Vec::new();
    while let Some(stream) = server.try_accept() {
        new_streams.push(stream);
    }

    for stream in new_streams {
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(100)));
        handle_ipc_client(stream, state);
    }

    // Clean up dead subscribers by attempting a zero-byte write.
    // peer_addr() is unreliable for detecting dead Unix domain sockets.
    state.ipc_subscribers.retain_mut(|sub| {
        // A zero-byte write succeeds on live sockets, fails on dead ones
        match sub.stream.write(&[]) {
            Ok(_) => true,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => true, // nonblocking, still alive
            Err(_) => false, // broken pipe, connection reset, etc.
        }
    });
}

/// Handle an IPC client connection. Reads messages in a loop so clients
/// can send multiple requests on the same connection. When a Subscribe
/// message arrives, the stream is MOVED into the subscriber list (keeping
/// the connection alive for event pushing).
fn handle_ipc_client(mut stream: UnixStream, state: &mut RwayState) {
    loop {
        let mut header_buf = [0u8; HEADER_SIZE];
        if stream.read_exact(&mut header_buf).is_err() {
            return; // Connection closed or read timeout — drop stream
        }

        let (payload_len, msg_type) = match protocol::decode_header(&header_buf) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("IPC header parse error: {}", e);
                return;
            }
        };

        if payload_len > 1_048_576 {
            tracing::warn!("IPC payload too large: {} bytes", payload_len);
            return;
        }
        let mut payload = vec![0u8; payload_len as usize];
        if payload_len > 0 && stream.read_exact(&mut payload).is_err() {
            return;
        }

        tracing::info!("IPC request: type={} payload_len={}", msg_type, payload_len);

        // Subscribe (msg_type 2): move stream into subscriber list
        if msg_type == 2 {
            let payload_str = String::from_utf8_lossy(&payload);
            let subs = rway_ipc::parse_subscribe_payload(&payload_str).unwrap_or_default();
            tracing::info!("IPC subscribe: {:?}", subs);

            let reply = protocol::encode_reply(
                protocol::MessageType::Subscribe,
                &serde_json::json!({"success": true}),
            );
            if let Err(e) = stream.write_all(&reply) {
                tracing::warn!("IPC subscribe reply write failed: {}", e);
                return;
            }
            let _ = stream.flush();

            // Move the stream into subscriber list — no clone, no premature close
            let _ = stream.set_nonblocking(true);
            state.ipc_subscribers.push(IpcSubscriber {
                stream,
                subscriptions: subs.into_iter().collect(),
            });
            return; // Stream ownership transferred, don't drop it
        }

        // Normal request-response
        let response = dispatch_ipc_message(state, msg_type, &payload);
        if stream.write_all(&response).is_err() {
            return;
        }
    }
    // Non-subscribe connections: stream dropped here, closing the connection
}

/// Broadcast an event to all subscribers with matching subscriptions.
/// Removes dead connections automatically.
pub(crate) fn broadcast_event(state: &mut RwayState, event_type: EventType, payload: &[u8]) {
    state
        .ipc_subscribers
        .retain_mut(|sub| sub.send_event(event_type, payload));
}

/// Broadcast a workspace focus event to IPC subscribers.
pub(crate) fn broadcast_workspace_focus(state: &mut RwayState) {
    if state.ipc_subscribers.is_empty() {
        return;
    }

    let workspaces = rway_tiling::workspace::get_workspaces(&state.tiling);
    let current = workspaces
        .iter()
        .find(|(_, _, vis)| *vis)
        .map(|(_, name, _)| {
            let rect = output_rect(state);
            WorkspaceInfo {
                id: 1,
                num: name.parse::<i32>().unwrap_or(1),
                name: name.clone(),
                visible: true,
                focused: true,
                urgent: false,
                output: "winit".to_string(),
                rect,
            }
        });

    let event = rway_ipc::WorkspaceEvent {
        change: "focus".to_string(),
        current,
        old: None,
    };
    let payload = serde_json::to_vec(&event).unwrap_or_default();
    broadcast_event(state, EventType::Workspace, &payload);
}

/// Broadcast a window event (new, close, focus) to IPC subscribers.
pub(crate) fn broadcast_window_event(state: &mut RwayState, change: &str) {
    if state.ipc_subscribers.is_empty() {
        return;
    }

    // Build a minimal container node for the focused window
    let focused_id = state.tiling.focused_window_id().unwrap_or(0);
    let container = TreeNode {
        id: focused_id as i64,
        name: None,
        node_type: "con".to_string(),
        layout: "none".to_string(),
        focused: true,
        urgent: false,
        rect: IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        window_rect: zero_rect(),
        deco_rect: zero_rect(),
        geometry: zero_rect(),
        nodes: vec![],
        floating_nodes: vec![],
        focus: vec![],
        app_id: None,
        window: None,
    };

    let event = rway_ipc::WindowEvent {
        change: change.to_string(),
        container,
    };
    let payload = serde_json::to_vec(&event).unwrap_or_default();
    broadcast_event(state, EventType::Window, &payload);
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
    let focus: Vec<i64> = children.iter().map(|c| c.index() as i64).collect();

    TreeNode {
        id: node_id.index() as i64,
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
        id: node_id.index() as i64,
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
