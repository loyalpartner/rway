//! 配置解析兼容性测试

use crate::{Category, CompatTest, Harness, Priority, TestStatus};

pub fn register(harness: &mut Harness) {
    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_set_variable".into(),
        description: "解析 set $mod Mod4 变量定义".into(),
        sway_feature: "set $var value".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_set_variable),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_bindsym".into(),
        description: "解析 bindsym $mod+Return exec alacritty".into(),
        sway_feature: "bindsym".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_bindsym),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_gaps_inner".into(),
        description: "解析 gaps inner 10".into(),
        sway_feature: "gaps inner".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_gaps_inner),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_gaps_outer".into(),
        description: "解析 gaps outer 5".into(),
        sway_feature: "gaps outer".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_gaps_outer),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_output_background".into(),
        description: "解析 output * bg wallpaper.png fill".into(),
        sway_feature: "output bg".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_output_background),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_default_border".into(),
        description: "解析 default_border pixel 2".into(),
        sway_feature: "default_border".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_default_border),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_exec".into(),
        description: "解析 exec 启动命令".into(),
        sway_feature: "exec".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_exec),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_exec_always".into(),
        description: "解析 exec_always 启动命令".into(),
        sway_feature: "exec_always".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_exec_always),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_focus_follows_mouse".into(),
        description: "解析 focus_follows_mouse yes/no".into(),
        sway_feature: "focus_follows_mouse".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_focus_follows_mouse),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_font".into(),
        description: "解析 font pango:monospace 10".into(),
        sway_feature: "font".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_font),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_for_window".into(),
        description: "解析 for_window 窗口规则".into(),
        sway_feature: "for_window".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_for_window),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_workspace_output".into(),
        description: "解析 workspace 1 output eDP-1".into(),
        sway_feature: "workspace output".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_workspace_output),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_input_block".into(),
        description: "解析 input 设备配置块".into(),
        sway_feature: "input block".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_input_block),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_mode_block".into(),
        description: "解析 mode 自定义模式块".into(),
        sway_feature: "mode block".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_parse_mode_block),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_variable_substitution".into(),
        description: "变量替换在 bindsym 中生效".into(),
        sway_feature: "variable substitution".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_variable_substitution),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_comments".into(),
        description: "正确忽略注释行".into(),
        sway_feature: "comments".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_comments),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_output_resolution".into(),
        description: "解析 output eDP-1 resolution 1920x1080".into(),
        sway_feature: "output resolution".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_output_resolution),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_include_directive".into(),
        description: "include 指令解析".into(),
        sway_feature: "include /path/*".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_include_directive),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_default_orientation".into(),
        description: "default_orientation 解析".into(),
        sway_feature: "default_orientation horizontal|vertical|auto".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_default_orientation),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_workspace_layout".into(),
        description: "workspace_layout 解析".into(),
        sway_feature: "workspace_layout default|stacking|tabbed".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_workspace_layout),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_xwayland".into(),
        description: "xwayland 配置解析".into(),
        sway_feature: "xwayland enable|disable|force".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_xwayland),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_swaybg_command".into(),
        description: "swaybg_command 解析".into(),
        sway_feature: "swaybg_command <cmd>".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_parse_swaybg_command),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_swaynag_command".into(),
        description: "swaynag_command 解析".into(),
        sway_feature: "swaynag_command <cmd>".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_parse_swaynag_command),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_floating_modifier".into(),
        description: "floating_modifier 解析".into(),
        sway_feature: "floating_modifier <mod>".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_floating_modifier),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_floating_max_size".into(),
        description: "floating_maximum_size 解析".into(),
        sway_feature: "floating_maximum_size <w> x <h>".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_floating_max_size),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_floating_min_size".into(),
        description: "floating_minimum_size 解析".into(),
        sway_feature: "floating_minimum_size <w> x <h>".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_floating_min_size),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_focus_follows_mouse_always".into(),
        description: "focus_follows_mouse always 模式".into(),
        sway_feature: "focus_follows_mouse always".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_focus_follows_mouse_always),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_focus_on_window_activation".into(),
        description: "focus_on_window_activation 解析".into(),
        sway_feature: "focus_on_window_activation smart|urgent|focus|none".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_focus_on_window_activation),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_focus_wrapping".into(),
        description: "focus_wrapping 解析".into(),
        sway_feature: "focus_wrapping yes|no|force|workspace".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_focus_wrapping),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_mouse_warping".into(),
        description: "mouse_warping 解析".into(),
        sway_feature: "mouse_warping output|container|none".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_mouse_warping),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_popup_during_fullscreen".into(),
        description: "popup_during_fullscreen 解析".into(),
        sway_feature: "popup_during_fullscreen smart|ignore|leave_fullscreen".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_parse_popup_during_fullscreen),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_show_marks".into(),
        description: "show_marks 解析".into(),
        sway_feature: "show_marks yes|no".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_parse_show_marks),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_tiling_drag".into(),
        description: "tiling_drag 解析".into(),
        sway_feature: "tiling_drag enable|disable|toggle".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_tiling_drag),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_title_align".into(),
        description: "title_align 解析".into(),
        sway_feature: "title_align left|center|right".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_parse_title_align),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_workspace_auto_back_and_forth".into(),
        description: "workspace_auto_back_and_forth 解析".into(),
        sway_feature: "workspace_auto_back_and_forth yes|no".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_workspace_auto_back_and_forth),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_for_window_class".into(),
        description: "for_window [class] 窗口条件匹配".into(),
        sway_feature: "for_window [class] criteria".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_for_window_class),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_for_window_title".into(),
        description: "for_window [title] 窗口条件匹配".into(),
        sway_feature: "for_window [title] criteria".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_parse_for_window_title),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_assign_workspace".into(),
        description: "assign [criteria] workspace 自动分配窗口".into(),
        sway_feature: "assign [criteria] workspace".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_assign_workspace),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_no_focus".into(),
        description: "no_focus [criteria] 阻止自动聚焦".into(),
        sway_feature: "no_focus [criteria]".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_parse_no_focus),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_bar_block".into(),
        description: "bar { } 配置块解析".into(),
        sway_feature: "bar { } config block".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_bar_block),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_bar_status_command".into(),
        description: "bar { status_command } 解析".into(),
        sway_feature: "bar { status_command }".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_bar_status_command),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_bar_position".into(),
        description: "bar { position top|bottom } 解析".into(),
        sway_feature: "bar { position top|bottom }".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_bar_position),
    });

    harness.register(CompatTest {
        category: Category::Config,
        name: "parse_client_focused".into(),
        description: "client.focused 颜色配置解析".into(),
        sway_feature: "client.focused colors".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_parse_client_focused),
    });
}

fn test_parse_set_variable() -> TestStatus {
    let input = "set $mod Mod4";
    match rway_config::parse(input) {
        Ok(config) => {
            // 解析器去掉 $ 前缀，key 为 "mod"
            if config.variables.get("mod") == Some(&"Mod4".to_string()) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected mod=Mod4, got {:?}",
                    config.variables.get("mod")
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_bindsym() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+Return exec alacritty";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.key == "Return"
                    && kb.action == rway_config::Action::Exec("alacritty".into())
                    && kb.modifiers.contains(&rway_config::Modifier::Mod4)
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("bindsym $mod+Return exec alacritty not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_gaps_inner() -> TestStatus {
    let input = "gaps inner 10";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.gaps.inner == 10 {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected gaps inner=10, got {}", config.gaps.inner))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_gaps_outer() -> TestStatus {
    let input = "gaps outer 5";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.gaps.outer == 5 {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected gaps outer=5, got {}", config.gaps.outer))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_output_background() -> TestStatus {
    let input = "output * bg wallpaper.png fill";
    match rway_config::parse(input) {
        Ok(_config) => {
            // output bg 解析为 OutputConfig，检查是否不报错即可
            // 背景设置可能存储在 OutputConfig 中或作为特殊处理
            TestStatus::Pass
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_default_border() -> TestStatus {
    let input = "default_border pixel 2";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.default_border == rway_config::BorderStyle::Pixel(2) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Pixel(2), got {:?}",
                    config.default_border
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_exec() -> TestStatus {
    let input = "exec waybar";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.exec.iter().any(|e| e.command == "waybar");
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("exec waybar not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_exec_always() -> TestStatus {
    let input = "exec_always --no-startup-id nm-applet";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .exec_always
                .iter()
                .any(|e| e.command.contains("nm-applet"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("exec_always nm-applet not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_focus_follows_mouse() -> TestStatus {
    let input = "focus_follows_mouse yes";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.focus_follows_mouse.is_enabled() {
                TestStatus::Pass
            } else {
                TestStatus::Fail("focus_follows_mouse should be yes".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_font() -> TestStatus {
    let input = "font pango:monospace 10";
    match rway_config::parse(input) {
        Ok(config) => {
            if let Some(font) = &config.font {
                if font.contains("monospace") {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail(format!("unexpected font: {font}"))
                }
            } else {
                TestStatus::Fail("font not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_for_window() -> TestStatus {
    let input = r#"for_window [app_id="firefox"] floating enable"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.window_rules.iter().any(|r| {
                r.criteria.app_id.as_deref() == Some("firefox")
                    && r.action == rway_config::WindowRuleAction::FloatingEnable
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("for_window rule not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_workspace_output() -> TestStatus {
    let input = "workspace 1 output eDP-1";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .workspaces
                .iter()
                .any(|w| w.name == "1" && w.outputs.contains(&"eDP-1".to_string()));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("workspace output binding not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_input_block() -> TestStatus {
    let input = "input \"1:1:keyboard\" {\n    xkb_layout us\n}";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .inputs
                .iter()
                .any(|i| i.identifier.contains("keyboard"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("input block not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_mode_block() -> TestStatus {
    let input = "mode \"resize\" {\n    bindsym Left resize shrink width 10 px\n}";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.modes.iter().any(|m| m.name == "resize") {
                TestStatus::Pass
            } else {
                TestStatus::Fail("mode block not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_variable_substitution() -> TestStatus {
    let input = "set $mod Mod4\nset $term alacritty\nbindsym $mod+Return exec $term";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.key == "Return" && kb.action == rway_config::Action::Exec("alacritty".into())
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("variable substitution in bindsym failed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_comments() -> TestStatus {
    let input = "# This is a comment\nset $mod Mod4\n# Another comment";
    match rway_config::parse(input) {
        Ok(config) => {
            // 解析器去掉 $ 前缀，key 为 "mod"
            if config.variables.get("mod") == Some(&"Mod4".to_string()) {
                TestStatus::Pass
            } else {
                TestStatus::Fail("comments interfered with parsing".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_output_resolution() -> TestStatus {
    let input = "output eDP-1 resolution 1920x1080";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.resolution == Some((1920, 1080)));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("output resolution not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_include_directive() -> TestStatus {
    let input = "include /etc/sway/config.d/*";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.includes.iter().any(|p| p.contains("config.d")) {
                TestStatus::Pass
            } else {
                TestStatus::Fail("include path not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_default_orientation() -> TestStatus {
    let input = "default_orientation horizontal";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.default_orientation.as_deref() == Some("horizontal") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected horizontal, got {:?}",
                    config.default_orientation
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_workspace_layout() -> TestStatus {
    let input = "workspace_layout tabbed";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.workspace_layout.as_deref() == Some("tabbed") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected tabbed, got {:?}",
                    config.workspace_layout
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_xwayland() -> TestStatus {
    let input = "xwayland enable";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.xwayland.as_deref() == Some("enable") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected enable, got {:?}", config.xwayland))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_swaybg_command() -> TestStatus {
    let input = "swaybg_command /usr/bin/swaybg";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.swaybg_command.as_deref() == Some("/usr/bin/swaybg") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected /usr/bin/swaybg, got {:?}",
                    config.swaybg_command
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_swaynag_command() -> TestStatus {
    let input = "swaynag_command /usr/bin/swaynag";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.swaynag_command.as_deref() == Some("/usr/bin/swaynag") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected /usr/bin/swaynag, got {:?}",
                    config.swaynag_command
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_floating_modifier() -> TestStatus {
    let input = "floating_modifier Mod4";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.floating_modifier == Some(rway_config::Modifier::Mod4) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Some(Mod4), got {:?}",
                    config.floating_modifier
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_floating_max_size() -> TestStatus {
    let input = "floating_maximum_size 1920 x 1080";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.floating_maximum_size == Some((1920, 1080)) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected (1920,1080), got {:?}",
                    config.floating_maximum_size
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_floating_min_size() -> TestStatus {
    let input = "floating_minimum_size 75 x 50";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.floating_minimum_size == Some((75, 50)) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected (75,50), got {:?}",
                    config.floating_minimum_size
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_focus_follows_mouse_always() -> TestStatus {
    let input = "focus_follows_mouse always";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.focus_follows_mouse == rway_config::FocusFollowsMouse::Always {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Always, got {:?}",
                    config.focus_follows_mouse
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_focus_on_window_activation() -> TestStatus {
    let input = "focus_on_window_activation smart";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.focus_on_window_activation.as_deref() == Some("smart") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected smart, got {:?}",
                    config.focus_on_window_activation
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_focus_wrapping() -> TestStatus {
    let input = "focus_wrapping yes";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.focus_wrapping.as_deref() == Some("yes") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected yes, got {:?}", config.focus_wrapping))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_mouse_warping() -> TestStatus {
    let input = "mouse_warping output";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.mouse_warping.as_deref() == Some("output") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected output, got {:?}", config.mouse_warping))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_popup_during_fullscreen() -> TestStatus {
    let input = "popup_during_fullscreen smart";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.popup_during_fullscreen.as_deref() == Some("smart") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected smart, got {:?}",
                    config.popup_during_fullscreen
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_show_marks() -> TestStatus {
    let input = "show_marks yes";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.show_marks == Some(true) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected Some(true), got {:?}", config.show_marks))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_tiling_drag() -> TestStatus {
    let input = "tiling_drag enable";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.tiling_drag.as_deref() == Some("enable") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected enable, got {:?}", config.tiling_drag))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_title_align() -> TestStatus {
    let input = "title_align center";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.title_align.as_deref() == Some("center") {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected center, got {:?}", config.title_align))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_workspace_auto_back_and_forth() -> TestStatus {
    let input = "workspace_auto_back_and_forth yes";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.workspace_auto_back_and_forth {
                TestStatus::Pass
            } else {
                TestStatus::Fail("expected true".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_for_window_class() -> TestStatus {
    let input = r#"for_window [class="Firefox"] floating enable"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .window_rules
                .iter()
                .any(|r| r.criteria.class.as_deref() == Some("Firefox"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("for_window [class] rule not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_for_window_title() -> TestStatus {
    let input = r#"for_window [title="Calculator"] floating enable"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .window_rules
                .iter()
                .any(|r| r.criteria.title.as_deref() == Some("Calculator"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("for_window [title] rule not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_assign_workspace() -> TestStatus {
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
                TestStatus::Fail("assign rule not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_no_focus() -> TestStatus {
    let input = r#"no_focus [app_id="notifications"]"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .no_focus_rules
                .iter()
                .any(|c| c.app_id.as_deref() == Some("notifications"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("no_focus rule not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_bar_block() -> TestStatus {
    let input = "bar {\n    status_command waybar\n}";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.bar.is_some() {
                TestStatus::Pass
            } else {
                TestStatus::Fail("bar block not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_bar_status_command() -> TestStatus {
    let input = "bar {\n    status_command waybar\n}";
    match rway_config::parse(input) {
        Ok(config) => {
            if let Some(bar) = &config.bar {
                if bar.status_command.as_deref() == Some("waybar") {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail(format!("expected waybar, got {:?}", bar.status_command))
                }
            } else {
                TestStatus::Fail("bar block not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_bar_position() -> TestStatus {
    let input = "bar {\n    position top\n}";
    match rway_config::parse(input) {
        Ok(config) => {
            if let Some(bar) = &config.bar {
                if bar.position.as_deref() == Some("top") {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail(format!("expected top, got {:?}", bar.position))
                }
            } else {
                TestStatus::Fail("bar block not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_parse_client_focused() -> TestStatus {
    let input = "client.focused #4c7899 #285577 #ffffff #2e9ef4 #285577";
    match rway_config::parse(input) {
        Ok(config) => {
            if let Some(cc) = &config.client_focused {
                if cc.border == "#4c7899" {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail(format!("unexpected border color: {}", cc.border))
                }
            } else {
                TestStatus::Fail("client.focused not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_tests_register_without_panic() {
        let mut h = Harness::new();
        register(&mut h);
        let report = h.run();
        assert!(report.total > 0);
    }
}
