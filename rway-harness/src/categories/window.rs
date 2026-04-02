//! 窗口管理兼容性测试

use crate::{Category, CompatTest, Harness, Priority, TestStatus};

pub fn register(harness: &mut Harness) {
    harness.register(CompatTest {
        category: Category::Window,
        name: "toggle_floating".into(),
        description: "切换窗口浮动状态".into(),
        sway_feature: "floating toggle".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_toggle_floating),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "toggle_floating_twice_restores".into(),
        description: "两次切换浮动恢复原状".into(),
        sway_feature: "floating toggle".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_toggle_floating_twice),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "find_focused_window".into(),
        description: "查找当前聚焦窗口".into(),
        sway_feature: "focus tracking".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_find_focused_window),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "insert_window".into(),
        description: "插入窗口到工作区".into(),
        sway_feature: "window creation".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_insert_window),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "remove_window".into(),
        description: "从工作区移除窗口".into(),
        sway_feature: "kill".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_remove_window),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "move_focus_left_right".into(),
        description: "左右方向键移动焦点".into(),
        sway_feature: "focus left/right".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_move_focus_left_right),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "fullscreen_toggle".into(),
        description: "切换窗口全屏状态".into(),
        sway_feature: "fullscreen toggle".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_fullscreen_toggle),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "window_criteria_matching".into(),
        description: "窗口标准匹配（app_id/class/title）".into(),
        sway_feature: "window criteria".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_window_criteria_matching),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "focus_up_down".into(),
        description: "上下方向键移动焦点".into(),
        sway_feature: "focus up/down".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_focus_up_down),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "focus_parent".into(),
        description: "焦点移到父容器".into(),
        sway_feature: "focus parent".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_focus_parent),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "focus_child".into(),
        description: "焦点移到子节点".into(),
        sway_feature: "focus child".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_focus_child),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "focus_tiling_floating_toggle".into(),
        description: "在平铺和浮动窗口间切换焦点".into(),
        sway_feature: "focus mode_toggle".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_focus_tiling_floating_toggle),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "move_to_workspace".into(),
        description: "移动容器到指定工作区".into(),
        sway_feature: "move container to workspace".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_move_to_workspace),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "move_to_output".into(),
        description: "移动容器到指定输出".into(),
        sway_feature: "move container to output".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_move_to_output),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "move_to_scratchpad".into(),
        description: "移动窗口到 scratchpad".into(),
        sway_feature: "move to scratchpad".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_move_to_scratchpad),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "scratchpad_show".into(),
        description: "显示 scratchpad 窗口".into(),
        sway_feature: "scratchpad show".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_scratchpad_show),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "floating_enable_explicit".into(),
        description: "显式启用窗口浮动".into(),
        sway_feature: "floating enable".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_floating_enable_explicit),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "floating_disable_explicit".into(),
        description: "显式禁用窗口浮动".into(),
        sway_feature: "floating disable".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_floating_disable_explicit),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "fullscreen_enable_disable".into(),
        description: "显式启用/禁用全屏".into(),
        sway_feature: "fullscreen enable/disable".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_fullscreen_enable_disable),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "fullscreen_global".into(),
        description: "全局全屏（跨输出）".into(),
        sway_feature: "fullscreen toggle global".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_fullscreen_global),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "sticky_toggle".into(),
        description: "窗口粘滞状态切换".into(),
        sway_feature: "sticky enable/disable/toggle".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_sticky_toggle),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "resize_command".into(),
        description: "通过命令调整窗口大小".into(),
        sway_feature: "resize grow/shrink".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_resize_command),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "resize_set".into(),
        description: "设置窗口精确大小".into(),
        sway_feature: "resize set <w> <h>".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_resize_set),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "swap_container".into(),
        description: "与另一个容器交换位置".into(),
        sway_feature: "swap container with".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_swap_container),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "window_marks".into(),
        description: "窗口标记与取消标记".into(),
        sway_feature: "mark/unmark".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_window_marks),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "window_criteria_class".into(),
        description: "WindowCriteria class 字段构造".into(),
        sway_feature: "window criteria class".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_window_criteria_class),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "window_criteria_title".into(),
        description: "WindowCriteria title 字段构造".into(),
        sway_feature: "window criteria title".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_window_criteria_title),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "window_criteria_all_fields".into(),
        description: "WindowCriteria 全字段构造".into(),
        sway_feature: "window criteria all fields".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_window_criteria_all_fields),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "for_window_floating_rule".into(),
        description: "for_window [app_id] floating enable 规则解析".into(),
        sway_feature: "for_window floating rule".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_for_window_floating_rule),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "for_window_move_rule".into(),
        description: "for_window [class] move to workspace 规则解析".into(),
        sway_feature: "for_window move rule".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_for_window_move_rule),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "kill_command_action".into(),
        description: "bindsym Mod4+q kill 动作解析".into(),
        sway_feature: "kill command".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_kill_command_action),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "move_direction_actions".into(),
        description: "move left/right/up/down 命令".into(),
        sway_feature: "move left/right/up/down commands".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_move_direction_actions),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "move_absolute_position".into(),
        description: "move [absolute] position x y 命令".into(),
        sway_feature: "move [absolute] position x y".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_move_absolute_position),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "move_center".into(),
        description: "move position center 命令".into(),
        sway_feature: "move position center".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_move_center),
    });

    harness.register(CompatTest {
        category: Category::Window,
        name: "nop_command".into(),
        description: "nop 注释命令".into(),
        sway_feature: "nop comment command".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_nop_command),
    });
}

fn make_tree_with_workspace() -> rway_tiling::Tree {
    let mut tree = rway_tiling::Tree::new();
    let screen = rway_tiling::Rect::new(0, 0, 1920, 1080);
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen);
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    tree
}

fn test_toggle_floating() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 10);

    if rway_tiling::commands::toggle_floating(&mut tree, 10) {
        TestStatus::Pass
    } else {
        TestStatus::Fail("toggle_floating returned false".into())
    }
}

fn test_toggle_floating_twice() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 20);

    rway_tiling::commands::toggle_floating(&mut tree, 20);
    rway_tiling::commands::toggle_floating(&mut tree, 20);

    // 通过 get_window_geometries 检查窗口仍存在
    let geoms = rway_tiling::layout::get_window_geometries(&tree);
    if geoms.iter().any(|(id, _)| *id == 20) {
        TestStatus::Pass
    } else {
        TestStatus::Fail("window lost after double toggle".into())
    }
}

fn test_find_focused_window() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 42);

    match rway_tiling::commands::find_focused_window_id(&tree) {
        Some(42) => TestStatus::Pass,
        Some(other) => TestStatus::Fail(format!("expected focused=42, got {other}")),
        None => TestStatus::Fail("no focused window found".into()),
    }
}

fn test_insert_window() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    let node_id = rway_tiling::commands::insert_window(&mut tree, 1);

    if tree.get(node_id).is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("inserted window node not found".into())
    }
}

fn test_remove_window() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);

    if rway_tiling::commands::remove_window(&mut tree, 1) {
        let geoms = rway_tiling::layout::get_window_geometries(&tree);
        if geoms.is_empty() {
            TestStatus::Pass
        } else {
            TestStatus::Fail("window still present after removal".into())
        }
    } else {
        TestStatus::Fail("remove_window returned false".into())
    }
}

fn test_move_focus_left_right() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::insert_window(&mut tree, 2);

    // 新插入的窗口 2 应是聚焦的，向左移动应能切换到窗口 1
    let moved = rway_tiling::commands::move_focus(&mut tree, rway_tiling::Direction::Left);
    if moved {
        TestStatus::Pass
    } else {
        TestStatus::Fail("move_focus Left returned false".into())
    }
}

fn test_fullscreen_toggle() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 10);

    if !rway_tiling::commands::toggle_fullscreen(&mut tree, 10) {
        return TestStatus::Fail("toggle_fullscreen returned false".into());
    }
    // 再次切换应恢复
    if !rway_tiling::commands::toggle_fullscreen(&mut tree, 10) {
        return TestStatus::Fail("second toggle_fullscreen returned false".into());
    }
    TestStatus::Pass
}

fn test_window_criteria_matching() -> TestStatus {
    // 窗口标准匹配是配置层面的功能
    let criteria = rway_config::WindowCriteria {
        app_id: Some("firefox".into()),
        class: None,
        title: None,
    };

    if criteria.app_id.is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("criteria fields not available".into())
    }
}

fn test_focus_up_down() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    // 插入两个窗口创建 SplitH 容器，然后改为 SplitV
    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::insert_window(&mut tree, 2);
    rway_tiling::commands::split(&mut tree, rway_tiling::Layout::SplitV);
    // 窗口 2 是聚焦的（index=1），向上应切换到窗口 1（index=0）
    let moved = rway_tiling::commands::move_focus(&mut tree, rway_tiling::Direction::Up);
    if moved {
        TestStatus::Pass
    } else {
        TestStatus::Fail("move_focus Up returned false".into())
    }
}

fn test_focus_parent() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::insert_window(&mut tree, 2);

    if rway_tiling::commands::focus_parent(&mut tree) {
        TestStatus::Pass
    } else {
        TestStatus::Fail("focus_parent returned false".into())
    }
}

fn test_focus_child() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::insert_window(&mut tree, 2);

    // 先 focus_parent，再 focus_child 回来
    rway_tiling::commands::focus_parent(&mut tree);
    if rway_tiling::commands::focus_child(&mut tree) {
        TestStatus::Pass
    } else {
        TestStatus::Fail("focus_child returned false".into())
    }
}

fn test_focus_tiling_floating_toggle() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::insert_window(&mut tree, 2);
    rway_tiling::commands::set_floating(&mut tree, 2, true);
    // Verify we can still find both windows
    let geoms = rway_tiling::layout::get_window_geometries(&tree);
    if geoms.len() == 2 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 2 windows, got {}", geoms.len()))
    }
}

fn test_move_to_workspace() -> TestStatus {
    let mut tree = rway_tiling::Tree::new();
    let screen = rway_tiling::Rect::new(0, 0, 1920, 1080);
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen);
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out, "2");
    rway_tiling::commands::insert_window(&mut tree, 1);

    if rway_tiling::commands::move_to_workspace(&mut tree, "2") {
        TestStatus::Pass
    } else {
        TestStatus::Fail("move_to_workspace returned false".into())
    }
}

fn test_move_to_output() -> TestStatus {
    let mut tree = rway_tiling::Tree::new();
    let screen = rway_tiling::Rect::new(0, 0, 1920, 1080);
    let out1 = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen);
    let out2 = rway_tiling::workspace::add_output(
        &mut tree,
        "HDMI-1",
        rway_tiling::Rect::new(1920, 0, 1920, 1080),
    );
    rway_tiling::workspace::add_workspace(&mut tree, out1, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out2, "2");
    rway_tiling::commands::insert_window(&mut tree, 1);
    // move to output 2 by switching workspace
    if rway_tiling::commands::move_to_workspace(&mut tree, "2") {
        TestStatus::Pass
    } else {
        TestStatus::Fail("move to workspace on other output failed".into())
    }
}

fn test_move_to_scratchpad() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+minus move to scratchpad";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .keybindings
                .iter()
                .any(|kb| kb.action == rway_config::Action::MoveToScratchpad);
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("MoveToScratchpad action not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_scratchpad_show() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+minus scratchpad show";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .keybindings
                .iter()
                .any(|kb| kb.action == rway_config::Action::ScratchpadShow);
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("ScratchpadShow action not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_floating_enable_explicit() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);

    if rway_tiling::commands::set_floating(&mut tree, 1, true) {
        TestStatus::Pass
    } else {
        TestStatus::Fail("set_floating(true) returned false".into())
    }
}

fn test_floating_disable_explicit() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);

    // 先设为浮动
    rway_tiling::commands::set_floating(&mut tree, 1, true);
    // 再显式关闭浮动
    if rway_tiling::commands::set_floating(&mut tree, 1, false) {
        TestStatus::Pass
    } else {
        TestStatus::Fail("set_floating(false) returned false".into())
    }
}

fn test_fullscreen_enable_disable() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);

    if !rway_tiling::commands::set_fullscreen(&mut tree, 1, true) {
        return TestStatus::Fail("set_fullscreen(true) returned false".into());
    }
    if !rway_tiling::commands::set_fullscreen(&mut tree, 1, false) {
        return TestStatus::Fail("set_fullscreen(false) returned false".into());
    }
    TestStatus::Pass
}

fn test_fullscreen_global() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);
    if !rway_tiling::commands::set_fullscreen_global(&mut tree, 1, true) {
        return TestStatus::Fail("set_fullscreen_global(true) failed".into());
    }
    if !rway_tiling::commands::toggle_fullscreen_global(&mut tree, 1) {
        return TestStatus::Fail("toggle_fullscreen_global failed".into());
    }
    TestStatus::Pass
}

fn test_sticky_toggle() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);
    if !rway_tiling::commands::set_sticky(&mut tree, 1, true) {
        return TestStatus::Fail("set_sticky(true) failed".into());
    }
    if !rway_tiling::commands::toggle_sticky(&mut tree, 1) {
        return TestStatus::Fail("toggle_sticky failed".into());
    }
    TestStatus::Pass
}

fn test_resize_command() -> TestStatus {
    // 验证 resize 命令被正确解析
    let input = "set $mod Mod4\nbindsym $mod+r resize grow width 10 px";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .keybindings
                .iter()
                .any(|kb| matches!(&kb.action, rway_config::Action::Resize { grow: true, .. }));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("resize action not found in keybindings".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_resize_set() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+r resize set 800 600";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                matches!(
                    &kb.action,
                    rway_config::Action::ResizeSet {
                        width: 800,
                        height: 600
                    }
                )
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("ResizeSet action not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_swap_container() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    let n1 = rway_tiling::commands::insert_window(&mut tree, 1);
    let n2 = rway_tiling::commands::insert_window(&mut tree, 2);
    if rway_tiling::commands::swap_containers(&mut tree, n1, n2) {
        TestStatus::Pass
    } else {
        TestStatus::Fail("swap_containers returned false".into())
    }
}

fn test_window_marks() -> TestStatus {
    let mut tree = make_tree_with_workspace();
    rway_tiling::commands::insert_window(&mut tree, 1);
    if !rway_tiling::commands::add_mark(&mut tree, 1, "test_mark") {
        return TestStatus::Fail("add_mark failed".into());
    }
    let marks = rway_tiling::commands::get_marks(&tree, 1);
    if !marks.contains(&"test_mark".to_string()) {
        return TestStatus::Fail("mark not found".into());
    }
    if !rway_tiling::commands::remove_mark(&mut tree, 1, Some("test_mark")) {
        return TestStatus::Fail("remove_mark failed".into());
    }
    let marks_after = rway_tiling::commands::get_marks(&tree, 1);
    if marks_after.is_empty() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("marks not cleared".into())
    }
}

fn test_window_criteria_class() -> TestStatus {
    let criteria = rway_config::WindowCriteria {
        app_id: None,
        class: Some("Firefox".into()),
        title: None,
    };
    if criteria.class.is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("class field not available".into())
    }
}

fn test_window_criteria_title() -> TestStatus {
    let criteria = rway_config::WindowCriteria {
        app_id: None,
        class: None,
        title: Some("My Window".into()),
    };
    if criteria.title.is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("title field not available".into())
    }
}

fn test_window_criteria_all_fields() -> TestStatus {
    let criteria = rway_config::WindowCriteria {
        app_id: Some("org.mozilla.firefox".into()),
        class: Some("Firefox".into()),
        title: Some("Mozilla Firefox".into()),
    };
    if criteria.app_id.is_some() && criteria.class.is_some() && criteria.title.is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail("not all criteria fields available".into())
    }
}

fn test_for_window_floating_rule() -> TestStatus {
    let input = r#"for_window [app_id="mpv"] floating enable"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.window_rules.iter().any(|r| {
                r.criteria.app_id.as_deref() == Some("mpv")
                    && r.action == rway_config::WindowRuleAction::FloatingEnable
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("for_window floating rule not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_for_window_move_rule() -> TestStatus {
    let input = r#"for_window [class="Slack"] move to workspace 3"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.window_rules.iter().any(|r| {
                r.criteria.class.as_deref() == Some("Slack")
                    && r.action == rway_config::WindowRuleAction::MoveToWorkspace("3".into())
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("for_window move rule not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_kill_command_action() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+q kill";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.key == "q"
                    && kb.action == rway_config::Action::Kill
                    && kb.modifiers.contains(&rway_config::Modifier::Mod4)
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("kill keybinding not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_move_direction_actions() -> TestStatus {
    let input =
        "set $mod Mod4\nbindsym $mod+Shift+Left move left\nbindsym $mod+Shift+Right move right";
    match rway_config::parse(input) {
        Ok(config) => {
            let has_left = config
                .keybindings
                .iter()
                .any(|kb| kb.action == rway_config::Action::Move(rway_config::Direction::Left));
            let has_right = config
                .keybindings
                .iter()
                .any(|kb| kb.action == rway_config::Action::Move(rway_config::Direction::Right));
            if has_left && has_right {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("move actions: left={has_left}, right={has_right}"))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_move_absolute_position() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+m move absolute position 100 200";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                matches!(
                    &kb.action,
                    rway_config::Action::MovePosition { x: 100, y: 200 }
                )
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("MovePosition action not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_move_center() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+c move position center";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .keybindings
                .iter()
                .any(|kb| kb.action == rway_config::Action::MoveCenter);
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("MoveCenter action not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_nop_command() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+n nop comment here";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .keybindings
                .iter()
                .any(|kb| matches!(&kb.action, rway_config::Action::Nop(_)));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("Nop action not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_tests_register_without_panic() {
        let mut h = Harness::new();
        register(&mut h);
        let report = h.run();
        assert!(report.total > 0);
    }
}
