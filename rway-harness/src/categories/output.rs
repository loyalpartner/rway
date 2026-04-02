//! 输出管理兼容性测试

use crate::{Category, CompatTest, Harness, Priority, TestStatus};

pub fn register(harness: &mut Harness) {
    harness.register(CompatTest {
        category: Category::Output,
        name: "output_config_resolution".into(),
        description: "输出分辨率配置".into(),
        sway_feature: "output resolution".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_output_config_resolution),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_config_scale".into(),
        description: "输出缩放配置".into(),
        sway_feature: "output scale".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_config_scale),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_config_position".into(),
        description: "输出位置配置".into(),
        sway_feature: "output position".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_config_position),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_config_transform".into(),
        description: "输出旋转/镜像配置".into(),
        sway_feature: "output transform".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_config_transform),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "multi_output_tiling".into(),
        description: "多输出独立平铺树".into(),
        sway_feature: "multi-output".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_multi_output_tiling),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "ipc_output_info".into(),
        description: "IPC get_outputs 响应格式".into(),
        sway_feature: "get_outputs".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_ipc_output_info),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_refresh_rate".into(),
        description: "输出刷新率配置".into(),
        sway_feature: "output refresh rate".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_refresh_rate),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_multiple_configs".into(),
        description: "多输出配置解析".into(),
        sway_feature: "multiple output configs".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_multiple_configs),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_enable_disable".into(),
        description: "输出启用/禁用".into(),
        sway_feature: "output enable/disable".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_enable_disable),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_toggle".into(),
        description: "输出切换".into(),
        sway_feature: "output toggle".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_toggle),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_power".into(),
        description: "输出电源开关".into(),
        sway_feature: "output power on/off".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_power),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_dpms".into(),
        description: "输出 DPMS 开关".into(),
        sway_feature: "output dpms on/off".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_dpms),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_bg_image".into(),
        description: "输出背景图片".into(),
        sway_feature: "output bg <file> <mode>".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_bg_image),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_bg_solid_color".into(),
        description: "输出纯色背景".into(),
        sway_feature: "output bg <color> solid_color".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_bg_solid_color),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_scale_filter".into(),
        description: "输出缩放过滤器".into(),
        sway_feature: "output scale_filter".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_scale_filter),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_adaptive_sync".into(),
        description: "输出自适应同步".into(),
        sway_feature: "output adaptive_sync".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_adaptive_sync),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_subpixel".into(),
        description: "输出子像素渲染".into(),
        sway_feature: "output subpixel".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_subpixel),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_config_name_wildcard".into(),
        description: "output * 通配符配置".into(),
        sway_feature: "output * wildcard".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_output_config_name_wildcard),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_ipc_rect_fields".into(),
        description: "IpcRect 序列化包含 x/y/width/height".into(),
        sway_feature: "IpcRect JSON fields".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_output_ipc_rect_fields),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_ipc_mode_fields".into(),
        description: "IpcMode 序列化包含 width/height/refresh".into(),
        sway_feature: "IpcMode JSON fields".into(),
        priority: Priority::P0,
        test_fn: Box::new(test_output_ipc_mode_fields),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_max_render_time".into(),
        description: "output max_render_time 配置".into(),
        sway_feature: "output max_render_time".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_max_render_time),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_color_profile".into(),
        description: "output color_profile 配置".into(),
        sway_feature: "output color_profile".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_color_profile),
    });

    harness.register(CompatTest {
        category: Category::Output,
        name: "output_allow_tearing".into(),
        description: "output allow_tearing 配置".into(),
        sway_feature: "output allow_tearing".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_output_allow_tearing),
    });
}

fn test_output_config_resolution() -> TestStatus {
    let input = "output eDP-1 resolution 2560x1440";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.resolution == Some((2560, 1440)));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("output resolution not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_config_scale() -> TestStatus {
    let input = "output eDP-1 scale 2";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.scale == Some(2.0));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("output scale not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_config_position() -> TestStatus {
    // 解析器使用逗号分隔的位置格式：position x,y
    let input = "output eDP-1 position 0,0";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.position == Some((0, 0)));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("output position not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_config_transform() -> TestStatus {
    let input = "output eDP-1 transform 90";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.transform.is_some());
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("output transform not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_multi_output_tiling() -> TestStatus {
    let mut tree = rway_tiling::Tree::new();
    let screen1 = rway_tiling::Rect::new(0, 0, 1920, 1080);
    let screen2 = rway_tiling::Rect::new(1920, 0, 2560, 1440);

    let out1 = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen1);
    let out2 = rway_tiling::workspace::add_output(&mut tree, "HDMI-1", screen2);

    rway_tiling::workspace::add_workspace(&mut tree, out1, "1");
    rway_tiling::workspace::add_workspace(&mut tree, out2, "2");

    // 两个输出都注册成功
    let children = tree.children(tree.root());
    if children.len() == 2 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("expected 2 outputs, got {}", children.len()))
    }
}

fn test_output_refresh_rate() -> TestStatus {
    let input = "output eDP-1 resolution 1920x1080@60";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.refresh_rate.is_some());
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("refresh rate not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_multiple_configs() -> TestStatus {
    let input = "output eDP-1 resolution 1920x1080\noutput HDMI-1 resolution 2560x1440";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.outputs.len() == 2 {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected 2 outputs, got {}", config.outputs.len()))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_enable_disable() -> TestStatus {
    let input = "output eDP-1 enable\noutput HDMI-1 disable";
    match rway_config::parse(input) {
        Ok(config) => {
            let edp_enable = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.enable == Some(true));
            let hdmi_disable = config
                .outputs
                .iter()
                .any(|o| o.name == "HDMI-1" && o.enable == Some(false));
            if edp_enable && hdmi_disable {
                TestStatus::Pass
            } else {
                TestStatus::Fail("enable/disable not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_toggle() -> TestStatus {
    // toggle 是运行时动作，此处仅验证 disable 的配置解析
    let input = "output eDP-1 disable";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.enable == Some(false));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("enable field not parsed for disable".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_power() -> TestStatus {
    let input = "output eDP-1 power on";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.power == Some(true));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("power not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_dpms() -> TestStatus {
    let input = "output eDP-1 dpms on";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.dpms == Some(true));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("dpms not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_bg_image() -> TestStatus {
    let input = "output eDP-1 bg /path/to/image.png fill";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.outputs.iter().any(|o| {
                o.name == "eDP-1"
                    && o.bg
                        .as_ref()
                        .is_some_and(|bg| bg.source == "/path/to/image.png" && bg.mode == "fill")
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("bg image not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_bg_solid_color() -> TestStatus {
    let input = "output * bg #000000 solid_color";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.outputs.iter().any(|o| {
                o.name == "*"
                    && o.bg
                        .as_ref()
                        .is_some_and(|bg| bg.source == "#000000" && bg.mode == "solid_color")
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("bg solid_color not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_scale_filter() -> TestStatus {
    let input = "output eDP-1 scale_filter linear";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.scale_filter.as_deref() == Some("linear"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("scale_filter not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_adaptive_sync() -> TestStatus {
    let input = "output eDP-1 adaptive_sync on";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.adaptive_sync == Some(true));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("adaptive_sync not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_subpixel() -> TestStatus {
    let input = "output eDP-1 subpixel rgb";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.subpixel.as_deref() == Some("rgb"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("subpixel not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_config_name_wildcard() -> TestStatus {
    let input = "output * bg #000000 solid_color";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.outputs.iter().any(|o| o.name == "*");
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("wildcard output '*' not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_ipc_rect_fields() -> TestStatus {
    let rect = rway_ipc::IpcRect {
        x: 10,
        y: 20,
        width: 1920,
        height: 1080,
    };
    let json = serde_json::to_value(&rect).unwrap();
    let required = ["x", "y", "width", "height"];
    for field in &required {
        if json.get(field).is_none() {
            return TestStatus::Fail(format!("missing field: {field}"));
        }
    }
    if json["x"] == 10 && json["y"] == 20 && json["width"] == 1920 && json["height"] == 1080 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("IpcRect values incorrect: {json}"))
    }
}

fn test_output_ipc_mode_fields() -> TestStatus {
    let mode = rway_ipc::IpcMode {
        width: 2560,
        height: 1440,
        refresh: 144000,
    };
    let json = serde_json::to_value(&mode).unwrap();
    let required = ["width", "height", "refresh"];
    for field in &required {
        if json.get(field).is_none() {
            return TestStatus::Fail(format!("missing field: {field}"));
        }
    }
    if json["width"] == 2560 && json["height"] == 1440 && json["refresh"] == 144000 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("IpcMode values incorrect: {json}"))
    }
}

fn test_output_max_render_time() -> TestStatus {
    let input = "output eDP-1 max_render_time 5";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.max_render_time == Some(5));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("max_render_time not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_color_profile() -> TestStatus {
    let input = "output eDP-1 color_profile srgb";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.color_profile.as_deref() == Some("srgb"));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("color_profile not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_output_allow_tearing() -> TestStatus {
    let input = "output eDP-1 allow_tearing yes";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config
                .outputs
                .iter()
                .any(|o| o.name == "eDP-1" && o.allow_tearing == Some(true));
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("allow_tearing not parsed correctly".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_ipc_output_info() -> TestStatus {
    let output = rway_ipc::OutputInfo {
        id: 1,
        name: "eDP-1".into(),
        make: "BOE".into(),
        model: "0x0ABC".into(),
        serial: "".into(),
        active: true,
        primary: true,
        scale: 2.0,
        transform: "normal".into(),
        current_workspace: Some("1".into()),
        rect: rway_ipc::IpcRect {
            x: 0,
            y: 0,
            width: 2560,
            height: 1600,
        },
        current_mode: rway_ipc::IpcMode {
            width: 2560,
            height: 1600,
            refresh: 60000,
        },
    };
    let json = serde_json::to_value(&output).unwrap();

    if json["name"] == "eDP-1" && json["scale"] == 2.0 && json["active"] == true {
        TestStatus::Pass
    } else {
        TestStatus::Fail("output info JSON format incorrect".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_tests_register_without_panic() {
        let mut h = Harness::new();
        register(&mut h);
        let report = h.run();
        assert!(report.total > 0);
    }
}
