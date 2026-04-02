// IPC 命令响应类型定义
//
// 所有结构体的 JSON 字段名必须与 Sway IPC 的格式完全一致，
// 确保 swaymsg、waybar 等工具可以直接解析。

use serde::Serialize;

/// 命令执行结果
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommandResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 工作区信息（对应 sway IPC GetWorkspaces 响应）
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkspaceInfo {
    pub id: i64,
    pub num: i32,
    pub name: String,
    pub visible: bool,
    pub focused: bool,
    pub urgent: bool,
    pub output: String,
    pub rect: IpcRect,
}

/// 输出（显示器）信息（对应 sway IPC GetOutputs 响应）
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OutputInfo {
    pub id: i64,
    pub name: String,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub active: bool,
    pub primary: bool,
    pub scale: f64,
    pub transform: String,
    pub current_workspace: Option<String>,
    pub rect: IpcRect,
    pub current_mode: IpcMode,
}

/// 矩形区域（位置+尺寸），Sway IPC 中广泛使用
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IpcRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// 显示器分辨率模式
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IpcMode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

/// 容器树节点（对应 sway IPC GetTree 响应）
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TreeNode {
    pub id: i64,
    pub name: Option<String>,
    /// 节点类型："root"、"output"、"workspace"、"con"、"floating_con"
    #[serde(rename = "type")]
    pub node_type: String,
    /// 布局方式："splith"、"splitv"、"tabbed"、"stacked"、"output"、"none"
    pub layout: String,
    pub focused: bool,
    pub urgent: bool,
    pub rect: IpcRect,
    pub window_rect: IpcRect,
    pub deco_rect: IpcRect,
    pub geometry: IpcRect,
    pub nodes: Vec<TreeNode>,
    pub floating_nodes: Vec<TreeNode>,
    pub focus: Vec<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<i64>,
}

/// 版本信息（对应 sway IPC GetVersion 响应）
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct VersionInfo {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub human_readable: String,
    pub loaded_config_file_name: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// 单元测试（TDD：先写测试，后写实现）
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    // ── CommandResult ─────────────────────────────────────────────────────────

    /// 测试成功命令结果的 JSON 序列化
    #[test]
    fn test_command_result_success_serialization() {
        let result = CommandResult {
            success: true,
            error: None,
        };
        let json = to_value(&result).unwrap();

        assert_eq!(json["success"], true);
        // error 字段为 None 时，skip_serializing_if 应该省略该字段
        assert!(
            json.get("error").is_none(),
            "error 字段为 None 时不应序列化"
        );
    }

    /// 测试失败命令结果的 JSON 序列化
    #[test]
    fn test_command_result_failure_serialization() {
        let result = CommandResult {
            success: false,
            error: Some("未知命令".to_string()),
        };
        let json = to_value(&result).unwrap();

        assert_eq!(json["success"], false);
        assert_eq!(json["error"], "未知命令");
    }

    /// 测试命令结果数组的序列化（Sway 返回的是数组）
    #[test]
    fn test_command_result_array_serialization() {
        let results = vec![
            CommandResult {
                success: true,
                error: None,
            },
            CommandResult {
                success: false,
                error: Some("error".to_string()),
            },
        ];
        let json = to_value(&results).unwrap();

        assert!(json.is_array());
        assert_eq!(json[0]["success"], true);
        assert_eq!(json[1]["success"], false);
        assert_eq!(json[1]["error"], "error");
    }

    // ── WorkspaceInfo ─────────────────────────────────────────────────────────

    fn make_test_rect() -> IpcRect {
        IpcRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    /// 测试工作区信息的 JSON 序列化字段名称与 Sway 兼容
    #[test]
    fn test_workspace_info_serialization_field_names() {
        let ws = WorkspaceInfo {
            id: 1,
            num: 1,
            name: "1".to_string(),
            visible: true,
            focused: true,
            urgent: false,
            output: "HDMI-A-1".to_string(),
            rect: make_test_rect(),
        };
        let json = to_value(&ws).unwrap();

        // 验证所有字段名与 Sway IPC 规范匹配
        assert!(json.get("id").is_some());
        assert!(json.get("num").is_some());
        assert!(json.get("name").is_some());
        assert!(json.get("visible").is_some());
        assert!(json.get("focused").is_some());
        assert!(json.get("urgent").is_some());
        assert!(json.get("output").is_some());
        assert!(json.get("rect").is_some());
    }

    /// 测试工作区信息的 JSON 值正确
    #[test]
    fn test_workspace_info_serialization_values() {
        let ws = WorkspaceInfo {
            id: 42,
            num: 3,
            name: "代码".to_string(),
            visible: false,
            focused: false,
            urgent: true,
            output: "DP-1".to_string(),
            rect: IpcRect {
                x: 1920,
                y: 0,
                width: 2560,
                height: 1440,
            },
        };
        let json = to_value(&ws).unwrap();

        assert_eq!(json["id"], 42);
        assert_eq!(json["num"], 3);
        assert_eq!(json["name"], "代码");
        assert_eq!(json["visible"], false);
        assert_eq!(json["focused"], false);
        assert_eq!(json["urgent"], true);
        assert_eq!(json["output"], "DP-1");
        assert_eq!(json["rect"]["x"], 1920);
        assert_eq!(json["rect"]["width"], 2560);
    }

    // ── IpcRect ───────────────────────────────────────────────────────────────

    /// 测试 IpcRect 字段名与 Sway 规范匹配
    #[test]
    fn test_ipc_rect_field_names() {
        let rect = IpcRect {
            x: 10,
            y: 20,
            width: 300,
            height: 400,
        };
        let json = to_value(&rect).unwrap();

        assert_eq!(json["x"], 10);
        assert_eq!(json["y"], 20);
        assert_eq!(json["width"], 300);
        assert_eq!(json["height"], 400);
    }

    /// 测试 IpcRect 负坐标（多显示器布局可能有负坐标）
    #[test]
    fn test_ipc_rect_negative_coordinates() {
        let rect = IpcRect {
            x: -1920,
            y: -1080,
            width: 1920,
            height: 1080,
        };
        let json = to_value(&rect).unwrap();

        assert_eq!(json["x"], -1920);
        assert_eq!(json["y"], -1080);
    }

    // ── OutputInfo ────────────────────────────────────────────────────────────

    /// 测试输出信息序列化（活跃状态）
    #[test]
    fn test_output_info_active_serialization() {
        let output = OutputInfo {
            id: 1,
            name: "HDMI-A-1".to_string(),
            make: "Samsung".to_string(),
            model: "C27F390".to_string(),
            serial: "HTPK900123".to_string(),
            active: true,
            primary: true,
            scale: 1.0,
            transform: "normal".to_string(),
            current_workspace: Some("1".to_string()),
            rect: make_test_rect(),
            current_mode: IpcMode {
                width: 1920,
                height: 1080,
                refresh: 60000,
            },
        };
        let json = to_value(&output).unwrap();

        assert_eq!(json["name"], "HDMI-A-1");
        assert_eq!(json["active"], true);
        assert_eq!(json["primary"], true);
        assert_eq!(json["scale"], 1.0);
        assert_eq!(json["transform"], "normal");
        assert_eq!(json["current_workspace"], "1");
        assert_eq!(json["current_mode"]["width"], 1920);
        assert_eq!(json["current_mode"]["refresh"], 60000);
    }

    /// 测试输出信息序列化（非活跃状态，current_workspace 为 None）
    #[test]
    fn test_output_info_inactive_serialization() {
        let output = OutputInfo {
            id: 2,
            name: "DP-1".to_string(),
            make: "".to_string(),
            model: "".to_string(),
            serial: "".to_string(),
            active: false,
            primary: false,
            scale: 1.0,
            transform: "normal".to_string(),
            current_workspace: None,
            rect: make_test_rect(),
            current_mode: IpcMode {
                width: 0,
                height: 0,
                refresh: 0,
            },
        };
        let json = to_value(&output).unwrap();

        assert_eq!(json["active"], false);
        // current_workspace 为 None 应序列化为 null（OutputInfo 不使用 skip_serializing_if）
        assert!(json["current_workspace"].is_null());
    }

    /// 测试 HiDPI 缩放比例序列化
    #[test]
    fn test_output_info_hidpi_scale() {
        let output = OutputInfo {
            id: 1,
            name: "eDP-1".to_string(),
            make: "".to_string(),
            model: "".to_string(),
            serial: "".to_string(),
            active: true,
            primary: true,
            scale: 2.0,
            transform: "normal".to_string(),
            current_workspace: Some("1".to_string()),
            rect: IpcRect {
                x: 0,
                y: 0,
                width: 2560,
                height: 1600,
            },
            current_mode: IpcMode {
                width: 2560,
                height: 1600,
                refresh: 60000,
            },
        };
        let json = to_value(&output).unwrap();
        assert_eq!(json["scale"], 2.0);
    }

    // ── TreeNode ──────────────────────────────────────────────────────────────

    fn make_zero_rect() -> IpcRect {
        IpcRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    /// 测试 TreeNode 中 "type" 字段 rename 生效
    #[test]
    fn test_tree_node_type_field_rename() {
        let node = TreeNode {
            id: 1,
            name: None,
            node_type: "root".to_string(),
            layout: "output".to_string(),
            focused: false,
            urgent: false,
            rect: make_zero_rect(),
            window_rect: make_zero_rect(),
            deco_rect: make_zero_rect(),
            geometry: make_zero_rect(),
            nodes: vec![],
            floating_nodes: vec![],
            focus: vec![],
            app_id: None,
            window: None,
        };
        let json = to_value(&node).unwrap();

        // 必须是 "type" 而不是 "node_type"
        assert!(json.get("type").is_some(), "字段应重命名为 'type'");
        assert!(json.get("node_type").is_none(), "不应出现 'node_type' 字段");
        assert_eq!(json["type"], "root");
    }

    /// 测试 TreeNode 中 app_id/window 为 None 时不序列化
    #[test]
    fn test_tree_node_optional_fields_skip_when_none() {
        let node = TreeNode {
            id: 1,
            name: None,
            node_type: "workspace".to_string(),
            layout: "splith".to_string(),
            focused: false,
            urgent: false,
            rect: make_zero_rect(),
            window_rect: make_zero_rect(),
            deco_rect: make_zero_rect(),
            geometry: make_zero_rect(),
            nodes: vec![],
            floating_nodes: vec![],
            focus: vec![],
            app_id: None,
            window: None,
        };
        let json = to_value(&node).unwrap();

        assert!(json.get("app_id").is_none(), "app_id 为 None 时不应序列化");
        assert!(json.get("window").is_none(), "window 为 None 时不应序列化");
    }

    /// 测试 TreeNode 嵌套子节点序列化
    #[test]
    fn test_tree_node_nested_children() {
        let child = TreeNode {
            id: 2,
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
        };

        let parent = TreeNode {
            id: 1,
            name: Some("1".to_string()),
            node_type: "workspace".to_string(),
            layout: "splith".to_string(),
            focused: true,
            urgent: false,
            rect: make_test_rect(),
            window_rect: make_zero_rect(),
            deco_rect: make_zero_rect(),
            geometry: make_zero_rect(),
            nodes: vec![child],
            floating_nodes: vec![],
            focus: vec![2],
            app_id: None,
            window: None,
        };
        let json = to_value(&parent).unwrap();

        assert_eq!(json["nodes"].as_array().unwrap().len(), 1);
        assert_eq!(json["nodes"][0]["id"], 2);
        assert_eq!(json["nodes"][0]["app_id"], "Alacritty");
        assert_eq!(json["focus"][0], 2);
    }

    /// 测试浮动节点序列化
    #[test]
    fn test_tree_node_floating_nodes() {
        let floating = TreeNode {
            id: 99,
            name: Some("float_win".to_string()),
            node_type: "floating_con".to_string(),
            layout: "none".to_string(),
            focused: false,
            urgent: false,
            rect: IpcRect {
                x: 100,
                y: 100,
                width: 800,
                height: 600,
            },
            window_rect: make_zero_rect(),
            deco_rect: make_zero_rect(),
            geometry: make_zero_rect(),
            nodes: vec![],
            floating_nodes: vec![],
            focus: vec![],
            app_id: None,
            window: Some(123456),
        };
        let json = to_value(&floating).unwrap();

        assert_eq!(json["type"], "floating_con");
        assert_eq!(json["window"], 123456);
    }

    // ── VersionInfo ───────────────────────────────────────────────────────────

    /// 测试版本信息序列化
    #[test]
    fn test_version_info_serialization() {
        let version = VersionInfo {
            major: 1,
            minor: 8,
            patch: 0,
            human_readable: "1.8".to_string(),
            loaded_config_file_name: "/home/user/.config/sway/config".to_string(),
        };
        let json = to_value(&version).unwrap();

        assert_eq!(json["major"], 1);
        assert_eq!(json["minor"], 8);
        assert_eq!(json["patch"], 0);
        assert_eq!(json["human_readable"], "1.8");
        assert_eq!(
            json["loaded_config_file_name"],
            "/home/user/.config/sway/config"
        );
    }

    // ── 与 Sway 真实输出对比测试（使用已知正确的 JSON 格式）────────────────────

    /// 测试与 swaymsg -t get_version 输出格式兼容
    #[test]
    fn test_version_info_matches_sway_format() {
        // 这是 Sway 实际返回的 get_version 响应格式
        let expected = json!({
            "major": 1,
            "minor": 8,
            "patch": 0,
            "human_readable": "1.8",
            "loaded_config_file_name": "/etc/sway/config"
        });

        let version = VersionInfo {
            major: 1,
            minor: 8,
            patch: 0,
            human_readable: "1.8".to_string(),
            loaded_config_file_name: "/etc/sway/config".to_string(),
        };
        let actual = to_value(&version).unwrap();

        assert_eq!(actual, expected);
    }

    /// 测试 CommandResult 与 swaymsg 运行命令响应格式兼容
    #[test]
    fn test_command_result_matches_sway_format() {
        let expected_success = json!([{"success": true}]);
        let result = vec![CommandResult {
            success: true,
            error: None,
        }];
        let actual = to_value(&result).unwrap();

        assert_eq!(actual, expected_success);
    }

    /// 测试 IpcMode 字段名与 Sway 兼容
    #[test]
    fn test_ipc_mode_field_names() {
        let mode = IpcMode {
            width: 1920,
            height: 1080,
            refresh: 60000,
        };
        let json = to_value(&mode).unwrap();

        assert_eq!(json["width"], 1920);
        assert_eq!(json["height"], 1080);
        assert_eq!(json["refresh"], 60000);
    }
}
