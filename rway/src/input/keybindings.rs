// input/keybindings.rs — 快捷键匹配与动作分发

use smithay::input::keyboard::ModifiersState;

use rway_config::{Action, Keybinding, Modifier};

use smithay::wayland::seat::WaylandFocus;

use crate::state::RwayState;

/// Check if the current key press matches a keybinding
pub(crate) fn find_matching_binding<'a>(
    keybindings: &'a [Keybinding],
    modifiers: &ModifiersState,
    keysym: u32,
) -> Option<&'a Action> {
    let key_name = keysym_to_name(keysym)?;

    for binding in keybindings {
        if binding.key.eq_ignore_ascii_case(&key_name)
            && modifiers_match(&binding.modifiers, modifiers)
        {
            return Some(&binding.action);
        }
    }
    None
}

/// Execute a keybinding action
pub(crate) fn execute_action(state: &mut RwayState, action: &Action) {
    match action {
        Action::Exec(cmd) => {
            tracing::info!("exec: {}", cmd);
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some((program, args)) = parts.split_first() {
                let _ = std::process::Command::new(program).args(args).spawn();
            }
        }
        Action::Focus(dir) => {
            let direction = config_dir_to_tiling(dir);
            rway_tiling::commands::move_focus(&mut state.tiling, direction);
            // 更新 Smithay 键盘焦点到平铺树的聚焦窗口
            update_keyboard_focus(state);
        }
        Action::Move(dir) => {
            let direction = config_dir_to_tiling(dir);
            rway_tiling::commands::move_window(&mut state.tiling, direction);
            state.relayout();
            update_keyboard_focus(state);
        }
        Action::Workspace(name) => {
            // 如果目标工作区不存在，先在当前输出上创建
            if !rway_tiling::workspace::switch_workspace(&mut state.tiling, name) {
                if let Some(output_id) = state.output_node {
                    rway_tiling::workspace::add_workspace(&mut state.tiling, output_id, name);
                    rway_tiling::workspace::switch_workspace(&mut state.tiling, name);
                }
            }
            state.relayout();
            update_keyboard_focus(state);
        }
        Action::MoveToWorkspace(name) => {
            // 如果目标工作区不存在，先创建
            if let Some(output_id) = state.output_node {
                rway_tiling::workspace::add_workspace(&mut state.tiling, output_id, name);
            }
            rway_tiling::commands::move_to_workspace(&mut state.tiling, name);
            state.relayout();
            update_keyboard_focus(state);
        }
        Action::FocusParent => {
            rway_tiling::commands::focus_parent(&mut state.tiling);
            update_keyboard_focus(state);
        }
        Action::FocusChild => {
            rway_tiling::commands::focus_child(&mut state.tiling);
            update_keyboard_focus(state);
        }
        Action::Resize {
            grow,
            axis,
            amount,
            unit,
        } => {
            if let Some(win_id) = rway_tiling::commands::find_focused_window_id(&state.tiling) {
                if let Some(node_id) =
                    rway_tiling::commands::find_node_by_window_id(&state.tiling, win_id)
                {
                    let delta = if *grow {
                        *amount as f64
                    } else {
                        -(*amount as f64)
                    };
                    let tiling_axis = match axis {
                        rway_config::ResizeAxis::Width => rway_tiling::commands::ResizeAxis::Width,
                        rway_config::ResizeAxis::Height => {
                            rway_tiling::commands::ResizeAxis::Height
                        }
                    };
                    let _ = unit; // unit is informational for now
                    rway_tiling::commands::resize_container(
                        &mut state.tiling,
                        node_id,
                        tiling_axis,
                        delta,
                    );
                    state.relayout();
                }
            }
        }
        Action::Split(split_dir) => {
            // Sway behavior: split h/v IMMEDIATELY wraps the focused window
            // in a new container with the specified layout direction.
            let layout = match split_dir {
                rway_config::SplitDirection::Horizontal => rway_tiling::Layout::SplitH,
                rway_config::SplitDirection::Vertical => rway_tiling::Layout::SplitV,
            };
            rway_tiling::commands::split(&mut state.tiling, layout);
            state.relayout();
        }
        Action::Layout(layout_type) => {
            let layout = match layout_type {
                rway_config::LayoutType::SplitH => rway_tiling::Layout::SplitH,
                rway_config::LayoutType::SplitV => rway_tiling::Layout::SplitV,
                rway_config::LayoutType::Tabbed => rway_tiling::Layout::Tabbed,
                rway_config::LayoutType::Stacked => rway_tiling::Layout::Stacked,
                rway_config::LayoutType::Toggle => rway_tiling::Layout::SplitH, // TODO: toggle
            };
            rway_tiling::commands::split(&mut state.tiling, layout);
            state.relayout();
        }
        Action::Kill => {
            // 关闭当前聚焦的窗口
            let focus = state.seat.get_keyboard().and_then(|kb| kb.current_focus());
            if let Some(window) = state
                .space
                .elements()
                .find(|w| {
                    let wl = w
                        .toplevel()
                        .map(|t| t.wl_surface().clone())
                        .or_else(|| w.wl_surface().map(|s| s.into_owned()));
                    wl.as_ref() == focus.as_ref()
                })
                .cloned()
            {
                if let Some(toplevel) = window.toplevel() {
                    toplevel.send_close();
                }
            }
        }
        Action::Floating(float_action) => {
            if let Some(win_id) = rway_tiling::commands::find_focused_window_id(&state.tiling) {
                match float_action {
                    rway_config::FloatingAction::Toggle => {
                        rway_tiling::commands::toggle_floating(&mut state.tiling, win_id);
                    }
                    rway_config::FloatingAction::Enable => {
                        rway_tiling::commands::set_floating(&mut state.tiling, win_id, true);
                    }
                    rway_config::FloatingAction::Disable => {
                        rway_tiling::commands::set_floating(&mut state.tiling, win_id, false);
                    }
                }
                state.relayout();
            }
        }
        Action::Fullscreen(fs_action) => {
            if let Some(win_id) = rway_tiling::commands::find_focused_window_id(&state.tiling) {
                match fs_action {
                    rway_config::FullscreenAction::Toggle => {
                        rway_tiling::commands::toggle_fullscreen(&mut state.tiling, win_id);
                    }
                    rway_config::FullscreenAction::Enable => {
                        rway_tiling::commands::set_fullscreen(&mut state.tiling, win_id, true);
                    }
                    rway_config::FullscreenAction::Disable => {
                        rway_tiling::commands::set_fullscreen(&mut state.tiling, win_id, false);
                    }
                }
                state.relayout();
            }
        }
        Action::Reload => {
            tracing::info!("重新加载配置");
            state.config = RwayState::load_config();
        }
        Action::Exit => {
            tracing::info!("退出 rway");
            state.loop_signal.stop();
        }
        Action::Raw(cmd) => {
            tracing::debug!("raw command: {}", cmd);
        }
        // 已在 config 中定义但尚未在合成器中实现的动作
        other => {
            tracing::warn!("unimplemented action: {:?}", other);
        }
    }
}

/// 更新键盘焦点到平铺树当前聚焦的窗口
fn update_keyboard_focus(state: &mut RwayState) {
    crate::focus::update_focus(state);
}

fn modifiers_match(required: &[Modifier], current: &ModifiersState) -> bool {
    for m in required {
        match m {
            Modifier::Mod4 => {
                if !current.logo {
                    return false;
                }
            }
            Modifier::Mod1 | Modifier::Alt => {
                if !current.alt {
                    return false;
                }
            }
            Modifier::Shift => {
                if !current.shift {
                    return false;
                }
            }
            Modifier::Control => {
                if !current.ctrl {
                    return false;
                }
            }
        }
    }
    // 确保没有多余的修饰键被按下
    let expected_logo = required.iter().any(|m| matches!(m, Modifier::Mod4));
    let expected_alt = required
        .iter()
        .any(|m| matches!(m, Modifier::Mod1 | Modifier::Alt));
    let expected_shift = required.iter().any(|m| matches!(m, Modifier::Shift));
    let expected_ctrl = required.iter().any(|m| matches!(m, Modifier::Control));

    current.logo == expected_logo
        && current.alt == expected_alt
        && current.shift == expected_shift
        && current.ctrl == expected_ctrl
}

/// Convert xkb keysym to sway config key name
#[allow(non_upper_case_globals)]
pub(crate) fn keysym_to_name(keysym: u32) -> Option<String> {
    use smithay::input::keyboard::keysyms::*;
    let name = match keysym {
        KEY_Return => "Return",
        KEY_Escape => "Escape",
        KEY_space => "space",
        KEY_Tab => "Tab",
        KEY_BackSpace => "BackSpace",
        KEY_Delete => "Delete",
        KEY_Left => "Left",
        KEY_Right => "Right",
        KEY_Up => "Up",
        KEY_Down => "Down",
        KEY_Home => "Home",
        KEY_End => "End",
        KEY_Prior => "Prior",
        KEY_Next => "Next",
        KEY_F1 => "F1",
        KEY_F2 => "F2",
        KEY_F3 => "F3",
        KEY_F4 => "F4",
        KEY_F5 => "F5",
        KEY_F6 => "F6",
        KEY_F7 => "F7",
        KEY_F8 => "F8",
        KEY_F9 => "F9",
        KEY_F10 => "F10",
        KEY_F11 => "F11",
        KEY_F12 => "F12",
        _ => {
            // 尝试将 keysym 转为单字符
            let ch = char::from_u32(keysym)?;
            if ch.is_alphanumeric() {
                // 返回小写形式
                return Some(ch.to_lowercase().to_string());
            }
            return None;
        }
    };
    Some(name.to_string())
}

fn config_dir_to_tiling(dir: &rway_config::Direction) -> rway_tiling::commands::Direction {
    match dir {
        rway_config::Direction::Left => rway_tiling::commands::Direction::Left,
        rway_config::Direction::Right => rway_tiling::commands::Direction::Right,
        rway_config::Direction::Up => rway_tiling::commands::Direction::Up,
        rway_config::Direction::Down => rway_tiling::commands::Direction::Down,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_match_exact() {
        let mods = ModifiersState {
            logo: true,
            alt: false,
            shift: false,
            ctrl: false,
            ..Default::default()
        };
        assert!(modifiers_match(&[Modifier::Mod4], &mods));
    }

    #[test]
    fn modifiers_match_multiple() {
        let mods = ModifiersState {
            logo: true,
            alt: false,
            shift: true,
            ctrl: false,
            ..Default::default()
        };
        assert!(modifiers_match(&[Modifier::Mod4, Modifier::Shift], &mods));
    }

    #[test]
    fn modifiers_no_match_extra_pressed() {
        let mods = ModifiersState {
            logo: true,
            alt: false,
            shift: true,
            ctrl: false,
            ..Default::default()
        };
        // 只要求 Mod4，但 Shift 也被按下了
        assert!(!modifiers_match(&[Modifier::Mod4], &mods));
    }

    #[test]
    fn keysym_return() {
        use smithay::input::keyboard::keysyms::KEY_Return;
        assert_eq!(keysym_to_name(KEY_Return), Some("Return".to_string()));
    }

    #[test]
    fn keysym_letter() {
        // 小写 'a' 的 keysym 值
        assert_eq!(keysym_to_name('a' as u32), Some("a".to_string()));
    }

    #[test]
    fn keysym_number() {
        assert_eq!(keysym_to_name('1' as u32), Some("1".to_string()));
    }
}
