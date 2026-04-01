// input/keybindings.rs — 快捷键匹配与动作分发

use smithay::input::keyboard::ModifiersState;

use rway_config::{Action, Keybinding, Modifier};

use crate::state::RwayState;

/// 检查当前按键是否匹配某个快捷键绑定
pub fn find_matching_binding<'a>(
    keybindings: &'a [Keybinding],
    modifiers: &ModifiersState,
    keysym: u32,
) -> Option<&'a Action> {
    let key_name = keysym_to_name(keysym)?;

    for binding in keybindings {
        if binding.key.eq_ignore_ascii_case(&key_name) && modifiers_match(&binding.modifiers, modifiers) {
            return Some(&binding.action);
        }
    }
    None
}

/// 执行快捷键动作
pub fn execute_action(state: &mut RwayState, action: &Action) {
    match action {
        Action::Exec(cmd) => {
            tracing::info!("exec: {}", cmd);
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some((program, args)) = parts.split_first() {
                let _ = std::process::Command::new(program)
                    .args(args)
                    .spawn();
            }
        }
        Action::Focus(dir) => {
            let direction = config_dir_to_tiling(dir);
            rway_tiling::commands::move_focus(&mut state.tiling, direction);
            // 更新 Smithay 键盘焦点到平铺树的聚焦窗口
            update_keyboard_focus(state);
        }
        Action::Move(_dir) => {
            // TODO: 移动窗口到指定方向
            tracing::debug!("move: 尚未实现");
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
        Action::MoveToWorkspace(_name) => {
            // TODO: 移动窗口到工作区
            tracing::debug!("move to workspace: 尚未实现");
        }
        Action::Split(split_dir) => {
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
            if let Some(window) = state.space.elements().find(|w| {
                w.toplevel()
                    .map(|t| t.wl_surface().clone())
                    .as_ref()
                    == state.seat.get_keyboard().unwrap().current_focus().as_ref()
            }).cloned() {
                window.toplevel().unwrap().send_close();
            }
        }
        Action::ToggleFloating => {
            if let Some(win_id) = rway_tiling::commands::find_focused_window_id(&state.tiling) {
                rway_tiling::commands::toggle_floating(&mut state.tiling, win_id);
                state.relayout();
            }
        }
        Action::Fullscreen => {
            tracing::debug!("fullscreen: 尚未实现");
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
    }
}

/// 更新键盘焦点到平铺树当前聚焦的窗口
fn update_keyboard_focus(state: &mut RwayState) {
    let focused_ws = rway_tiling::workspace::get_focused_workspace(&state.tiling);
    if let Some(ws_id) = focused_ws {
        // 尝试从聚焦工作区中找到聚焦的窗口
        if let Some(win_surface) = find_focused_window_surface(state, ws_id) {
            let serial = smithay::utils::SERIAL_COUNTER.next_serial();
            state.seat.get_keyboard().unwrap().set_focus(
                state,
                Some(win_surface),
                serial,
            );
            return;
        }
    }
    // 没有聚焦窗口时清除焦点
    let serial = smithay::utils::SERIAL_COUNTER.next_serial();
    state.seat.get_keyboard().unwrap().set_focus(
        state,
        Option::<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>::None,
        serial,
    );
}

/// 从平铺树中递归找到当前聚焦的窗口 ID
fn find_focused_window_surface(
    state: &RwayState,
    node_id: rway_tiling::NodeId,
) -> Option<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface> {
    let node = state.tiling.get(node_id)?;
    match &node.data {
        rway_tiling::NodeData::Window { window_id, .. } => {
            let window = state.window_map.get(window_id)?;
            Some(window.toplevel()?.wl_surface().clone())
        }
        rway_tiling::NodeData::Container { focused_child, .. } => {
            let children = state.tiling.children(node_id);
            let focus_idx = (*focused_child).min(children.len().saturating_sub(1));
            let child_id = *children.get(focus_idx)?;
            find_focused_window_surface(state, child_id)
        }
        rway_tiling::NodeData::Workspace { .. } => {
            let children = state.tiling.children(node_id);
            children.first().and_then(|&c| find_focused_window_surface(state, c))
        }
        _ => {
            let children = state.tiling.children(node_id);
            children.first().and_then(|&c| find_focused_window_surface(state, c))
        }
    }
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
    let expected_alt = required.iter().any(|m| matches!(m, Modifier::Mod1 | Modifier::Alt));
    let expected_shift = required.iter().any(|m| matches!(m, Modifier::Shift));
    let expected_ctrl = required.iter().any(|m| matches!(m, Modifier::Control));

    current.logo == expected_logo
        && current.alt == expected_alt
        && current.shift == expected_shift
        && current.ctrl == expected_ctrl
}

/// 将 xkb keysym 转换为 sway 配置中的按键名称
#[allow(non_upper_case_globals)]
pub fn keysym_to_name(keysym: u32) -> Option<String> {
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
