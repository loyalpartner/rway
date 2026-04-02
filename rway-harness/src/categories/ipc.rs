//! IPC 协议兼容性测试

use crate::{Category, CompatTest, Harness, Priority, TestStatus};

pub fn register(harness: &mut Harness) {
    harness.register(CompatTest {
        category: Category::Ipc,
        name: "version_info_fields".into(),
        description: "get_version 响应包含所有必需字段".into(),
        sway_feature: "get_version".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_version_info_fields),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "version_info_json_format".into(),
        description: "get_version JSON 格式与 Sway 兼容".into(),
        sway_feature: "get_version".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_version_info_json_format),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "workspace_info_fields".into(),
        description: "get_workspaces 响应包含所有必需字段".into(),
        sway_feature: "get_workspaces".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_info_fields),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "tree_node_type_rename".into(),
        description: "get_tree 节点 type 字段正确重命名".into(),
        sway_feature: "get_tree".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_tree_node_type_rename),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "tree_node_nested".into(),
        description: "get_tree 支持嵌套子节点".into(),
        sway_feature: "get_tree".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_tree_node_nested),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "command_result_format".into(),
        description: "命令结果数组格式与 Sway 兼容".into(),
        sway_feature: "run_command".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_command_result_format),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "output_info_fields".into(),
        description: "get_outputs 响应包含所有必需字段".into(),
        sway_feature: "get_outputs".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_output_info_fields),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "protocol_magic_bytes".into(),
        description: "IPC 协议 magic bytes 正确".into(),
        sway_feature: "ipc protocol".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_protocol_magic_bytes),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "protocol_header_size".into(),
        description: "IPC 协议 header 大小正确".into(),
        sway_feature: "ipc protocol".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_protocol_header_size),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "tree_node_optional_skip".into(),
        description: "TreeNode 可选字段为 None 时不序列化".into(),
        sway_feature: "get_tree".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_tree_node_optional_skip),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "subscribe_event_types".into(),
        description: "支持 workspace/window 事件订阅".into(),
        sway_feature: "subscribe".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_subscribe_event_types),
    });

    // --- Group 1: MessageType value verification ---

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_run_command".into(),
        description: "MessageType::RunCommand == 0".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_message_type_run_command),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_workspaces".into(),
        description: "MessageType::GetWorkspaces == 1".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_message_type_get_workspaces),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_subscribe".into(),
        description: "MessageType::Subscribe == 2".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_message_type_subscribe),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_outputs".into(),
        description: "MessageType::GetOutputs == 3".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_message_type_get_outputs),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_tree".into(),
        description: "MessageType::GetTree == 4".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_message_type_get_tree),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_marks".into(),
        description: "MessageType::GetMarks == 5".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_message_type_get_marks),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_bar_config".into(),
        description: "MessageType::GetBarConfig == 6".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_message_type_get_bar_config),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_version".into(),
        description: "MessageType::GetVersion == 7".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_message_type_get_version),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_binding_modes".into(),
        description: "MessageType::GetBindingModes == 8".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_message_type_get_binding_modes),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_config".into(),
        description: "MessageType::GetConfig == 9".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_message_type_get_config),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_inputs".into(),
        description: "MessageType::GetInputs == 100".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_message_type_get_inputs),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_get_seats".into(),
        description: "MessageType::GetSeats == 101".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_message_type_get_seats),
    });

    // --- Group 2: EventType value verification ---

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_workspace".into(),
        description: "EventType::Workspace == 0x80000000".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_event_type_workspace),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_window".into(),
        description: "EventType::Window == 0x80000003".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_event_type_window),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_output".into(),
        description: "EventType::Output == 0x80000001".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_event_type_output),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_mode".into(),
        description: "EventType::Mode == 0x80000002".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_event_type_mode),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_binding".into(),
        description: "EventType::Binding == 0x80000005".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_event_type_binding),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_shutdown".into(),
        description: "EventType::Shutdown == 0x80000006".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_event_type_shutdown),
    });

    // --- Group 3: Protocol round-trip ---

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "encode_decode_roundtrip".into(),
        description: "IPC 协议 encode/decode 往返正确".into(),
        sway_feature: "ipc protocol encode/decode".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_encode_decode_roundtrip),
    });

    // --- Group 4: Additional EventType / MessageType values ---

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_tick".into(),
        description: "EventType::Tick == 0x80000007".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_event_type_tick),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_bar_config_update".into(),
        description: "EventType::BarConfigUpdate == 0x80000004".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_event_type_bar_config_update),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_bar_state_update".into(),
        description: "EventType::BarStateUpdate == 0x80000014".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_event_type_bar_state_update),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "event_type_input".into(),
        description: "EventType::Input == 0x80000015".into(),
        sway_feature: "ipc event types".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_event_type_input),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_send_tick".into(),
        description: "MessageType::SendTick == 10".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_message_type_send_tick),
    });

    harness.register(CompatTest {
        category: Category::Ipc,
        name: "message_type_sync".into(),
        description: "MessageType::Sync == 11".into(),
        sway_feature: "ipc message types".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_message_type_sync),
    });
}

fn test_version_info_fields() -> TestStatus {
    let version = rway_ipc::VersionInfo {
        major: 1,
        minor: 0,
        patch: 0,
        human_readable: "1.0.0".into(),
        loaded_config_file_name: "/etc/sway/config".into(),
    };
    let json = serde_json::to_value(&version).unwrap();

    let required = [
        "major",
        "minor",
        "patch",
        "human_readable",
        "loaded_config_file_name",
    ];
    for field in &required {
        if json.get(field).is_none() {
            return TestStatus::Fail(format!("missing field: {field}"));
        }
    }
    TestStatus::Pass
}

fn test_version_info_json_format() -> TestStatus {
    let version = rway_ipc::VersionInfo {
        major: 1,
        minor: 8,
        patch: 0,
        human_readable: "1.8".into(),
        loaded_config_file_name: "/etc/sway/config".into(),
    };
    let json = serde_json::to_value(&version).unwrap();
    let expected = serde_json::json!({
        "major": 1,
        "minor": 8,
        "patch": 0,
        "human_readable": "1.8",
        "loaded_config_file_name": "/etc/sway/config"
    });
    if json == expected {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("JSON mismatch: {json}"))
    }
}

fn test_workspace_info_fields() -> TestStatus {
    let ws = rway_ipc::WorkspaceInfo {
        id: 1,
        num: 1,
        name: "1".into(),
        visible: true,
        focused: true,
        urgent: false,
        output: "eDP-1".into(),
        rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
    };
    let json = serde_json::to_value(&ws).unwrap();

    let required = [
        "id", "num", "name", "visible", "focused", "urgent", "output", "rect",
    ];
    for field in &required {
        if json.get(field).is_none() {
            return TestStatus::Fail(format!("missing field: {field}"));
        }
    }
    TestStatus::Pass
}

fn test_tree_node_type_rename() -> TestStatus {
    let node = rway_ipc::TreeNode {
        id: 1,
        name: None,
        node_type: "root".into(),
        layout: "output".into(),
        focused: false,
        urgent: false,
        rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        window_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        deco_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        geometry: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        nodes: vec![],
        floating_nodes: vec![],
        focus: vec![],
        app_id: None,
        window: None,
    };
    let json = serde_json::to_value(&node).unwrap();

    if json.get("type").is_some() && json.get("node_type").is_none() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("'type' field rename not working".into())
    }
}

fn test_tree_node_nested() -> TestStatus {
    let child = rway_ipc::TreeNode {
        id: 2,
        name: Some("child".into()),
        node_type: "con".into(),
        layout: "none".into(),
        focused: true,
        urgent: false,
        rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 960,
            height: 1080,
        },
        window_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        deco_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        geometry: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        nodes: vec![],
        floating_nodes: vec![],
        focus: vec![],
        app_id: Some("test".into()),
        window: None,
    };
    let parent = rway_ipc::TreeNode {
        id: 1,
        name: Some("ws".into()),
        node_type: "workspace".into(),
        layout: "splith".into(),
        focused: true,
        urgent: false,
        rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        window_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        deco_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        geometry: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        nodes: vec![child],
        floating_nodes: vec![],
        focus: vec![2],
        app_id: None,
        window: None,
    };
    let json = serde_json::to_value(&parent).unwrap();

    if let Some(nodes) = json["nodes"].as_array() {
        if nodes.len() == 1 && nodes[0]["id"] == 2 {
            return TestStatus::Pass;
        }
    }
    TestStatus::Fail("nested tree node serialization failed".into())
}

fn test_command_result_format() -> TestStatus {
    let results = vec![rway_ipc::CommandResult {
        success: true,
        error: None,
    }];
    let json = serde_json::to_value(&results).unwrap();
    let expected = serde_json::json!([{"success": true}]);

    if json == expected {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected {expected}, got {json}"))
    }
}

fn test_output_info_fields() -> TestStatus {
    let output = rway_ipc::OutputInfo {
        id: 1,
        name: "eDP-1".into(),
        make: "".into(),
        model: "".into(),
        serial: "".into(),
        active: true,
        primary: true,
        scale: 1.0,
        transform: "normal".into(),
        current_workspace: Some("1".into()),
        rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        current_mode: rway_ipc::IpcMode {
            width: 1920,
            height: 1080,
            refresh: 60000,
        },
    };
    let json = serde_json::to_value(&output).unwrap();

    let required = [
        "id",
        "name",
        "make",
        "model",
        "serial",
        "active",
        "primary",
        "scale",
        "transform",
        "current_workspace",
        "rect",
        "current_mode",
    ];
    for field in &required {
        if json.get(field).is_none() {
            return TestStatus::Fail(format!("missing field: {field}"));
        }
    }
    TestStatus::Pass
}

fn test_protocol_magic_bytes() -> TestStatus {
    if rway_ipc::MAGIC == b"i3-ipc" {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected b\"i3-ipc\", got {:?}", rway_ipc::MAGIC))
    }
}

fn test_protocol_header_size() -> TestStatus {
    // i3-ipc header: 6 bytes magic + 4 bytes length + 4 bytes type = 14
    if rway_ipc::HEADER_SIZE == 14 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 14, got {}", rway_ipc::HEADER_SIZE))
    }
}

fn test_tree_node_optional_skip() -> TestStatus {
    let node = rway_ipc::TreeNode {
        id: 1,
        name: None,
        node_type: "workspace".into(),
        layout: "splith".into(),
        focused: false,
        urgent: false,
        rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        window_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        deco_rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        geometry: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        nodes: vec![],
        floating_nodes: vec![],
        focus: vec![],
        app_id: None,
        window: None,
    };
    let json = serde_json::to_value(&node).unwrap();

    if json.get("app_id").is_none() && json.get("window").is_none() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("optional fields should be skipped when None".into())
    }
}

fn test_subscribe_event_types() -> TestStatus {
    // 验证事件订阅解析支持 workspace 和 window
    let payload = r#"["workspace", "window"]"#;
    match rway_ipc::parse_subscribe_payload(payload) {
        Ok(types) => {
            let has_workspace = types.contains(&rway_ipc::SubscriptionType::Workspace);
            let has_window = types.contains(&rway_ipc::SubscriptionType::Window);
            if has_workspace && has_window {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "missing event types: ws={has_workspace} win={has_window}"
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

// --- Group 1: MessageType value verification ---

fn test_message_type_run_command() -> TestStatus {
    if rway_ipc::MessageType::RunCommand as u32 == 0 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0, got {}",
            rway_ipc::MessageType::RunCommand as u32
        ))
    }
}

fn test_message_type_get_workspaces() -> TestStatus {
    if rway_ipc::MessageType::GetWorkspaces as u32 == 1 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 1, got {}",
            rway_ipc::MessageType::GetWorkspaces as u32
        ))
    }
}

fn test_message_type_subscribe() -> TestStatus {
    if rway_ipc::MessageType::Subscribe as u32 == 2 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 2, got {}",
            rway_ipc::MessageType::Subscribe as u32
        ))
    }
}

fn test_message_type_get_outputs() -> TestStatus {
    if rway_ipc::MessageType::GetOutputs as u32 == 3 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 3, got {}",
            rway_ipc::MessageType::GetOutputs as u32
        ))
    }
}

fn test_message_type_get_tree() -> TestStatus {
    if rway_ipc::MessageType::GetTree as u32 == 4 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 4, got {}",
            rway_ipc::MessageType::GetTree as u32
        ))
    }
}

fn test_message_type_get_marks() -> TestStatus {
    if rway_ipc::MessageType::GetMarks as u32 == 5 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 5, got {}",
            rway_ipc::MessageType::GetMarks as u32
        ))
    }
}

fn test_message_type_get_bar_config() -> TestStatus {
    if rway_ipc::MessageType::GetBarConfig as u32 == 6 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 6, got {}",
            rway_ipc::MessageType::GetBarConfig as u32
        ))
    }
}

fn test_message_type_get_version() -> TestStatus {
    if rway_ipc::MessageType::GetVersion as u32 == 7 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 7, got {}",
            rway_ipc::MessageType::GetVersion as u32
        ))
    }
}

fn test_message_type_get_binding_modes() -> TestStatus {
    if rway_ipc::MessageType::GetBindingModes as u32 == 8 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 8, got {}",
            rway_ipc::MessageType::GetBindingModes as u32
        ))
    }
}

fn test_message_type_get_config() -> TestStatus {
    if rway_ipc::MessageType::GetConfig as u32 == 9 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 9, got {}",
            rway_ipc::MessageType::GetConfig as u32
        ))
    }
}

fn test_message_type_get_inputs() -> TestStatus {
    if rway_ipc::MessageType::GetInputs as u32 == 100 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 100, got {}",
            rway_ipc::MessageType::GetInputs as u32
        ))
    }
}

fn test_message_type_get_seats() -> TestStatus {
    if rway_ipc::MessageType::GetSeats as u32 == 101 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 101, got {}",
            rway_ipc::MessageType::GetSeats as u32
        ))
    }
}

// --- Group 2: EventType value verification ---

fn test_event_type_workspace() -> TestStatus {
    if rway_ipc::EventType::Workspace as u32 == 0x80000000 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000000, got 0x{:08X}",
            rway_ipc::EventType::Workspace as u32
        ))
    }
}

fn test_event_type_window() -> TestStatus {
    if rway_ipc::EventType::Window as u32 == 0x80000003 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000003, got 0x{:08X}",
            rway_ipc::EventType::Window as u32
        ))
    }
}

fn test_event_type_output() -> TestStatus {
    if rway_ipc::EventType::Output as u32 == 0x80000001 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000001, got 0x{:08X}",
            rway_ipc::EventType::Output as u32
        ))
    }
}

fn test_event_type_mode() -> TestStatus {
    if rway_ipc::EventType::Mode as u32 == 0x80000002 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000002, got 0x{:08X}",
            rway_ipc::EventType::Mode as u32
        ))
    }
}

fn test_event_type_binding() -> TestStatus {
    if rway_ipc::EventType::Binding as u32 == 0x80000005 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000005, got 0x{:08X}",
            rway_ipc::EventType::Binding as u32
        ))
    }
}

fn test_event_type_shutdown() -> TestStatus {
    if rway_ipc::EventType::Shutdown as u32 == 0x80000006 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000006, got 0x{:08X}",
            rway_ipc::EventType::Shutdown as u32
        ))
    }
}

// --- Group 3: Protocol round-trip ---

// --- Group 4: Additional EventType / MessageType values ---

fn test_event_type_tick() -> TestStatus {
    if rway_ipc::EventType::Tick as u32 == 0x80000007 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000007, got 0x{:08X}",
            rway_ipc::EventType::Tick as u32
        ))
    }
}

fn test_event_type_bar_config_update() -> TestStatus {
    if rway_ipc::EventType::BarConfigUpdate as u32 == 0x80000004 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000004, got 0x{:08X}",
            rway_ipc::EventType::BarConfigUpdate as u32
        ))
    }
}

fn test_event_type_bar_state_update() -> TestStatus {
    if rway_ipc::EventType::BarStateUpdate as u32 == 0x80000014 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000014, got 0x{:08X}",
            rway_ipc::EventType::BarStateUpdate as u32
        ))
    }
}

fn test_event_type_input() -> TestStatus {
    if rway_ipc::EventType::Input as u32 == 0x80000015 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 0x80000015, got 0x{:08X}",
            rway_ipc::EventType::Input as u32
        ))
    }
}

fn test_message_type_send_tick() -> TestStatus {
    if rway_ipc::MessageType::SendTick as u32 == 10 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 10, got {}",
            rway_ipc::MessageType::SendTick as u32
        ))
    }
}

fn test_message_type_sync() -> TestStatus {
    if rway_ipc::MessageType::Sync as u32 == 11 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "expected 11, got {}",
            rway_ipc::MessageType::Sync as u32
        ))
    }
}

fn test_encode_decode_roundtrip() -> TestStatus {
    let msg = rway_ipc::encode_message(7, b"hello");
    if msg.len() < 14 {
        return TestStatus::Fail("message too short".into());
    }
    let mut header = [0u8; 14];
    header.copy_from_slice(&msg[..14]);
    match rway_ipc::decode_header(&header) {
        Ok((payload_len, msg_type)) => {
            if msg_type == 7 && payload_len == 5 {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("type={msg_type} len={payload_len}"))
            }
        }
        Err(e) => TestStatus::Fail(format!("decode error: {e:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_tests_register_without_panic() {
        let mut h = Harness::new();
        register(&mut h);
        let report = h.run();
        assert!(report.total > 0);
    }
}
