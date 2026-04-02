//! 外观样式兼容性测试

use crate::{Category, CompatTest, Harness, Priority, TestStatus};

pub fn register(harness: &mut Harness) {
    harness.register(CompatTest {
        category: Category::Appearance,
        name: "default_border_pixel".into(),
        description: "default_border pixel N 配置".into(),
        sway_feature: "default_border pixel".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_default_border_pixel),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "default_border_normal".into(),
        description: "default_border normal N 配置".into(),
        sway_feature: "default_border normal".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_default_border_normal),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "default_border_none".into(),
        description: "default_border none 配置".into(),
        sway_feature: "default_border none".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_default_border_none),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "font_config".into(),
        description: "字体配置解析".into(),
        sway_feature: "font".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_font_config),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "gaps_visual".into(),
        description: "间距影响实际窗口几何".into(),
        sway_feature: "gaps".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_gaps_visual),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "client_colors".into(),
        description: "client.focused 等颜色配置".into(),
        sway_feature: "client colors".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_client_colors),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "title_format".into(),
        description: "窗口标题栏格式".into(),
        sway_feature: "title_format".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_title_format),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "gaps_inner_outer_combined".into(),
        description: "内外间距组合效果".into(),
        sway_feature: "gaps inner/outer combined".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_gaps_inner_outer_combined),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "default_floating_border".into(),
        description: "浮动窗口默认边框".into(),
        sway_feature: "default_floating_border".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_default_floating_border),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "border_command".into(),
        description: "边框命令".into(),
        sway_feature: "border none/normal/pixel".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_border_command),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "border_toggle".into(),
        description: "边框切换".into(),
        sway_feature: "border toggle".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_border_toggle),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "hide_edge_borders".into(),
        description: "隐藏边缘边框".into(),
        sway_feature: "hide_edge_borders".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_hide_edge_borders),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "smart_borders".into(),
        description: "智能边框".into(),
        sway_feature: "smart_borders on/off".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_smart_borders),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "smart_gaps".into(),
        description: "智能间距".into(),
        sway_feature: "smart_gaps on/off".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_smart_gaps),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "opacity_command".into(),
        description: "透明度命令".into(),
        sway_feature: "opacity set/plus/minus".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_opacity_command),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "titlebar_border_thickness".into(),
        description: "标题栏边框厚度".into(),
        sway_feature: "titlebar_border_thickness".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_titlebar_border_thickness),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "titlebar_padding".into(),
        description: "标题栏内边距".into(),
        sway_feature: "titlebar_padding".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_titlebar_padding),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "client_focused_colors".into(),
        description: "聚焦窗口颜色配置".into(),
        sway_feature: "client.focused".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_client_focused_colors),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "client_unfocused_colors".into(),
        description: "非聚焦窗口颜色配置".into(),
        sway_feature: "client.unfocused".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_client_unfocused_colors),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "default_border_zero_pixel".into(),
        description: "default_border pixel 0 无边框".into(),
        sway_feature: "default_border pixel 0".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_default_border_zero_pixel),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "gaps_zero".into(),
        description: "gaps inner 0 + gaps outer 0 无间距".into(),
        sway_feature: "gaps zero".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_gaps_zero),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "font_pango_prefix".into(),
        description: "font pango:Sans 11 解析".into(),
        sway_feature: "font pango prefix".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_font_pango_prefix),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "client_urgent_colors".into(),
        description: "client.urgent 颜色配置".into(),
        sway_feature: "client.urgent".into(),
        priority: Priority::P1,
        test_fn: Box::new(test_client_urgent_colors),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "client_placeholder_colors".into(),
        description: "client.placeholder 颜色配置".into(),
        sway_feature: "client.placeholder".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_client_placeholder_colors),
    });

    harness.register(CompatTest {
        category: Category::Appearance,
        name: "client_background_color".into(),
        description: "client.background 颜色配置".into(),
        sway_feature: "client.background".into(),
        priority: Priority::P2,
        test_fn: Box::new(test_client_background_color),
    });
}

fn test_default_border_pixel() -> TestStatus {
    let input = "default_border pixel 3";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.default_border == rway_config::BorderStyle::Pixel(3) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Pixel(3), got {:?}",
                    config.default_border
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_default_border_normal() -> TestStatus {
    let input = "default_border normal 2";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.default_border == rway_config::BorderStyle::Normal(2) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Normal(2), got {:?}",
                    config.default_border
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_default_border_none() -> TestStatus {
    let input = "default_border none";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.default_border == rway_config::BorderStyle::None {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!("expected None, got {:?}", config.default_border))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_font_config() -> TestStatus {
    let input = "font pango:DejaVu Sans 12";
    match rway_config::parse(input) {
        Ok(config) => {
            if let Some(font) = &config.font {
                if font.contains("DejaVu") {
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

fn test_gaps_visual() -> TestStatus {
    let mut tree = rway_tiling::Tree::new();
    let screen = rway_tiling::Rect::new(0, 0, 1920, 1080);
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen);
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::commands::insert_window(&mut tree, 1);

    let root = tree.root();
    let gaps = rway_tiling::GapsConfig {
        inner: 10,
        outer: 20,
    };
    rway_tiling::layout::compute_layout(&mut tree, root, screen, &gaps);

    let geoms = rway_tiling::layout::get_window_geometries(&tree);
    if geoms.len() != 1 {
        return TestStatus::Fail("expected 1 window".into());
    }

    let w = geoms[0].1;
    // 窗口应该比全屏小（outer + inner 都有影响）
    if w.width < 1920 && w.height < 1080 {
        TestStatus::Pass
    } else {
        TestStatus::Fail(format!("gaps not affecting geometry: {:?}", w))
    }
}

fn test_gaps_inner_outer_combined() -> TestStatus {
    let mut tree = rway_tiling::Tree::new();
    let screen = rway_tiling::Rect::new(0, 0, 1920, 1080);
    let out = rway_tiling::workspace::add_output(&mut tree, "eDP-1", screen);
    rway_tiling::workspace::add_workspace(&mut tree, out, "1");
    rway_tiling::commands::insert_window(&mut tree, 1);
    rway_tiling::commands::insert_window(&mut tree, 2);
    let gaps = rway_tiling::GapsConfig {
        inner: 10,
        outer: 20,
    };
    let root = tree.root();
    rway_tiling::layout::compute_layout(&mut tree, root, screen, &gaps);
    let geoms = rway_tiling::layout::get_window_geometries(&tree);
    if geoms.len() == 2 {
        let (_, r1) = &geoms[0];
        let (_, r2) = &geoms[1];
        // Both windows should be smaller than half screen due to gaps
        if r1.width < 960 && r2.width < 960 {
            TestStatus::Pass
        } else {
            TestStatus::Fail(format!("gaps not applied: w1={} w2={}", r1.width, r2.width))
        }
    } else {
        TestStatus::Fail(format!("expected 2 windows, got {}", geoms.len()))
    }
}

fn test_default_floating_border() -> TestStatus {
    let input = "default_floating_border pixel 3";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.default_floating_border == rway_config::BorderStyle::Pixel(3) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Pixel(3), got {:?}",
                    config.default_floating_border
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_border_command() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+b border pixel 2";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.action == rway_config::Action::Border(rway_config::BorderAction::Pixel(2))
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("border pixel 2 action not found in keybindings".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_border_toggle() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+t border toggle";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.action == rway_config::Action::Border(rway_config::BorderAction::Toggle)
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("border toggle action not found in keybindings".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_hide_edge_borders() -> TestStatus {
    let input = "hide_edge_borders smart";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.hide_edge_borders == rway_config::HideEdgeBorders::Smart {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Smart, got {:?}",
                    config.hide_edge_borders
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_smart_borders() -> TestStatus {
    let input = "smart_borders on";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.smart_borders {
                TestStatus::Pass
            } else {
                TestStatus::Fail("expected smart_borders=true, got false".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_smart_gaps() -> TestStatus {
    let input = "smart_gaps on";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.smart_gaps {
                TestStatus::Pass
            } else {
                TestStatus::Fail("expected smart_gaps=true, got false".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_opacity_command() -> TestStatus {
    let input = "set $mod Mod4\nbindsym $mod+o opacity set 0.8";
    match rway_config::parse(input) {
        Ok(config) => {
            let found = config.keybindings.iter().any(|kb| {
                kb.action == rway_config::Action::Opacity(rway_config::OpacityAction::Set(0.8))
            });
            if found {
                TestStatus::Pass
            } else {
                TestStatus::Fail("opacity set 0.8 action not found in keybindings".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_titlebar_border_thickness() -> TestStatus {
    let input = "titlebar_border_thickness 2";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.titlebar_border_thickness == Some(2) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Some(2), got {:?}",
                    config.titlebar_border_thickness
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_titlebar_padding() -> TestStatus {
    let input = "titlebar_padding 5 3";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.titlebar_padding == Some((5, 3)) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Some((5, 3)), got {:?}",
                    config.titlebar_padding
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_client_focused_colors() -> TestStatus {
    let input = "client.focused #4c7899 #285577 #ffffff #2e9ef4 #285577";
    match rway_config::parse(input) {
        Ok(config) => match &config.client_focused {
            Some(cc) if cc.border == "#4c7899" => TestStatus::Pass,
            Some(cc) => TestStatus::Fail(format!("expected border=#4c7899, got {}", cc.border)),
            None => TestStatus::Fail("client_focused not parsed".into()),
        },
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_client_unfocused_colors() -> TestStatus {
    let input = "client.unfocused #333333 #222222 #888888 #292d2e #222222";
    match rway_config::parse(input) {
        Ok(config) => match &config.client_unfocused {
            Some(cc) if cc.border == "#333333" => TestStatus::Pass,
            Some(cc) => TestStatus::Fail(format!("expected border=#333333, got {}", cc.border)),
            None => TestStatus::Fail("client_unfocused not parsed".into()),
        },
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_client_colors() -> TestStatus {
    let input = "client.focused_inactive #333333 #5f676a #ffffff #484e50 #5f676a";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.client_focused_inactive.is_some() {
                TestStatus::Pass
            } else {
                TestStatus::Fail("client_focused_inactive not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_title_format() -> TestStatus {
    let input = "title_format %title - %app_id";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.title_format == Some("%title - %app_id".to_string()) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Some(\"%title - %app_id\"), got {:?}",
                    config.title_format
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_default_border_zero_pixel() -> TestStatus {
    let input = "default_border pixel 0";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.default_border == rway_config::BorderStyle::Pixel(0) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Pixel(0), got {:?}",
                    config.default_border
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_gaps_zero() -> TestStatus {
    let input = "gaps inner 0\ngaps outer 0";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.gaps.inner == 0 && config.gaps.outer == 0 {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected inner=0 outer=0, got inner={} outer={}",
                    config.gaps.inner, config.gaps.outer
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_font_pango_prefix() -> TestStatus {
    let input = "font pango:Sans 11";
    match rway_config::parse(input) {
        Ok(config) => {
            if let Some(font) = &config.font {
                if font.contains("Sans") {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail(format!("font does not contain 'Sans': {font}"))
                }
            } else {
                TestStatus::Fail("font not parsed".into())
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_client_urgent_colors() -> TestStatus {
    let input = "client.urgent #2f343a #900000 #ffffff #900000 #900000";
    match rway_config::parse(input) {
        Ok(config) => match &config.client_urgent {
            Some(cc) if cc.border == "#2f343a" => TestStatus::Pass,
            Some(cc) => TestStatus::Fail(format!("expected border=#2f343a, got {}", cc.border)),
            None => TestStatus::Fail("client_urgent not parsed".into()),
        },
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_client_placeholder_colors() -> TestStatus {
    let input = "client.placeholder #000000 #0c0c0c #ffffff #000000 #0c0c0c";
    match rway_config::parse(input) {
        Ok(config) => match &config.client_placeholder {
            Some(cc) if cc.border == "#000000" => TestStatus::Pass,
            Some(cc) => TestStatus::Fail(format!("expected border=#000000, got {}", cc.border)),
            None => TestStatus::Fail("client_placeholder not parsed".into()),
        },
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

fn test_client_background_color() -> TestStatus {
    let input = "client.background #ffffff";
    match rway_config::parse(input) {
        Ok(config) => {
            if config.client_background == Some("#ffffff".to_string()) {
                TestStatus::Pass
            } else {
                TestStatus::Fail(format!(
                    "expected Some(\"#ffffff\"), got {:?}",
                    config.client_background
                ))
            }
        }
        Err(e) => TestStatus::Fail(format!("parse error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appearance_tests_register_without_panic() {
        let mut h = Harness::new();
        register(&mut h);
        let report = h.run();
        assert!(report.total > 0);
    }
}
