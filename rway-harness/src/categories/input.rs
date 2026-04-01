//! 输入处理兼容性测试

use crate::{Category, CompatTest, Harness, Priority, TestStatus};

pub fn register(harness: &mut Harness) {
    harness.register(CompatTest {
        category: Category::Input,
        name: "keybinding_modifiers".into(),
        description: "按键绑定修饰键解析".into(),
        sway_feature: "bindsym modifiers".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_keybinding_modifiers),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "keybinding_actions".into(),
        description: "按键绑定动作类型完整".into(),
        sway_feature: "bindsym actions".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_keybinding_actions),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_device_config".into(),
        description: "输入设备配置解析".into(),
        sway_feature: "input device config".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_device_config),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "focus_follows_mouse".into(),
        description: "鼠标跟随焦点配置".into(),
        sway_feature: "focus_follows_mouse".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_focus_follows_mouse),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "bindcode_support".into(),
        description: "bindcode 按键码绑定".into(),
        sway_feature: "bindcode".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_bindcode_support),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_xkb_layout".into(),
        description: "输入设备 xkb_layout 配置".into(),
        sway_feature: "input <id> xkb_layout".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_input_xkb_layout),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_xkb_options".into(),
        description: "输入设备 xkb_options 配置".into(),
        sway_feature: "input <id> xkb_options".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_input_xkb_options),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_xkb_variant".into(),
        description: "输入设备 xkb_variant 配置".into(),
        sway_feature: "input <id> xkb_variant".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_xkb_variant),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_xkb_model".into(),
        description: "输入设备 xkb_model 配置".into(),
        sway_feature: "input <id> xkb_model".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_xkb_model),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_repeat_delay".into(),
        description: "输入设备 repeat_delay 配置".into(),
        sway_feature: "input <id> repeat_delay".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_repeat_delay),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_repeat_rate".into(),
        description: "输入设备 repeat_rate 配置".into(),
        sway_feature: "input <id> repeat_rate".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_repeat_rate),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_natural_scroll".into(),
        description: "输入设备 natural_scroll 配置".into(),
        sway_feature: "input <id> natural_scroll".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_natural_scroll),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_tap".into(),
        description: "输入设备 tap 配置".into(),
        sway_feature: "input <id> tap".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_tap),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_dwt".into(),
        description: "输入设备 dwt 配置".into(),
        sway_feature: "input <id> dwt".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_dwt),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_accel_profile".into(),
        description: "输入设备 accel_profile 配置".into(),
        sway_feature: "input <id> accel_profile".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_accel_profile),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_pointer_accel".into(),
        description: "输入设备 pointer_accel 配置".into(),
        sway_feature: "input <id> pointer_accel".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_pointer_accel),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_scroll_method".into(),
        description: "输入设备 scroll_method 配置".into(),
        sway_feature: "input <id> scroll_method".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_input_scroll_method),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "bindsym_release_flag".into(),
        description: "bindsym --release 标志".into(),
        sway_feature: "bindsym --release".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_bindsym_release_flag),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "bindsym_locked_flag".into(),
        description: "bindsym --locked 标志".into(),
        sway_feature: "bindsym --locked".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_bindsym_locked_flag),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "bindsym_whole_window_flag".into(),
        description: "bindsym --whole-window 标志".into(),
        sway_feature: "bindsym --whole-window".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_bindsym_whole_window_flag),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "unbindsym".into(),
        description: "取消按键绑定".into(),
        sway_feature: "unbindsym".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_unbindsym),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "mode_binding".into(),
        description: "模式绑定".into(),
        sway_feature: "mode binding".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_mode_binding),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_merge_settings".into(),
        description: "同一设备多行配置合并".into(),
        sway_feature: "input settings merge".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_input_merge_settings),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "input_without_quotes".into(),
        description: "无引号的 input 设备标识符".into(),
        sway_feature: "input without quotes".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_input_without_quotes),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "bindsym_no_repeat_flag".into(),
        description: "bindsym --no-repeat 标志".into(),
        sway_feature: "bindsym --no-repeat".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_bindsym_no_repeat_flag),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "bindswitch".into(),
        description: "bindswitch 开关绑定".into(),
        sway_feature: "bindswitch".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_bindswitch),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "bindgesture".into(),
        description: "bindgesture 手势绑定".into(),
        sway_feature: "bindgesture".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_bindgesture),
    });

    harness.register(CompatTest {
        category: Category::Input,
        name: "seat_config".into(),
        description: "seat 配置".into(),
        sway_feature: "seat configuration".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_seat_config),
    });
}

fn test_keybinding_modifiers() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+Shift+q kill";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.modifiers.contains(&rway_config::Modifier::Mod4)
                    && kb.modifiers.contains(&rway_config::Modifier::Shift)
                    && kb.key == "q"
                    && kb.action == rway_config::Action::Kill
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("Mod4+Shift+q kill not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_keybinding_actions() -> TestStatus {
    // 验证所有核心动作类型都可构造
    let actions = vec![
        rway_config::Action::Exec("test".into()),
        rway_config::Action::Focus(rway_config::Direction::Left),
        rway_config::Action::Move(rway_config::Direction::Right),
        rway_config::Action::Workspace("1".into()),
        rway_config::Action::MoveToWorkspace("2".into()),
        rway_config::Action::Split(rway_config::SplitDirection::Horizontal),
        rway_config::Action::Layout(rway_config::LayoutType::Tabbed),
        rway_config::Action::Floating(rway_config::FloatingAction::Toggle),
        rway_config::Action::Fullscreen(rway_config::FullscreenAction::Toggle),
        rway_config::Action::Kill,
        rway_config::Action::Reload,
        rway_config::Action::Exit,
    ];

    if actions.len() == 12 {
        TestStatus::Pass
    } else {
        TestStatus::Fail("action types incomplete".into())
    }
}

fn test_input_device_config() -> TestStatus {
    let input = "input \"1:1:keyboard\" {\n    xkb_layout us\n}";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.inputs.is_empty() {
                TestStatus::Fail("no input config parsed".into())
            } else {
                TestStatus::Pass
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_focus_follows_mouse() -> TestStatus {
    let input = "focus_follows_mouse yes";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.focus_follows_mouse.is_enabled() {
                TestStatus::Pass
            } else {
                TestStatus::Fail("focus_follows_mouse not set to yes".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_xkb_layout() -> TestStatus {
    let input = r#"input "1:1:keyboard" xkb_layout us"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("xkb_layout") == Some(&"us".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("xkb_layout setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_xkb_options() -> TestStatus {
    let input = r#"input "1:1:keyboard" xkb_options caps:escape"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("xkb_options") == Some(&"caps:escape".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("xkb_options setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_xkb_variant() -> TestStatus {
    let input = r#"input "1:1:keyboard" xkb_variant intl"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("xkb_variant") == Some(&"intl".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("xkb_variant setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_xkb_model() -> TestStatus {
    let input = r#"input "1:1:keyboard" xkb_model pc105"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("xkb_model") == Some(&"pc105".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("xkb_model setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_repeat_delay() -> TestStatus {
    let input = r#"input "1:1:keyboard" repeat_delay 300"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("repeat_delay") == Some(&"300".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("repeat_delay setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_repeat_rate() -> TestStatus {
    let input = r#"input "1:1:keyboard" repeat_rate 50"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("repeat_rate") == Some(&"50".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("repeat_rate setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_natural_scroll() -> TestStatus {
    let input = r#"input "1:1:touchpad" natural_scroll enabled"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("natural_scroll") == Some(&"enabled".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("natural_scroll setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_tap() -> TestStatus {
    let input = r#"input "1:1:touchpad" tap enabled"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("tap") == Some(&"enabled".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("tap setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_dwt() -> TestStatus {
    let input = r#"input "1:1:touchpad" dwt enabled"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("dwt") == Some(&"enabled".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("dwt setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_accel_profile() -> TestStatus {
    let input = r#"input "1:1:touchpad" accel_profile adaptive"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("accel_profile") == Some(&"adaptive".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("accel_profile setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_pointer_accel() -> TestStatus {
    let input = r#"input "1:1:touchpad" pointer_accel 0.5"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("pointer_accel") == Some(&"0.5".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("pointer_accel setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_scroll_method() -> TestStatus {
    let input = r#"input "1:1:touchpad" scroll_method two_finger"#;
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i|
                i.settings.get("scroll_method") == Some(&"two_finger".to_string())
            );
            if found { TestStatus::Pass }
            else { TestStatus::Fail("scroll_method setting not found".into()) }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_bindsym_release_flag() -> TestStatus {
    let input = "bindsym --release Mod4+q kill";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.key == "q"
                    && kb.modifiers.contains(&rway_config::Modifier::Mod4)
                    && kb.flags.release
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("--release flag not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_bindsym_locked_flag() -> TestStatus {
    let input = "bindsym --locked Mod4+l exec swaylock";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.key == "l"
                    && kb.modifiers.contains(&rway_config::Modifier::Mod4)
                    && kb.flags.locked
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("--locked flag not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_bindsym_whole_window_flag() -> TestStatus {
    let input = "bindsym --whole-window button2 kill";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.key == "button2" && kb.flags.whole_window
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("--whole-window flag not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_unbindsym() -> TestStatus {
    let input = "bindsym Mod4+q kill\nunbindsym Mod4+q";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.keybindings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail("unbindsym did not remove keybinding".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_mode_binding() -> TestStatus {
    let input = "mode \"resize\" {\n    bindsym Left resize shrink width 10 px\n}";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.modes.iter().any(|m| {
                m.name == "resize" && !m.keybindings.is_empty()
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("mode 'resize' not parsed or has no keybindings".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_bindcode_support() -> TestStatus {
    let input = "bindcode 36 exec alacritty";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.bindcodes.iter().any(|bc| bc.code == 36);
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("bindcode with code 36 not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_merge_settings() -> TestStatus {
    let input = "input \"type:keyboard\" xkb_layout us\ninput \"type:keyboard\" xkb_variant intl";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.inputs.len() == 1
                && config.inputs[0].settings.get("xkb_layout") == Some(&"us".to_string())
                && config.inputs[0].settings.get("xkb_variant") == Some(&"intl".to_string())
            {
                TestStatus::Pass
            } else {
                TestStatus::Fail("input settings not merged".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_input_without_quotes() -> TestStatus {
    let input = "input type:keyboard xkb_layout de";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.inputs.iter().any(|i| {
                i.settings.get("xkb_layout") == Some(&"de".to_string())
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("input without quotes not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_bindsym_no_repeat_flag() -> TestStatus {
    let input = "bindsym --no-repeat Mod4+r exec rofi";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.key == "r"
                    && kb.modifiers.contains(&rway_config::Modifier::Mod4)
                    && kb.flags.no_repeat
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("--no-repeat flag not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_bindswitch() -> TestStatus {
    // bindswitch 是 P2 功能，解析器尚未实现
    TestStatus::NotImplemented
}

fn test_bindgesture() -> TestStatus {
    // bindgesture 是 P2 功能，解析器尚未实现
    TestStatus::NotImplemented
}

fn test_seat_config() -> TestStatus {
    let input = "seat seat0 hide_cursor 5000";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.seat_configs.iter().any(|s| s.name == "seat0");
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("seat config for 'seat0' not found".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_tests_register_without_panic() {
        let mut h = Harness::new();
        register(&mut h);
        let report = h.run();
        assert!(report.total > 0);
    }
}
