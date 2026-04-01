// IPC 事件订阅系统
//
// 负责解析客户端的 Subscribe 请求（JSON 数组形式），
// 以及将服务端事件序列化为 Sway 兼容的 JSON 格式。

use serde::Serialize;

use crate::commands::{TreeNode, WorkspaceInfo};

/// 事件订阅类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubscriptionType {
    Workspace,
    Output,
    Mode,
    Window,
    Binding,
    Shutdown,
    Tick,
    BarConfigUpdate,
    BarStateUpdate,
    Input,
}

/// 工作区事件（对应 workspace 事件订阅）
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceEvent {
    /// 事件类型："focus"、"init"、"empty"、"urgent"、"reload"、"rename"、"move"
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<WorkspaceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old: Option<WorkspaceInfo>,
}

/// 窗口事件（对应 window 事件订阅）
#[derive(Debug, Clone, Serialize)]
pub struct WindowEvent {
    /// 事件类型："new"、"close"、"focus"、"title"、"fullscreen_mode"、"move"、"floating"、"urgent"、"mark"
    pub change: String,
    pub container: TreeNode,
}

/// 解析 Subscribe 请求的 JSON payload
///
/// 格式：`["workspace", "window", ...]`
///
/// # 参数
/// - `payload`: JSON 字符串
///
/// # 返回
/// 订阅类型列表，或 JSON 解析错误
pub fn parse_subscribe_payload(payload: &str) -> Result<Vec<SubscriptionType>, serde_json::Error> {
    let names: Vec<String> = serde_json::from_str(payload)?;
    let mut subs = Vec::with_capacity(names.len());

    for name in names {
        let sub = match name.as_str() {
            "workspace" => SubscriptionType::Workspace,
            "output" => SubscriptionType::Output,
            "mode" => SubscriptionType::Mode,
            "window" => SubscriptionType::Window,
            "binding" => SubscriptionType::Binding,
            "shutdown" => SubscriptionType::Shutdown,
            "tick" => SubscriptionType::Tick,
            "barconfig_update" => SubscriptionType::BarConfigUpdate,
            "bar_state_update" => SubscriptionType::BarStateUpdate,
            "input" => SubscriptionType::Input,
            // 未知事件类型静默忽略（Sway 行为一致）
            _ => continue,
        };
        subs.push(sub);
    }

    Ok(subs)
}

// ─────────────────────────────────────────────────────────────────────────────
// 单元测试（TDD：先写测试，后写实现）
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{IpcMode, IpcRect};
    use serde_json::to_value;

    // ── parse_subscribe_payload ───────────────────────────────────────────────

    /// 测试解析单个订阅类型
    #[test]
    fn test_parse_subscribe_single_workspace() {
        let result = parse_subscribe_payload(r#"["workspace"]"#).unwrap();
        assert_eq!(result, vec![SubscriptionType::Workspace]);
    }

    /// 测试解析多个订阅类型
    #[test]
    fn test_parse_subscribe_multiple() {
        let result =
            parse_subscribe_payload(r#"["workspace", "window", "output"]"#).unwrap();
        assert_eq!(
            result,
            vec![
                SubscriptionType::Workspace,
                SubscriptionType::Window,
                SubscriptionType::Output,
            ]
        );
    }

    /// 测试解析所有已知订阅类型
    #[test]
    fn test_parse_subscribe_all_known_types() {
        let payload = r#"["workspace","output","mode","window","binding","shutdown","tick","barconfig_update","bar_state_update","input"]"#;
        let result = parse_subscribe_payload(payload).unwrap();

        assert_eq!(result.len(), 10);
        assert!(result.contains(&SubscriptionType::Workspace));
        assert!(result.contains(&SubscriptionType::Output));
        assert!(result.contains(&SubscriptionType::Mode));
        assert!(result.contains(&SubscriptionType::Window));
        assert!(result.contains(&SubscriptionType::Binding));
        assert!(result.contains(&SubscriptionType::Shutdown));
        assert!(result.contains(&SubscriptionType::Tick));
        assert!(result.contains(&SubscriptionType::BarConfigUpdate));
        assert!(result.contains(&SubscriptionType::BarStateUpdate));
        assert!(result.contains(&SubscriptionType::Input));
    }

    /// 测试空数组返回空列表
    #[test]
    fn test_parse_subscribe_empty_array() {
        let result = parse_subscribe_payload("[]").unwrap();
        assert!(result.is_empty());
    }

    /// 测试未知类型被忽略（与 Sway 行为一致）
    #[test]
    fn test_parse_subscribe_unknown_type_ignored() {
        let result =
            parse_subscribe_payload(r#"["workspace", "unknown_event", "window"]"#).unwrap();
        // unknown_event 应被忽略
        assert_eq!(
            result,
            vec![SubscriptionType::Workspace, SubscriptionType::Window]
        );
    }

    /// 测试无效 JSON 返回解析错误
    #[test]
    fn test_parse_subscribe_invalid_json() {
        let result = parse_subscribe_payload("not json");
        assert!(result.is_err());
    }

    /// 测试非数组 JSON 返回解析错误
    #[test]
    fn test_parse_subscribe_non_array_json() {
        let result = parse_subscribe_payload(r#"{"workspace": true}"#);
        assert!(result.is_err());
    }

    /// 测试 JSON 数组中含数字类型返回错误
    #[test]
    fn test_parse_subscribe_array_with_non_string_elements() {
        let result = parse_subscribe_payload(r#"[1, 2, 3]"#);
        assert!(result.is_err());
    }

    /// 测试只有未知类型的情况（全部忽略后返回空列表）
    #[test]
    fn test_parse_subscribe_all_unknown_returns_empty() {
        let result =
            parse_subscribe_payload(r#"["foo", "bar", "baz"]"#).unwrap();
        assert!(result.is_empty());
    }

    /// 测试空字符串返回解析错误
    #[test]
    fn test_parse_subscribe_empty_string() {
        let result = parse_subscribe_payload("");
        assert!(result.is_err());
    }

    // ── WorkspaceEvent ────────────────────────────────────────────────────────

    fn make_zero_rect() -> IpcRect {
        IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    fn make_test_workspace() -> WorkspaceInfo {
        WorkspaceInfo {
            id: 1,
            num: 1,
            name: "1".to_string(),
            visible: true,
            focused: true,
            urgent: false,
            output: "HDMI-A-1".to_string(),
            rect: make_zero_rect(),
        }
    }

    /// 测试 WorkspaceEvent focus 事件序列化
    #[test]
    fn test_workspace_event_focus_serialization() {
        let event = WorkspaceEvent {
            change: "focus".to_string(),
            current: Some(make_test_workspace()),
            old: None,
        };
        let json = to_value(&event).unwrap();

        assert_eq!(json["change"], "focus");
        assert!(json["current"].is_object());
        // old 为 None 时不序列化
        assert!(json.get("old").is_none());
    }

    /// 测试 WorkspaceEvent init 事件序列化
    #[test]
    fn test_workspace_event_init_serialization() {
        let event = WorkspaceEvent {
            change: "init".to_string(),
            current: Some(make_test_workspace()),
            old: None,
        };
        let json = to_value(&event).unwrap();

        assert_eq!(json["change"], "init");
        assert_eq!(json["current"]["name"], "1");
    }

    /// 测试 WorkspaceEvent 含 old 字段时序列化
    #[test]
    fn test_workspace_event_with_old_workspace() {
        let mut old_ws = make_test_workspace();
        old_ws.name = "2".to_string();
        old_ws.focused = false;

        let event = WorkspaceEvent {
            change: "focus".to_string(),
            current: Some(make_test_workspace()),
            old: Some(old_ws),
        };
        let json = to_value(&event).unwrap();

        assert_eq!(json["old"]["name"], "2");
        assert_eq!(json["current"]["name"], "1");
    }

    /// 测试 WorkspaceEvent empty 事件（current 和 old 都为 None）
    #[test]
    fn test_workspace_event_empty_change() {
        let event = WorkspaceEvent {
            change: "empty".to_string(),
            current: None,
            old: None,
        };
        let json = to_value(&event).unwrap();

        assert_eq!(json["change"], "empty");
        assert!(json.get("current").is_none());
        assert!(json.get("old").is_none());
    }

    // ── WindowEvent ───────────────────────────────────────────────────────────

    fn make_test_tree_node() -> TreeNode {
        TreeNode {
            id: 100,
            name: Some("alacritty".to_string()),
            node_type: "con".to_string(),
            layout: "none".to_string(),
            focused: true,
            urgent: false,
            rect: IpcRect {
                x: 0,
                y: 0,
                width: 960,
                height: 1080,
            },
            window_rect: make_zero_rect(),
            deco_rect: make_zero_rect(),
            geometry: make_zero_rect(),
            nodes: vec![],
            floating_nodes: vec![],
            focus: vec![],
            app_id: Some("Alacritty".to_string()),
            window: None,
        }
    }

    /// 测试 WindowEvent new 事件序列化
    #[test]
    fn test_window_event_new_serialization() {
        let event = WindowEvent {
            change: "new".to_string(),
            container: make_test_tree_node(),
        };
        let json = to_value(&event).unwrap();

        assert_eq!(json["change"], "new");
        assert!(json["container"].is_object());
        assert_eq!(json["container"]["id"], 100);
        assert_eq!(json["container"]["type"], "con");
    }

    /// 测试 WindowEvent close 事件序列化
    #[test]
    fn test_window_event_close_serialization() {
        let event = WindowEvent {
            change: "close".to_string(),
            container: make_test_tree_node(),
        };
        let json = to_value(&event).unwrap();
        assert_eq!(json["change"], "close");
    }

    /// 测试 WindowEvent focus 事件序列化
    #[test]
    fn test_window_event_focus_serialization() {
        let mut node = make_test_tree_node();
        node.focused = true;
        let event = WindowEvent {
            change: "focus".to_string(),
            container: node,
        };
        let json = to_value(&event).unwrap();

        assert_eq!(json["change"], "focus");
        assert_eq!(json["container"]["focused"], true);
    }

    /// 测试 WindowEvent 中 app_id 字段正确序列化
    #[test]
    fn test_window_event_container_app_id() {
        let event = WindowEvent {
            change: "new".to_string(),
            container: make_test_tree_node(),
        };
        let json = to_value(&event).unwrap();
        assert_eq!(json["container"]["app_id"], "Alacritty");
    }

    // ── SubscriptionType 相等性测试 ───────────────────────────────────────────

    /// 测试 SubscriptionType 相等性比较
    #[test]
    fn test_subscription_type_equality() {
        assert_eq!(SubscriptionType::Workspace, SubscriptionType::Workspace);
        assert_ne!(SubscriptionType::Workspace, SubscriptionType::Window);
    }

    /// 测试 SubscriptionType 克隆
    #[test]
    fn test_subscription_type_clone() {
        let original = SubscriptionType::Window;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    /// 测试 SubscriptionType 在 HashSet 中使用
    #[test]
    fn test_subscription_type_in_hashset() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SubscriptionType::Workspace);
        set.insert(SubscriptionType::Workspace); // 重复插入
        set.insert(SubscriptionType::Window);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&SubscriptionType::Workspace));
        assert!(set.contains(&SubscriptionType::Window));
    }
}
