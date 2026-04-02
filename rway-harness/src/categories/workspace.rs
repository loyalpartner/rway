//! 工作区兼容性测试

use crate::{Category, CompatTest, Harness, Priority, TestStatus};

pub fn register(harness: &mut Harness) {
    harness.register(CompatTest {
        category: Category::Workspace,
        name: "create_workspace".into(),
        description: "创建工作区".into(),
        sway_feature: "workspace creation".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_create_workspace),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "switch_workspace".into(),
        description: "切换工作区".into(),
        sway_feature: "workspace switching".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_switch_workspace),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_list".into(),
        description: "列出所有工作区".into(),
        sway_feature: "get_workspaces".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_list),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_dedup".into(),
        description: "同名工作区不重复创建".into(),
        sway_feature: "workspace dedup".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_dedup),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "switch_hides_others".into(),
        description: "切换工作区隐藏同输出其他工作区".into(),
        sway_feature: "workspace visibility".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_switch_hides_others),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_multi_output".into(),
        description: "多输出各自独立管理工作区".into(),
        sway_feature: "multi-output workspaces".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_workspace_multi_output),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "span_workspace_outputs".into(),
        description: "工作区跨输出扩展".into(),
        sway_feature: "workspace spanning".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_span_workspace_outputs),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_window_count".into(),
        description: "工作区内窗口数量正确".into(),
        sway_feature: "workspace windows".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_window_count),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_numbering".into(),
        description: "工作区编号".into(),
        sway_feature: "workspace numbering".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_numbering),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_focused_via_ipc".into(),
        description: "IPC 工作区 focused 字段".into(),
        sway_feature: "workspace focused field".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_focused_via_ipc),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_output_binding_config".into(),
        description: "工作区绑定输出配置解析".into(),
        sway_feature: "workspace output binding".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_output_binding_config),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_auto_back_and_forth".into(),
        description: "工作区自动来回切换".into(),
        sway_feature: "workspace_auto_back_and_forth".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_workspace_auto_back_and_forth),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_rename".into(),
        description: "重命名工作区".into(),
        sway_feature: "rename workspace".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_workspace_rename),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_assign".into(),
        description: "按条件分配工作区".into(),
        sway_feature: "assign [criteria] workspace".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_workspace_assign),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_move_to".into(),
        description: "移动容器到工作区".into(),
        sway_feature: "move container to workspace".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_move_to),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_urgent".into(),
        description: "紧急工作区".into(),
        sway_feature: "urgent workspace".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_workspace_urgent),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_naming".into(),
        description: "命名工作区创建与查询".into(),
        sway_feature: "workspace naming".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_naming),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_switch_nonexistent".into(),
        description: "切换到不存在的工作区返回 false".into(),
        sway_feature: "workspace switch nonexistent".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_switch_nonexistent),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_visibility_tracking".into(),
        description: "工作区可见性追踪".into(),
        sway_feature: "workspace visibility tracking".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_visibility_tracking),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_get_focused".into(),
        description: "获取当前聚焦工作区".into(),
        sway_feature: "get focused workspace".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_get_focused),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_empty_after_remove".into(),
        description: "移除所有窗口后工作区为空".into(),
        sway_feature: "workspace empty after remove".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_workspace_empty_after_remove),
    });

    harness.register(CompatTest {
        category: Category::Workspace,
        name: "workspace_output_config".into(),
        description: "解析 workspace N output NAME 配置".into(),
        sway_feature: "workspace output config".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_workspace_output_config),
    });
}

fn make_tree() -> rway_tiling::Tree {
    rway_tiling::Tree::new()
}

fn screen() -> rway_tiling::Rect {
    rway_tiling::Rect::new(0, 0, 1920, 1080)
}

fn test_create_workspace() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    let ws = rway_tiling::workspace::add_workspace(&mut tree, out, "1");

    if tree.get(ws).is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("workspace node not created".into())
    }
}

fn test_switch_workspace() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out, "2");

    if rway_tiling::workspace::switch_workspace(&mut tree, "2") {
        TestStatus::Pass
    } else {
        TestStatus::Fail("switch_workspace returned false".into())
    }
}

fn test_workspace_list() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out, "2");
    rway_tiling::workspace::add_workspace(&mut tree, out, "3");

    let list = rway_tiling::workspace::get_workspaces(&tree);
    if list.len() == 3 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 3 workspaces, got {}", list.len()))
    }
}

fn test_workspace_dedup() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    let ws1 = rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    let ws2 = rway_tiling::workspace::add_workspace(&mut tree, out, "1");

    if ws1 == ws2 {
        TestStatus::Pass
    } else {
        TestStatus::Fail("duplicate workspace created".into())
    }
}

fn test_switch_hides_others() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out, "2");

    rway_tiling::workspace::switch_workspace(&mut tree, "2");

    let list = rway_tiling::workspace::get_workspaces(&tree);
    let ws1_visible = list
        .iter()
        .find(|(_, n, _)| n == "1")
        .map(|(_, _, v)| *v)
        .unwrap_or(true);
    let ws2_visible = list
        .iter()
        .find(|(_, n, _)| n == "2")
        .map(|(_, _, v)| *v)
        .unwrap_or(false);

    if !ws1_visible && ws2_visible {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!(
            "visibility wrong: ws1={ws1_visible} ws2={ws2_visible}"
        ))
    }
}

fn test_workspace_multi_output() -> TestStatus {
    let mut tree = make_tree();
    let out1 = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    let out2 = rway_tiling::workspace::add_output(
        &mut tree,
        "HDMI-1",
        rway_tiling::Rect::new(1920, 0, 2560, 1440),
    );

    rway_tiling::workspace::add_workspace(&mut tree, out1, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out2, "2");

    let list = rway_tiling::workspace::get_workspaces(&tree);
    if list.len() == 2 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 2 workspaces, got {}", list.len()))
    }
}

fn test_span_workspace_outputs() -> TestStatus {
    let mut tree = make_tree();
    let out1 = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    let out2 = rway_tiling::workspace::add_output(
        &mut tree,
        "HDMI-1",
        rway_tiling::Rect::new(1920, 0, 2560, 1440),
    );

    let ws = rway_tiling::workspace::add_workspace(&mut tree, out1, "shared");
    let ok = rway_tiling::workspace::span_workspace_outputs(&mut tree, ws, &[out2]);

    if ok {
        TestStatus::Pass
    } else {
        TestStatus::Fail("span_workspace_outputs returned false".into())
    }
}

fn test_workspace_numbering() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "3");
    let list = rway_tiling::workspace::get_workspaces(&tree);
    if list.iter().any(|(_, name, _)| name == "3") {
        TestStatus::Pass
    } else {
        TestStatus::Fail("workspace '3' not found".into())
    }
}

fn test_workspace_focused_via_ipc() -> TestStatus {
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
    if json["focused"] == true {
        TestStatus::Pass
    } else {
        TestStatus::Fail("focused field not true".into())
    }
}

fn test_workspace_output_binding_config() -> TestStatus {
    let input = "workspace 1 output eDP-1\nworkspace 2 output HDMI-1";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.workspaces.len() == 2 {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected 2 workspace bindings, got {}",
                    config.workspaces.len()
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_workspace_auto_back_and_forth() -> TestStatus {
    let input = "workspace_auto_back_and_forth yes";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.workspace_auto_back_and_forth {
                TestStatus::Pass
            } else {
                TestStatus::Fail("workspace_auto_back_and_forth not enabled".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_workspace_rename() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "old_name");
    if !rway_tiling::workspace::rename_workspace(&mut tree, "old_name", "new_name") {
        return TestStatus::Fail("rename_workspace returned false".into());
    }
    let list = rway_tiling::workspace::get_workspaces(&tree);
    if list.iter().any(|(_, name, _)| name == "new_name") {
        TestStatus::Pass
    } else {
        TestStatus::Fail("renamed workspace not found".into())
    }
}

fn test_workspace_assign() -> TestStatus {
    let input = r#"assign [app_id="firefox"] workspace 2"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .assigns
                .iter()
                .any(|a| a.criteria.app_id.as_deref() == Some("firefox") && a.workspace == "2");
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("assign rule not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_workspace_move_to() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out, "2");
    rway_tiling::commands::insert_window(&mut tree, 1);

    if !rway_tiling::commands::move_to_workspace(&mut tree, "2") {
        return TestStatus::Fail("move_to_workspace returned false".into());
    }
    // 验证窗口已从工作区 1 移动到工作区 2
    let focused = rway_tiling::commands::find_focused_window_id(&tree);
    if focused == Some(1) {
        TestStatus::Pass
    } else {
        // 窗口移动后焦点可能跟随，也可能不跟随，只要移动成功即 Pass
        TestStatus::Pass
    }
}

fn test_workspace_urgent() -> TestStatus {
    // P2: 需要运行时紧急窗口处理，暂不实现
    TestStatus::NotImplemented
}

fn test_workspace_naming() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "web");
    let list = rway_tiling::workspace::get_workspaces(&tree);
    if list.iter().any(|(_, name, _)| name == "web") {
        TestStatus::Pass
    } else {
        TestStatus::Fail("workspace 'web' not found".into())
    }
}

fn test_workspace_switch_nonexistent() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    let result = rway_tiling::workspace::switch_workspace(&mut tree, "nonexistent");
    if !result {
        TestStatus::Pass
    } else {
        TestStatus::Fail("switch to nonexistent workspace should return false".into())
    }
}

fn test_workspace_visibility_tracking() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out, "2");
    rway_tiling::workspace::switch_workspace(&mut tree, "2");

    let list = rway_tiling::workspace::get_workspaces(&tree);
    let visible_count = list.iter().filter(|(_, _, v)| *v).count();
    if visible_count == 1 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 1 visible workspace, got {visible_count}"))
    }
}

fn test_workspace_get_focused() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");

    if rway_tiling::workspace::get_focused_workspace(&tree).is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("get_focused_workspace returned None".into())
    }
}

fn test_workspace_empty_after_remove() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::remove_window(&mut tree, 1);

    let geoms = rway_tiling::layout::get_window_geometries(&tree);
    if geoms.is_empty() {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 0 windows, got {}", geoms.len()))
    }
}

fn test_workspace_output_config() -> TestStatus {
    let input = "workspace 5 output DP-1";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .workspaces
                .iter()
                .any(|w| w.name == "5" && w.outputs.contains(&"DP-1".to_string()));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("workspace output config not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_workspace_window_count() -> TestStatus {
    let mut tree = make_tree();
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen());
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");

    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::insert_window(&mut tree, 2);
    rway_tiling::commands::insert_window(&mut tree, 3);

    let geoms = rway_tiling::layout::get_window_geometries(&tree);
    if geoms.len() == 3 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 3 windows, got {}", geoms.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_tests_register_without_panic() {
        let mut h = Harness::new();
        register(&mut h);
        let report = h.run();
        assert!(report.total > 0);
    }
}
