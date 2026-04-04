// ipc.rs — Sway-compatible IPC server using per-client calloop fd sources
//
// Architecture matches sway's ipc-server.c:
// - Listener fd → calloop Generic → accept connections immediately
// - Each client fd → calloop Generic → async read/write per client
// - Subscribe sets event bitmask, client stays connected for event push
// - Events are broadcast to all subscribed clients

use std::collections::HashSet;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::net::UnixStream;

use smithay::reexports::calloop::{
    generic::Generic, Interest, LoopHandle, Mode, PostAction, RegistrationToken,
};

use rway_ipc::{
    events::SubscriptionType,
    protocol::{self, EventType, HEADER_SIZE},
    IpcMode, IpcRect, OutputInfo, TreeNode, VersionInfo, WorkspaceInfo,
};

use crate::state::RwayState;

// ── Per-client state ─────────────────────────────────────────

/// An active IPC client connection, managed by its own calloop fd source.
pub(crate) struct IpcClient {
    stream: UnixStream,
    subscriptions: HashSet<SubscriptionType>,
    /// calloop token to remove the event source on disconnect
    token: RegistrationToken,
}

impl IpcClient {
    fn send_reply(&mut self, msg_type: u32, payload: &[u8]) -> bool {
        let msg = protocol::encode_message(msg_type, payload);
        let result = self.stream.write_all(&msg);
        let _ = self.stream.flush();
        if let Err(ref e) = result {
            tracing::warn!("IPC send_reply failed: type={} err={}", msg_type, e);
        }
        result.is_ok()
    }

    fn send_event(&mut self, event_type: EventType, payload: &[u8]) -> bool {
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
            return true; // Not subscribed to this event type, still alive
        }
        let msg = protocol::encode_message(event_type as u32, payload);
        self.stream.write_all(&msg).is_ok()
    }
}

// ── Listener registration ────────────────────────────────────

/// Register the IPC listener with calloop. New connections trigger
/// immediate accept (zero-latency, fd-driven, not polled).
pub(crate) fn register_ipc_source(handle: &LoopHandle<'static, RwayState>) {
    // Bootstrap: use an immediate timer to register the listener fd source
    // once RwayState is initialized (listener is in state.ipc_server).
    handle
        .insert_source(
            smithay::reexports::calloop::timer::Timer::immediate(),
            |_, _, state| {
                if let Some(server) = &state.ipc_server {
                    tracing::info!("IPC: registering listener fd source");
                    let fd = server.as_fd().try_clone_to_owned().unwrap();
                    let source = Generic::new(fd, Interest::READ, Mode::Level);
                    state
                        .loop_handle
                        .insert_source(source, |_, _, state| {
                            accept_ipc_connections(state);
                            Ok(PostAction::Continue)
                        })
                        .expect("Failed to register IPC listener fd");
                    tracing::info!("IPC: listener fd source registered OK");
                } else {
                    tracing::warn!("IPC: no server found, skipping listener registration");
                }
                smithay::reexports::calloop::timer::TimeoutAction::Drop
            },
        )
        .expect("Failed to register IPC init timer");
}

/// Accept all pending connections and process them synchronously.
/// Each connection is read immediately (data is typically available
/// because waybar writes right after connect). Subscribed clients
/// are kept alive with calloop fd sources for future event delivery.
fn accept_ipc_connections(state: &mut RwayState) {
    let streams: Vec<UnixStream> = {
        let Some(server) = &state.ipc_server else {
            return;
        };
        let mut v = Vec::new();
        while let Some(stream) = server.try_accept() {
            v.push(stream);
        }
        v
    };

    if streams.is_empty() {
        return;
    }

    // For each accepted connection: try nonblocking read immediately.
    // If data is ready, process it synchronously (fast path).
    // If not ready (WouldBlock), register with calloop for async processing.
    for stream in streams {
        stream.set_nonblocking(true).ok();
        let mut header_buf = [0u8; HEADER_SIZE];
        match (&stream).read_exact(&mut header_buf) {
            Ok(()) => {
                // Data ready — process immediately (no event loop delay)
                let mut stream = stream;
                stream.set_nonblocking(false).ok();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    .ok();
                process_client_with_header(&mut stream, &header_buf, state);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Data not ready yet — register calloop fd source for async read
                if let Err(e) = register_pending_client(state, stream) {
                    tracing::warn!("IPC: failed to register pending client: {}", e);
                }
            }
            Err(_) => {} // Connection error, drop
        }
    }
}

/// Process an IPC message when the header has already been read.
/// Register a client that hasn't sent data yet. Calloop will fire
/// when data arrives, and we process it then (no blocking).
fn register_pending_client(state: &mut RwayState, stream: UnixStream) -> std::io::Result<()> {
    let raw_fd = stream.as_raw_fd();
    let owned_fd = stream.as_fd().try_clone_to_owned()?;

    let source = Generic::new(owned_fd, Interest::READ, Mode::Level);
    let token = state
        .loop_handle
        .insert_source(source, move |_, _, state| {
            if !handle_client_readable(state, raw_fd) {
                remove_client(state, raw_fd);
            }
            Ok(PostAction::Continue)
        })
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Store as a client with no subscriptions (yet)
    stream.set_nonblocking(false).ok();
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(100)))
        .ok();
    state.ipc_clients.push(IpcClient {
        stream,
        subscriptions: HashSet::new(),
        token,
    });
    Ok(())
}

/// Process messages on a stream starting with an already-read header.
/// Loops to handle multiple messages (waybar sends 2 subscribes on one fd).
fn process_client_with_header(
    stream: &mut UnixStream,
    first_header: &[u8; HEADER_SIZE],
    state: &mut RwayState,
) {
    let mut all_subs: HashSet<SubscriptionType> = HashSet::new();
    let mut header_buf = *first_header;
    let mut first = true;

    loop {
        if !first && stream.read_exact(&mut header_buf).is_err() {
            break;
        }
        first = false;

        let (payload_len, msg_type) = match protocol::decode_header(&header_buf) {
            Ok(v) => v,
            Err(_) => break,
        };

        if payload_len > 1_048_576 {
            break;
        }

        let mut payload = vec![0u8; payload_len as usize];
        if payload_len > 0 && stream.read_exact(&mut payload).is_err() {
            break;
        }

        tracing::info!("IPC request: type={} payload_len={}", msg_type, payload_len);

        if msg_type == 2 {
            let payload_str = String::from_utf8_lossy(&payload);
            let subs = rway_ipc::parse_subscribe_payload(&payload_str).unwrap_or_default();
            tracing::info!("IPC subscribe: {:?}", subs);

            let msg = protocol::encode_message(2, b"{\"success\": true}");
            let _ = stream.write_all(&msg);
            let _ = stream.flush();
            all_subs.extend(subs);
        } else {
            let response = dispatch_ipc_message(state, msg_type, &payload);
            let msg = protocol::encode_message(msg_type, &response);
            let _ = stream.write_all(&msg);
        }
    }

    if !all_subs.is_empty() {
        if let Ok(cloned) = stream.try_clone() {
            let subs_vec: Vec<SubscriptionType> = all_subs.into_iter().collect();
            if let Err(e) = register_subscriber(state, cloned, subs_vec) {
                tracing::warn!("IPC: failed to register subscriber: {}", e);
            }
        }
    }
}

/// Register a subscribed client for event delivery via calloop fd source.
fn register_subscriber(
    state: &mut RwayState,
    stream: UnixStream,
    subs: Vec<SubscriptionType>,
) -> std::io::Result<()> {
    let raw_fd = stream.as_raw_fd();
    let owned_fd = stream.as_fd().try_clone_to_owned()?;

    // Monitor for disconnect (readable = either data or hangup)
    let source = Generic::new(owned_fd, Interest::READ, Mode::Level);
    let token = state
        .loop_handle
        .insert_source(source, move |_, _, state| {
            // Subscriber sent more data or disconnected — handle it
            if !handle_client_readable(state, raw_fd) {
                remove_client(state, raw_fd);
            }
            Ok(PostAction::Continue)
        })
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    stream.set_nonblocking(true)?;
    state.ipc_clients.push(IpcClient {
        stream,
        subscriptions: subs.into_iter().collect(),
        token,
    });

    Ok(())
}

/// Remove a client by raw fd: unregister calloop source and drop client.
fn remove_client(state: &mut RwayState, raw_fd: i32) {
    if let Some(idx) = state
        .ipc_clients
        .iter()
        .position(|c| c.stream.as_raw_fd() == raw_fd)
    {
        let client = state.ipc_clients.remove(idx);
        state.loop_handle.remove(client.token);
    }
}

// ── Per-client readable handler ──────────────────────────────

/// Called when a client fd is readable. Returns false if client disconnected.
fn handle_client_readable(state: &mut RwayState, raw_fd: i32) -> bool {
    // Find the client by fd
    let client_idx = match state
        .ipc_clients
        .iter()
        .position(|c| c.stream.as_raw_fd() == raw_fd)
    {
        Some(idx) => idx,
        None => return false,
    };

    // Read header (14 bytes)
    let mut header_buf = [0u8; HEADER_SIZE];
    match state.ipc_clients[client_idx]
        .stream
        .read_exact(&mut header_buf)
    {
        Ok(()) => {}
        Err(ref e)
            if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut =>
        {
            return true; // No data yet, try again on next calloop wakeup
        }
        Err(_) => return false, // Disconnected
    }

    let (payload_len, msg_type) = match protocol::decode_header(&header_buf) {
        Ok(v) => v,
        Err(_) => return false,
    };

    if payload_len > 1_048_576 {
        return false;
    }

    // Read payload
    let mut payload = vec![0u8; payload_len as usize];
    if payload_len > 0
        && state.ipc_clients[client_idx]
            .stream
            .read_exact(&mut payload)
            .is_err()
    {
        return false;
    }

    tracing::info!("IPC request: type={} payload_len={}", msg_type, payload_len);

    // Dispatch the message
    if msg_type == 2 {
        // Subscribe
        let payload_str = String::from_utf8_lossy(&payload);
        let subs = rway_ipc::parse_subscribe_payload(&payload_str).unwrap_or_default();
        tracing::info!("IPC subscribe: {:?}", subs);

        let reply = serde_json::to_vec(&serde_json::json!({"success": true})).unwrap();
        let sent = state.ipc_clients[client_idx].send_reply(2, &reply);
        tracing::info!("IPC subscribe reply sent: {}", sent);
        state.ipc_clients[client_idx].subscriptions.extend(subs);
    } else {
        let response = dispatch_ipc_message(state, msg_type, &payload);
        if !state.ipc_clients[client_idx].send_reply(msg_type, &response) {
            return false;
        }
    }

    true
}

// ── Event broadcasting ───────────────────────────────────────

/// Broadcast an event to all subscribed clients. Dead connections are removed.
pub(crate) fn broadcast_event(state: &mut RwayState, event_type: EventType, payload: &[u8]) {
    let mut dead_fds = Vec::new();
    for client in &mut state.ipc_clients {
        if !client.send_event(event_type, payload) {
            dead_fds.push(client.stream.as_raw_fd());
        }
    }
    for fd in dead_fds {
        remove_client(state, fd);
    }
}

/// Broadcast workspace focus event.
pub(crate) fn broadcast_workspace_focus(state: &mut RwayState) {
    if state.ipc_clients.is_empty() {
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

/// Broadcast window event (new, close, focus).
pub(crate) fn broadcast_window_event(state: &mut RwayState, change: &str) {
    if state.ipc_clients.is_empty() {
        return;
    }

    let focused_id = state.tiling.focused_window_id().unwrap_or(0);
    let container = TreeNode {
        id: focused_id as i64,
        name: None,
        node_type: "con".to_string(),
        layout: "none".to_string(),
        focused: true,
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
    };

    let event = rway_ipc::WindowEvent {
        change: change.to_string(),
        container,
    };
    let payload = serde_json::to_vec(&event).unwrap_or_default();
    broadcast_event(state, EventType::Window, &payload);
}

// ── Message dispatch ─────────────────────────────────────────

fn dispatch_ipc_message(state: &RwayState, msg_type: u32, payload: &[u8]) -> Vec<u8> {
    let response_json = match msg_type {
        0 => handle_run_command(payload),
        1 => handle_get_workspaces(state),
        3 => handle_get_outputs(state),
        4 => handle_get_tree(state),
        7 => handle_get_version(),
        100 => serde_json::json!([]),
        101 => serde_json::json!([]),
        _ => serde_json::json!([{"success": false, "error": "unsupported"}]),
    };
    serde_json::to_vec(&response_json).unwrap_or_default()
}

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

// ── Helpers ──────────────────────────────────────────────────

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
