// Sway 兼容配置文件解析器

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use crate::types::{
    Action, BorderStyle, Config, Direction, ExecCommand, GapsConfig, InputConfig, Keybinding,
    LayoutType, Modifier, OutputConfig, SplitDirection, WindowCriteria, WindowRule,
    WindowRuleAction, WorkspaceConfig,
};

// ════════════════════════════════════════════════════════════
// 错误类型
// ════════════════════════════════════════════════════════════

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("第 {line} 行语法错误: {message}")]
    Syntax { line: usize, message: String },

    #[error("无效的分辨率格式: '{0}'")]
    InvalidResolution(String),

    #[error("无效的位置格式: '{0}'")]
    InvalidPosition(String),

    #[error("无效的数值: '{0}'")]
    InvalidNumber(String),
}

// ════════════════════════════════════════════════════════════
// 公开 API
// ════════════════════════════════════════════════════════════

/// 从字符串内容解析 Sway 配置
pub fn parse(input: &str) -> Result<Config, ParseError> {
    Parser::new(input).parse()
}

/// 从文件路径解析 Sway 配置
pub fn parse_file(path: &Path) -> Result<Config, ParseError> {
    let content = std::fs::read_to_string(path)?;
    parse(&content)
}

// ════════════════════════════════════════════════════════════
// 内部解析器
// ════════════════════════════════════════════════════════════

struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input }
    }

    fn parse(self) -> Result<Config, ParseError> {
        let mut config = Config::default();

        for (line_no, raw_line) in self.input.lines().enumerate() {
            let line_no = line_no + 1; // 行号从 1 开始

            // 去除前导/尾部空白
            let line = raw_line.trim();

            // 空行和注释跳过
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 去除行内注释（# 之后的内容）
            let line = strip_inline_comment(line);

            // 变量替换（$varname -> 已设置的值）
            let substituted = substitute_vars(line, &config.variables);
            let s = substituted.trim();

            parse_directive(s, line_no, &mut config)?;
        }

        Ok(config)
    }
}

// ════════════════════════════════════════════════════════════
// 指令分发
// ════════════════════════════════════════════════════════════

fn parse_directive(line: &str, line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    // 切分第一个关键字
    let (keyword, rest) = split_first(line);

    match keyword {
        "set" => parse_set(rest, line_no, config),
        "bindsym" => parse_bindsym(rest, line_no, config),
        "output" => parse_output(rest, line_no, config),
        "input" => parse_input(rest, line_no, config),
        "exec_always" => {
            config.exec_always.push(ExecCommand { command: rest.to_string() });
            Ok(())
        }
        "exec" => {
            config.exec.push(ExecCommand { command: rest.to_string() });
            Ok(())
        }
        "gaps" => parse_gaps(rest, line_no, config),
        "default_border" => parse_default_border(rest, line_no, config),
        "for_window" => parse_for_window(rest, line_no, config),
        "workspace" => parse_workspace(rest, line_no, config),
        "font" => {
            config.font = Some(rest.to_string());
            Ok(())
        }
        "focus_follows_mouse" => {
            config.focus_follows_mouse = matches!(rest.trim(), "yes" | "true" | "1");
            Ok(())
        }
        // 未知指令：忽略（宽容解析，保持 sway 兼容性）
        _ => Ok(()),
    }
}

// ════════════════════════════════════════════════════════════
// 各指令解析器
// ════════════════════════════════════════════════════════════

/// 解析 `set $varname value`
fn parse_set(rest: &str, line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let rest = rest.trim();
    if !rest.starts_with('$') {
        return Err(ParseError::Syntax {
            line: line_no,
            message: format!("set 指令需要以 $ 开头的变量名，得到: '{rest}'"),
        });
    }
    // `$` 后面必须紧跟非空白的变量名字符
    let after_dollar = &rest[1..];
    if after_dollar.is_empty() || after_dollar.starts_with(char::is_whitespace) {
        return Err(ParseError::Syntax {
            line: line_no,
            message: "set 指令变量名不能为空（$ 后不能有空格）".to_string(),
        });
    }
    let (name, value) = split_first(after_dollar);
    if name.is_empty() {
        return Err(ParseError::Syntax {
            line: line_no,
            message: "set 指令变量名不能为空".to_string(),
        });
    }
    config.variables.insert(name.to_string(), value.trim().to_string());
    Ok(())
}

/// 解析 `bindsym <modifiers+key> <action...>`
fn parse_bindsym(rest: &str, line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let rest = rest.trim();
    let (combo, action_str) = split_first(rest);
    if combo.is_empty() {
        return Err(ParseError::Syntax {
            line: line_no,
            message: "bindsym 缺少按键组合".to_string(),
        });
    }

    let (modifiers, key) = parse_key_combo(combo);
    let action = parse_action(action_str.trim());

    config.keybindings.push(Keybinding { modifiers, key, action });
    Ok(())
}

/// 解析修饰键+按键组合，如 `Mod4+Shift+Return`
fn parse_key_combo(combo: &str) -> (Vec<Modifier>, String) {
    let parts: Vec<&str> = combo.split('+').collect();
    let mut modifiers = Vec::new();
    let mut key = String::new();

    for (i, part) in parts.iter().enumerate() {
        if let Some(m) = str_to_modifier(part) {
            modifiers.push(m);
        } else {
            // 最后一个非修饰键部分作为实际按键
            // 如果有多个非修饰键（不应该），取最后一个
            key = parts[i..].join("+");
            break;
        }
    }

    if key.is_empty() && !parts.is_empty() {
        key = parts.last().unwrap().to_string();
    }

    (modifiers, key)
}

fn str_to_modifier(s: &str) -> Option<Modifier> {
    match s {
        "Mod1" => Some(Modifier::Mod1),
        "Mod4" => Some(Modifier::Mod4),
        "Shift" => Some(Modifier::Shift),
        "Control" | "Ctrl" => Some(Modifier::Control),
        "Alt" => Some(Modifier::Alt),
        _ => None,
    }
}

/// 解析动作字符串
fn parse_action(s: &str) -> Action {
    let (verb, rest) = split_first(s);
    match verb {
        "exec" => Action::Exec(rest.trim().to_string()),
        "focus" => {
            if let Some(dir) = str_to_direction(rest.trim()) {
                Action::Focus(dir)
            } else {
                Action::Raw(s.to_string())
            }
        }
        "move" => parse_move_action(rest.trim()),
        "workspace" => parse_workspace_action(rest.trim()),
        "splith" => Action::Split(SplitDirection::Horizontal),
        "splitv" => Action::Split(SplitDirection::Vertical),
        "split" => {
            match rest.trim() {
                "h" | "horizontal" => Action::Split(SplitDirection::Horizontal),
                "v" | "vertical" => Action::Split(SplitDirection::Vertical),
                _ => Action::Raw(s.to_string()),
            }
        }
        "layout" => parse_layout_action(rest.trim()),
        "fullscreen" => Action::Fullscreen,
        "floating" => {
            if rest.trim() == "toggle" {
                Action::ToggleFloating
            } else {
                Action::Raw(s.to_string())
            }
        }
        "kill" => Action::Kill,
        "reload" => Action::Reload,
        "exit" => Action::Exit,
        _ => Action::Raw(s.to_string()),
    }
}

fn parse_move_action(rest: &str) -> Action {
    let (first, remainder) = split_first(rest);
    match first {
        "left" => Action::Move(Direction::Left),
        "right" => Action::Move(Direction::Right),
        "up" => Action::Move(Direction::Up),
        "down" => Action::Move(Direction::Down),
        "container" | "window" => {
            // `move container to workspace number 1` 或 `move container to workspace 5`
            let (to, ws_part) = split_first(remainder.trim());
            if to == "to" {
                let (ws_kw, ws_name) = split_first(ws_part.trim());
                if ws_kw == "workspace" {
                    let name = normalize_workspace_name(ws_name.trim());
                    return Action::MoveToWorkspace(name);
                }
            }
            Action::Raw(format!("move {rest}"))
        }
        "to" => {
            // `move to workspace number 1`
            let (ws_kw, ws_name) = split_first(remainder.trim());
            if ws_kw == "workspace" {
                let name = normalize_workspace_name(ws_name.trim());
                Action::MoveToWorkspace(name)
            } else {
                Action::Raw(format!("move {rest}"))
            }
        }
        _ => Action::Raw(format!("move {rest}")),
    }
}

fn parse_workspace_action(rest: &str) -> Action {
    let name = normalize_workspace_name(rest);
    Action::Workspace(name)
}

fn normalize_workspace_name(s: &str) -> String {
    // `number 1` -> `1`，`1` -> `1`，`web` -> `web`
    if let Some(stripped) = s.strip_prefix("number ") {
        stripped.trim().to_string()
    } else {
        s.to_string()
    }
}

fn parse_layout_action(rest: &str) -> Action {
    match rest {
        "splith" => Action::Layout(LayoutType::SplitH),
        "splitv" => Action::Layout(LayoutType::SplitV),
        "tabbed" => Action::Layout(LayoutType::Tabbed),
        "stacking" => Action::Layout(LayoutType::Stacked),
        "toggle" | "toggle split" => Action::Layout(LayoutType::Toggle),
        _ => Action::Raw(format!("layout {rest}")),
    }
}

fn str_to_direction(s: &str) -> Option<Direction> {
    match s {
        "left" => Some(Direction::Left),
        "right" => Some(Direction::Right),
        "up" => Some(Direction::Up),
        "down" => Some(Direction::Down),
        _ => None,
    }
}

/// 解析 `output <name> [resolution WxH[@Hz]] [position x,y] [scale f] [transform t]`
fn parse_output(rest: &str, _line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let rest = rest.trim();
    let (name, props) = split_first(rest);
    if name.is_empty() {
        return Ok(());
    }

    let mut out = OutputConfig {
        name: name.to_string(),
        resolution: None,
        refresh_rate: None,
        position: None,
        scale: None,
        transform: None,
    };

    let mut tokens = props.split_whitespace().peekable();
    while let Some(tok) = tokens.next() {
        match tok {
            "resolution" | "mode" => {
                if let Some(res_str) = tokens.next() {
                    let (w, h, hz) = parse_resolution(res_str)?;
                    out.resolution = Some((w, h));
                    out.refresh_rate = hz;
                }
            }
            "position" | "pos" => {
                if let Some(pos_str) = tokens.next() {
                    let (x, y) = parse_position(pos_str)?;
                    out.position = Some((x, y));
                }
            }
            "scale" => {
                if let Some(scale_str) = tokens.next() {
                    out.scale = Some(
                        scale_str
                            .parse::<f64>()
                            .map_err(|_| ParseError::InvalidNumber(scale_str.to_string()))?,
                    );
                }
            }
            "transform" => {
                if let Some(t) = tokens.next() {
                    out.transform = Some(t.to_string());
                }
            }
            _ => {} // 跳过未知属性
        }
    }

    // 合并同名输出（后来居上策略）
    if let Some(existing) = config.outputs.iter_mut().find(|o| o.name == out.name) {
        *existing = out;
    } else {
        config.outputs.push(out);
    }
    Ok(())
}

/// 解析 `WxH` 或 `WxH@Hz` 格式
fn parse_resolution(s: &str) -> Result<(u32, u32, Option<f64>), ParseError> {
    // 格式：2560x1440 或 2560x1440@144Hz 或 2560x1440@144.0
    let (dims, hz_part) = if let Some(idx) = s.find('@') {
        (&s[..idx], Some(&s[idx + 1..]))
    } else {
        (s, None)
    };

    let parts: Vec<&str> = dims.split('x').collect();
    if parts.len() != 2 {
        return Err(ParseError::InvalidResolution(s.to_string()));
    }

    let w = parts[0]
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidResolution(s.to_string()))?;
    let h = parts[1]
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidResolution(s.to_string()))?;

    let hz = if let Some(hz_str) = hz_part {
        // 去掉可能的 "Hz" 后缀
        let hz_str = hz_str.trim_end_matches("Hz");
        Some(
            hz_str
                .parse::<f64>()
                .map_err(|_| ParseError::InvalidResolution(s.to_string()))?,
        )
    } else {
        None
    };

    Ok((w, h, hz))
}

/// 解析 `x,y` 格式的坐标
fn parse_position(s: &str) -> Result<(i32, i32), ParseError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(ParseError::InvalidPosition(s.to_string()));
    }
    let x = parts[0]
        .parse::<i32>()
        .map_err(|_| ParseError::InvalidPosition(s.to_string()))?;
    let y = parts[1]
        .parse::<i32>()
        .map_err(|_| ParseError::InvalidPosition(s.to_string()))?;
    Ok((x, y))
}

/// 解析 `input "identifier" key value [key value ...]`
fn parse_input(rest: &str, _line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let rest = rest.trim();
    // identifier 可能带引号
    let (identifier, props) = parse_quoted_or_plain(rest);
    if identifier.is_empty() {
        return Ok(());
    }

    let mut settings = HashMap::new();
    let mut tokens = props.split_whitespace();
    while let Some(k) = tokens.next() {
        if let Some(v) = tokens.next() {
            settings.insert(k.to_string(), v.to_string());
        }
    }

    // 合并同名输入
    if let Some(existing) = config.inputs.iter_mut().find(|i| i.identifier == identifier) {
        existing.settings.extend(settings);
    } else {
        config.inputs.push(InputConfig { identifier, settings });
    }
    Ok(())
}

/// 解析 `gaps inner <n>` 或 `gaps outer <n>`
fn parse_gaps(rest: &str, line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let (direction, value_str) = split_first(rest.trim());
    let value = value_str
        .trim()
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidNumber(value_str.trim().to_string()))?;

    match direction {
        "inner" => config.gaps.inner = value,
        "outer" => config.gaps.outer = value,
        _ => {
            return Err(ParseError::Syntax {
                line: line_no,
                message: format!("gaps 方向必须是 inner 或 outer，得到: '{direction}'"),
            })
        }
    }
    Ok(())
}

/// 解析 `default_border normal|pixel <n>|none`
fn parse_default_border(rest: &str, line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let (style, width_str) = split_first(rest.trim());
    match style {
        "none" => config.default_border = BorderStyle::None,
        "normal" => {
            let w = if width_str.trim().is_empty() {
                2
            } else {
                width_str
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| ParseError::InvalidNumber(width_str.trim().to_string()))?
            };
            config.default_border = BorderStyle::Normal(w);
        }
        "pixel" => {
            let w = if width_str.trim().is_empty() {
                2
            } else {
                width_str
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| ParseError::InvalidNumber(width_str.trim().to_string()))?
            };
            config.default_border = BorderStyle::Pixel(w);
        }
        _ => {
            return Err(ParseError::Syntax {
                line: line_no,
                message: format!("未知的边框样式: '{style}'"),
            })
        }
    }
    Ok(())
}

/// 解析 `for_window [criteria] action`
fn parse_for_window(rest: &str, line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let rest = rest.trim();
    // 查找 `[` 开头的条件块
    if !rest.starts_with('[') {
        return Err(ParseError::Syntax {
            line: line_no,
            message: "for_window 需要以 [ 开头的条件块".to_string(),
        });
    }

    let end = rest.find(']').ok_or_else(|| ParseError::Syntax {
        line: line_no,
        message: "for_window 条件块缺少 ]".to_string(),
    })?;

    let criteria_str = &rest[1..end];
    let action_str = rest[end + 1..].trim();

    let criteria = parse_window_criteria(criteria_str);
    let action = parse_window_rule_action(action_str, line_no)?;

    config.window_rules.push(WindowRule { criteria, action });
    Ok(())
}

/// 解析窗口条件，如 `app_id="firefox" title=".*"` 等
fn parse_window_criteria(s: &str) -> WindowCriteria {
    let mut criteria = WindowCriteria { app_id: None, class: None, title: None };

    // 使用简单状态机解析 key="value" 或 key=value 对
    let mut remaining = s.trim();
    while !remaining.is_empty() {
        let eq_pos = match remaining.find('=') {
            Some(p) => p,
            None => break,
        };
        let key = remaining[..eq_pos].trim();
        remaining = remaining[eq_pos + 1..].trim_start();

        let (value, after) = if remaining.starts_with('"') {
            // 带引号的值
            let end = remaining[1..].find('"').map(|i| i + 1).unwrap_or(remaining.len() - 1);
            (&remaining[1..end], remaining[end + 1..].trim_start())
        } else {
            // 无引号：取到下一个空格
            let end = remaining.find(char::is_whitespace).unwrap_or(remaining.len());
            (&remaining[..end], remaining[end..].trim_start())
        };

        match key {
            "app_id" => criteria.app_id = Some(value.to_string()),
            "class" => criteria.class = Some(value.to_string()),
            "title" => criteria.title = Some(value.to_string()),
            _ => {}
        }

        remaining = after;
    }

    criteria
}

/// 解析窗口规则动作
fn parse_window_rule_action(s: &str, line_no: usize) -> Result<WindowRuleAction, ParseError> {
    let (verb, rest) = split_first(s.trim());
    match verb {
        "floating" => {
            let (sub, _) = split_first(rest.trim());
            match sub {
                "enable" => Ok(WindowRuleAction::FloatingEnable),
                "disable" => Ok(WindowRuleAction::FloatingDisable),
                _ => Err(ParseError::Syntax {
                    line: line_no,
                    message: format!("未知的 floating 子命令: '{sub}'"),
                }),
            }
        }
        "move" => {
            // `move to workspace <name>`
            let (to, ws_part) = split_first(rest.trim());
            if to == "to" {
                let (ws_kw, ws_name) = split_first(ws_part.trim());
                if ws_kw == "workspace" {
                    return Ok(WindowRuleAction::MoveToWorkspace(ws_name.trim().to_string()));
                }
            }
            Err(ParseError::Syntax {
                line: line_no,
                message: format!("无法解析 for_window move 动作: '{s}'"),
            })
        }
        "resize" => {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let w = parts[0]
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[0].to_string()))?;
                let h = parts[1]
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
                Ok(WindowRuleAction::Resize(w, h))
            } else {
                Err(ParseError::Syntax {
                    line: line_no,
                    message: "resize 需要宽度和高度两个参数".to_string(),
                })
            }
        }
        _ => Err(ParseError::Syntax {
            line: line_no,
            message: format!("未知的 for_window 动作: '{verb}'"),
        }),
    }
}

/// 解析 `workspace <name> output <output1> [output2 ...]`
fn parse_workspace(rest: &str, _line_no: usize, config: &mut Config) -> Result<(), ParseError> {
    let mut tokens = rest.split_whitespace();
    let name = match tokens.next() {
        Some(n) => n.to_string(),
        None => return Ok(()),
    };

    let mut outputs = Vec::new();
    let mut found_output_kw = false;
    for tok in tokens {
        if tok == "output" {
            found_output_kw = true;
            continue;
        }
        if found_output_kw {
            outputs.push(tok.to_string());
        }
    }

    // 合并同名工作区
    if let Some(existing) = config.workspaces.iter_mut().find(|w| w.name == name) {
        for o in outputs {
            if !existing.outputs.contains(&o) {
                existing.outputs.push(o);
            }
        }
    } else {
        config.workspaces.push(WorkspaceConfig { name, outputs });
    }
    Ok(())
}

// ════════════════════════════════════════════════════════════
// 工具函数
// ════════════════════════════════════════════════════════════

/// 将字符串切分为第一个词和剩余部分
fn split_first(s: &str) -> (&str, &str) {
    let s = s.trim();
    match s.find(char::is_whitespace) {
        Some(pos) => (&s[..pos], s[pos..].trim_start()),
        None => (s, ""),
    }
}

/// 去除行内注释（未被引号包围的 `#` 之后的内容）
fn strip_inline_comment(line: &str) -> &str {
    let mut in_quote = false;
    let mut quote_char = '"';
    for (i, ch) in line.char_indices() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
            }
            '#' if !in_quote => {
                return line[..i].trim_end();
            }
            _ => {}
        }
    }
    line
}

/// 解析可选带引号的第一个词
fn parse_quoted_or_plain(s: &str) -> (String, &str) {
    let s = s.trim();
    if s.starts_with('"') {
        // 带双引号
        if let Some(end) = s[1..].find('"') {
            let identifier = s[1..end + 1].to_string();
            let rest = s[end + 2..].trim_start();
            return (identifier, rest);
        }
    }
    // 无引号：取第一个词
    let (word, rest) = split_first(s);
    (word.to_string(), rest)
}

/// 变量替换：将 `$varname` 替换为已定义的值
fn substitute_vars(line: &str, vars: &HashMap<String, String>) -> String {
    if !line.contains('$') {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // 收集变量名（字母、数字、下划线）
            let mut var_name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    var_name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if var_name.is_empty() {
                result.push('$');
            } else if let Some(val) = vars.get(&var_name) {
                result.push_str(val);
            } else {
                // 未定义的变量保留原样
                result.push('$');
                result.push_str(&var_name);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

// ════════════════════════════════════════════════════════════
// 单元测试（TDD：先于实现完整编写，然后运行确认 RED）
// ════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::io::Write;

    // ── 辅助宏 ────────────────────────────────────────────────

    macro_rules! parse_ok {
        ($input:expr) => {
            parse($input).expect("解析应成功")
        };
    }

    macro_rules! parse_err {
        ($input:expr) => {
            parse($input).expect_err("解析应失败")
        };
    }

    // ════════════════════════════════════════════════════════
    // 基础解析：空输入与注释
    // ════════════════════════════════════════════════════════

    #[test]
    fn empty_input_returns_default_config() {
        let cfg = parse_ok!("");
        assert!(cfg.keybindings.is_empty());
        assert!(cfg.variables.is_empty());
    }

    #[test]
    fn whitespace_only_input_is_valid() {
        let cfg = parse_ok!("   \n\t\n  ");
        assert!(cfg.keybindings.is_empty());
    }

    #[test]
    fn comment_lines_are_ignored() {
        let cfg = parse_ok!("# 这是注释\n# 另一行注释");
        assert!(cfg.keybindings.is_empty());
    }

    #[test]
    fn inline_comment_is_stripped() {
        let cfg = parse_ok!("gaps inner 10 # 内边距");
        assert_eq!(cfg.gaps.inner, 10);
    }

    #[test]
    fn inline_comment_after_quoted_string() {
        // 引号内的 # 不应被视为注释
        let cfg = parse_ok!(r#"for_window [app_id="test#name"] floating enable"#);
        assert_eq!(cfg.window_rules.len(), 1);
        assert_eq!(cfg.window_rules[0].criteria.app_id.as_deref(), Some("test#name"));
    }

    // ════════════════════════════════════════════════════════
    // 变量定义与替换
    // ════════════════════════════════════════════════════════

    #[test]
    fn set_variable_simple() {
        let cfg = parse_ok!("set $mod Mod4");
        assert_eq!(cfg.variables.get("mod"), Some(&"Mod4".to_string()));
    }

    #[test]
    fn set_variable_multiword_value() {
        let cfg = parse_ok!("set $term foot --server");
        assert_eq!(cfg.variables.get("term"), Some(&"foot --server".to_string()));
    }

    #[test]
    fn variable_substitution_in_bindsym() {
        let cfg = parse_ok!("set $mod Mod4\nbindsym $mod+Return exec foot");
        assert_eq!(cfg.keybindings.len(), 1);
        assert!(cfg.keybindings[0].modifiers.contains(&Modifier::Mod4));
        assert_eq!(cfg.keybindings[0].key, "Return");
    }

    #[test]
    fn variable_substitution_in_exec() {
        let cfg = parse_ok!("set $term foot\nexec $term");
        assert_eq!(cfg.exec[0].command, "foot");
    }

    #[test]
    fn undefined_variable_kept_as_is() {
        let cfg = parse_ok!("exec $undefined_var");
        assert_eq!(cfg.exec[0].command, "$undefined_var");
    }

    #[test]
    fn set_missing_dollar_sign_returns_error() {
        let err = parse_err!("set mod Mod4");
        assert!(matches!(err, ParseError::Syntax { .. }));
    }

    #[test]
    fn set_empty_variable_name_returns_error() {
        // "set $" 后面没有名称
        let err = parse_err!("set $ Mod4");
        assert!(matches!(err, ParseError::Syntax { .. }));
    }

    // ════════════════════════════════════════════════════════
    // 按键绑定解析
    // ════════════════════════════════════════════════════════

    #[test]
    fn bindsym_exec_action() {
        let cfg = parse_ok!("bindsym Mod4+Return exec foot");
        assert_eq!(cfg.keybindings.len(), 1);
        let kb = &cfg.keybindings[0];
        assert_eq!(kb.modifiers, vec![Modifier::Mod4]);
        assert_eq!(kb.key, "Return");
        assert_eq!(kb.action, Action::Exec("foot".to_string()));
    }

    #[test]
    fn bindsym_exec_multiword() {
        let cfg = parse_ok!("bindsym Mod4+d exec wofi --show drun");
        let kb = &cfg.keybindings[0];
        assert_eq!(kb.action, Action::Exec("wofi --show drun".to_string()));
    }

    #[test]
    fn bindsym_kill() {
        let cfg = parse_ok!("bindsym Mod4+Shift+q kill");
        let kb = &cfg.keybindings[0];
        assert_eq!(kb.modifiers, vec![Modifier::Mod4, Modifier::Shift]);
        assert_eq!(kb.key, "q");
        assert_eq!(kb.action, Action::Kill);
    }

    #[test]
    fn bindsym_focus_directions() {
        let input = "bindsym Mod4+Left focus left\nbindsym Mod4+Right focus right\nbindsym Mod4+Up focus up\nbindsym Mod4+Down focus down";
        let cfg = parse_ok!(input);
        assert_eq!(cfg.keybindings[0].action, Action::Focus(Direction::Left));
        assert_eq!(cfg.keybindings[1].action, Action::Focus(Direction::Right));
        assert_eq!(cfg.keybindings[2].action, Action::Focus(Direction::Up));
        assert_eq!(cfg.keybindings[3].action, Action::Focus(Direction::Down));
    }

    #[test]
    fn bindsym_move_directions() {
        let cfg = parse_ok!("bindsym Mod4+Shift+Left move left");
        assert_eq!(cfg.keybindings[0].action, Action::Move(Direction::Left));
    }

    #[test]
    fn bindsym_workspace_number() {
        let cfg = parse_ok!("bindsym Mod4+1 workspace number 1");
        assert_eq!(cfg.keybindings[0].action, Action::Workspace("1".to_string()));
    }

    #[test]
    fn bindsym_workspace_named() {
        let cfg = parse_ok!("bindsym Mod4+w workspace web");
        assert_eq!(cfg.keybindings[0].action, Action::Workspace("web".to_string()));
    }

    #[test]
    fn bindsym_move_to_workspace() {
        let cfg = parse_ok!("bindsym Mod4+Shift+1 move container to workspace number 1");
        assert_eq!(
            cfg.keybindings[0].action,
            Action::MoveToWorkspace("1".to_string())
        );
    }

    #[test]
    fn bindsym_split_horizontal() {
        let cfg = parse_ok!("bindsym Mod4+h splith");
        assert_eq!(cfg.keybindings[0].action, Action::Split(SplitDirection::Horizontal));
    }

    #[test]
    fn bindsym_split_vertical() {
        let cfg = parse_ok!("bindsym Mod4+v splitv");
        assert_eq!(cfg.keybindings[0].action, Action::Split(SplitDirection::Vertical));
    }

    #[test]
    fn bindsym_layout_stacking() {
        let cfg = parse_ok!("bindsym Mod4+s layout stacking");
        assert_eq!(cfg.keybindings[0].action, Action::Layout(LayoutType::Stacked));
    }

    #[test]
    fn bindsym_layout_tabbed() {
        let cfg = parse_ok!("bindsym Mod4+w layout tabbed");
        assert_eq!(cfg.keybindings[0].action, Action::Layout(LayoutType::Tabbed));
    }

    #[test]
    fn bindsym_layout_toggle_split() {
        let cfg = parse_ok!("bindsym Mod4+e layout toggle split");
        assert_eq!(cfg.keybindings[0].action, Action::Layout(LayoutType::Toggle));
    }

    #[test]
    fn bindsym_fullscreen() {
        let cfg = parse_ok!("bindsym Mod4+f fullscreen");
        assert_eq!(cfg.keybindings[0].action, Action::Fullscreen);
    }

    #[test]
    fn bindsym_floating_toggle() {
        let cfg = parse_ok!("bindsym Mod4+Shift+space floating toggle");
        assert_eq!(cfg.keybindings[0].action, Action::ToggleFloating);
    }

    #[test]
    fn bindsym_reload() {
        let cfg = parse_ok!("bindsym Mod4+Shift+c reload");
        assert_eq!(cfg.keybindings[0].action, Action::Reload);
    }

    #[test]
    fn bindsym_exit() {
        let cfg = parse_ok!("bindsym Mod4+Shift+e exit");
        assert_eq!(cfg.keybindings[0].action, Action::Exit);
    }

    #[test]
    fn bindsym_unknown_action_is_raw() {
        let cfg = parse_ok!("bindsym Mod4+x some_unknown_command arg1 arg2");
        assert_eq!(
            cfg.keybindings[0].action,
            Action::Raw("some_unknown_command arg1 arg2".to_string())
        );
    }

    #[test]
    fn bindsym_ctrl_modifier() {
        let cfg = parse_ok!("bindsym Control+c kill");
        assert!(cfg.keybindings[0].modifiers.contains(&Modifier::Control));
    }

    #[test]
    fn bindsym_ctrl_alias() {
        let cfg = parse_ok!("bindsym Ctrl+c kill");
        assert!(cfg.keybindings[0].modifiers.contains(&Modifier::Control));
    }

    #[test]
    fn bindsym_mod1_modifier() {
        let cfg = parse_ok!("bindsym Mod1+Tab focus right");
        assert!(cfg.keybindings[0].modifiers.contains(&Modifier::Mod1));
    }

    #[test]
    fn bindsym_missing_combo_returns_error() {
        let err = parse_err!("bindsym");
        assert!(matches!(err, ParseError::Syntax { .. }));
    }

    #[test]
    fn multiple_keybindings() {
        let input = "bindsym Mod4+Return exec foot\nbindsym Mod4+q kill\nbindsym Mod4+f fullscreen";
        let cfg = parse_ok!(input);
        assert_eq!(cfg.keybindings.len(), 3);
    }

    // ════════════════════════════════════════════════════════
    // 输出配置
    // ════════════════════════════════════════════════════════

    #[test]
    fn output_resolution_only() {
        let cfg = parse_ok!("output DP-1 resolution 2560x1440");
        assert_eq!(cfg.outputs.len(), 1);
        let out = &cfg.outputs[0];
        assert_eq!(out.name, "DP-1");
        assert_eq!(out.resolution, Some((2560, 1440)));
        assert!(out.refresh_rate.is_none());
    }

    #[test]
    fn output_resolution_with_refresh_rate() {
        let cfg = parse_ok!("output DP-1 resolution 2560x1440@144Hz");
        let out = &cfg.outputs[0];
        assert_eq!(out.resolution, Some((2560, 1440)));
        assert_eq!(out.refresh_rate, Some(144.0));
    }

    #[test]
    fn output_resolution_with_decimal_refresh() {
        let cfg = parse_ok!("output DP-1 resolution 1920x1080@59.94Hz");
        let out = &cfg.outputs[0];
        assert!((out.refresh_rate.unwrap() - 59.94).abs() < 0.001);
    }

    #[test]
    fn output_position() {
        let cfg = parse_ok!("output DP-1 position 0,0");
        assert_eq!(cfg.outputs[0].position, Some((0, 0)));
    }

    #[test]
    fn output_negative_position() {
        let cfg = parse_ok!("output DP-1 position -1920,0");
        assert_eq!(cfg.outputs[0].position, Some((-1920, 0)));
    }

    #[test]
    fn output_scale() {
        let cfg = parse_ok!("output DP-1 scale 1.5");
        assert_eq!(cfg.outputs[0].scale, Some(1.5));
    }

    #[test]
    fn output_full_config() {
        let cfg = parse_ok!("output DP-1 resolution 2560x1440@144Hz position 0,0 scale 1.5");
        let out = &cfg.outputs[0];
        assert_eq!(out.resolution, Some((2560, 1440)));
        assert_eq!(out.refresh_rate, Some(144.0));
        assert_eq!(out.position, Some((0, 0)));
        assert_eq!(out.scale, Some(1.5));
    }

    #[test]
    fn output_transform() {
        let cfg = parse_ok!("output DP-1 transform 90");
        assert_eq!(cfg.outputs[0].transform.as_deref(), Some("90"));
    }

    #[test]
    fn multiple_outputs() {
        let input = "output DP-1 resolution 2560x1440@144Hz position 0,0\noutput HDMI-A-1 resolution 1920x1080 position 2560,0";
        let cfg = parse_ok!(input);
        assert_eq!(cfg.outputs.len(), 2);
        assert_eq!(cfg.outputs[0].name, "DP-1");
        assert_eq!(cfg.outputs[1].name, "HDMI-A-1");
    }

    #[test]
    fn output_invalid_resolution_returns_error() {
        let err = parse_err!("output DP-1 resolution 2560-1440");
        assert!(matches!(err, ParseError::InvalidResolution(_)));
    }

    #[test]
    fn output_invalid_position_returns_error() {
        let err = parse_err!("output DP-1 position 0-0");
        assert!(matches!(err, ParseError::InvalidPosition(_)));
    }

    #[test]
    fn output_mode_keyword_alias() {
        let cfg = parse_ok!("output DP-1 mode 1920x1080");
        assert_eq!(cfg.outputs[0].resolution, Some((1920, 1080)));
    }

    #[test]
    fn output_pos_keyword_alias() {
        let cfg = parse_ok!("output DP-1 pos 100,200");
        assert_eq!(cfg.outputs[0].position, Some((100, 200)));
    }

    // ════════════════════════════════════════════════════════
    // 输入配置
    // ════════════════════════════════════════════════════════

    #[test]
    fn input_keyboard_layout() {
        let cfg = parse_ok!(r#"input "type:keyboard" xkb_layout us"#);
        assert_eq!(cfg.inputs.len(), 1);
        assert_eq!(cfg.inputs[0].identifier, "type:keyboard");
        assert_eq!(cfg.inputs[0].settings.get("xkb_layout"), Some(&"us".to_string()));
    }

    #[test]
    fn input_pointer_accel() {
        let cfg = parse_ok!(r#"input "type:pointer" accel_profile adaptive"#);
        assert_eq!(cfg.inputs[0].settings.get("accel_profile"), Some(&"adaptive".to_string()));
    }

    #[test]
    fn input_without_quotes() {
        let cfg = parse_ok!("input type:keyboard xkb_layout de");
        assert_eq!(cfg.inputs[0].identifier, "type:keyboard");
        assert_eq!(cfg.inputs[0].settings.get("xkb_layout"), Some(&"de".to_string()));
    }

    #[test]
    fn input_same_device_merges_settings() {
        let input = "input type:keyboard xkb_layout us\ninput type:keyboard xkb_variant intl";
        let cfg = parse_ok!(input);
        assert_eq!(cfg.inputs.len(), 1, "同名输入设备应合并");
        assert_eq!(cfg.inputs[0].settings.get("xkb_layout"), Some(&"us".to_string()));
        assert_eq!(cfg.inputs[0].settings.get("xkb_variant"), Some(&"intl".to_string()));
    }

    // ════════════════════════════════════════════════════════
    // exec / exec_always
    // ════════════════════════════════════════════════════════

    #[test]
    fn exec_single_command() {
        let cfg = parse_ok!("exec foot");
        assert_eq!(cfg.exec.len(), 1);
        assert_eq!(cfg.exec[0].command, "foot");
    }

    #[test]
    fn exec_always_command() {
        let cfg = parse_ok!("exec_always waybar");
        assert_eq!(cfg.exec_always.len(), 1);
        assert_eq!(cfg.exec_always[0].command, "waybar");
    }

    #[test]
    fn exec_multiword_command() {
        let cfg = parse_ok!("exec foot --server --config /etc/foot.ini");
        assert_eq!(cfg.exec[0].command, "foot --server --config /etc/foot.ini");
    }

    #[test]
    fn multiple_exec_commands() {
        let cfg = parse_ok!("exec foot\nexec waybar\nexec mako");
        assert_eq!(cfg.exec.len(), 3);
    }

    // ════════════════════════════════════════════════════════
    // gaps / default_border / font / focus_follows_mouse
    // ════════════════════════════════════════════════════════

    #[test]
    fn gaps_inner() {
        let cfg = parse_ok!("gaps inner 10");
        assert_eq!(cfg.gaps.inner, 10);
    }

    #[test]
    fn gaps_outer() {
        let cfg = parse_ok!("gaps outer 5");
        assert_eq!(cfg.gaps.outer, 5);
    }

    #[test]
    fn gaps_both() {
        let cfg = parse_ok!("gaps inner 10\ngaps outer 5");
        assert_eq!(cfg.gaps, GapsConfig { inner: 10, outer: 5 });
    }

    #[test]
    fn gaps_zero() {
        let cfg = parse_ok!("gaps inner 0");
        assert_eq!(cfg.gaps.inner, 0);
    }

    #[test]
    fn gaps_invalid_direction_returns_error() {
        let err = parse_err!("gaps diagonal 10");
        assert!(matches!(err, ParseError::Syntax { .. }));
    }

    #[test]
    fn gaps_invalid_number_returns_error() {
        let err = parse_err!("gaps inner abc");
        assert!(matches!(err, ParseError::InvalidNumber(_)));
    }

    #[test]
    fn default_border_pixel() {
        let cfg = parse_ok!("default_border pixel 2");
        assert_eq!(cfg.default_border, BorderStyle::Pixel(2));
    }

    #[test]
    fn default_border_normal() {
        let cfg = parse_ok!("default_border normal 4");
        assert_eq!(cfg.default_border, BorderStyle::Normal(4));
    }

    #[test]
    fn default_border_none() {
        let cfg = parse_ok!("default_border none");
        assert_eq!(cfg.default_border, BorderStyle::None);
    }

    #[test]
    fn default_border_pixel_no_width_uses_2() {
        let cfg = parse_ok!("default_border pixel");
        assert_eq!(cfg.default_border, BorderStyle::Pixel(2));
    }

    #[test]
    fn default_border_unknown_style_returns_error() {
        let err = parse_err!("default_border fancy 2");
        assert!(matches!(err, ParseError::Syntax { .. }));
    }

    #[test]
    fn font_setting() {
        let cfg = parse_ok!("font pango:monospace 10");
        assert_eq!(cfg.font.as_deref(), Some("pango:monospace 10"));
    }

    #[test]
    fn focus_follows_mouse_yes() {
        let cfg = parse_ok!("focus_follows_mouse yes");
        assert!(cfg.focus_follows_mouse);
    }

    #[test]
    fn focus_follows_mouse_no() {
        let cfg = parse_ok!("focus_follows_mouse no");
        assert!(!cfg.focus_follows_mouse);
    }

    #[test]
    fn focus_follows_mouse_true() {
        let cfg = parse_ok!("focus_follows_mouse true");
        assert!(cfg.focus_follows_mouse);
    }

    #[test]
    fn focus_follows_mouse_1() {
        let cfg = parse_ok!("focus_follows_mouse 1");
        assert!(cfg.focus_follows_mouse);
    }

    // ════════════════════════════════════════════════════════
    // for_window 规则
    // ════════════════════════════════════════════════════════

    #[test]
    fn for_window_app_id_floating_enable() {
        let cfg = parse_ok!(r#"for_window [app_id="firefox"] floating enable"#);
        assert_eq!(cfg.window_rules.len(), 1);
        let rule = &cfg.window_rules[0];
        assert_eq!(rule.criteria.app_id.as_deref(), Some("firefox"));
        assert!(rule.criteria.class.is_none());
        assert_eq!(rule.action, WindowRuleAction::FloatingEnable);
    }

    #[test]
    fn for_window_class_move_to_workspace() {
        let cfg = parse_ok!(r#"for_window [class="Steam"] move to workspace 5"#);
        let rule = &cfg.window_rules[0];
        assert_eq!(rule.criteria.class.as_deref(), Some("Steam"));
        assert_eq!(rule.action, WindowRuleAction::MoveToWorkspace("5".to_string()));
    }

    #[test]
    fn for_window_floating_disable() {
        let cfg = parse_ok!(r#"for_window [app_id="term"] floating disable"#);
        assert_eq!(cfg.window_rules[0].action, WindowRuleAction::FloatingDisable);
    }

    #[test]
    fn for_window_resize() {
        let cfg = parse_ok!(r#"for_window [app_id="dialog"] resize 800 600"#);
        assert_eq!(cfg.window_rules[0].action, WindowRuleAction::Resize(800, 600));
    }

    #[test]
    fn for_window_title_criteria() {
        let cfg = parse_ok!(r#"for_window [title="Firefox"] floating enable"#);
        assert_eq!(cfg.window_rules[0].criteria.title.as_deref(), Some("Firefox"));
    }

    #[test]
    fn for_window_multiple_criteria() {
        let cfg = parse_ok!(r#"for_window [app_id="gimp" class="Gimp"] floating enable"#);
        let rule = &cfg.window_rules[0];
        assert_eq!(rule.criteria.app_id.as_deref(), Some("gimp"));
        assert_eq!(rule.criteria.class.as_deref(), Some("Gimp"));
    }

    #[test]
    fn for_window_missing_bracket_returns_error() {
        let err = parse_err!("for_window app_id=firefox floating enable");
        assert!(matches!(err, ParseError::Syntax { .. }));
    }

    #[test]
    fn for_window_unclosed_bracket_returns_error() {
        let err = parse_err!("for_window [app_id=firefox floating enable");
        assert!(matches!(err, ParseError::Syntax { .. }));
    }

    // ════════════════════════════════════════════════════════
    // workspace 配置（含 rway 扩展：多输出）
    // ════════════════════════════════════════════════════════

    #[test]
    fn workspace_single_output() {
        let cfg = parse_ok!("workspace 1 output DP-1");
        assert_eq!(cfg.workspaces.len(), 1);
        assert_eq!(cfg.workspaces[0].name, "1");
        assert_eq!(cfg.workspaces[0].outputs, vec!["DP-1"]);
    }

    #[test]
    fn workspace_multiple_outputs_rway_extension() {
        let cfg = parse_ok!("workspace 2 output DP-1 HDMI-A-1");
        assert_eq!(cfg.workspaces[0].outputs, vec!["DP-1", "HDMI-A-1"]);
    }

    #[test]
    fn workspace_three_outputs() {
        let cfg = parse_ok!("workspace 1 output DP-1 HDMI-A-1 eDP-1");
        assert_eq!(cfg.workspaces[0].outputs.len(), 3);
    }

    #[test]
    fn workspace_no_output() {
        let cfg = parse_ok!("workspace 1");
        assert_eq!(cfg.workspaces[0].name, "1");
        assert!(cfg.workspaces[0].outputs.is_empty());
    }

    #[test]
    fn multiple_workspaces() {
        let input = "workspace 1 output DP-1\nworkspace 2 output HDMI-A-1";
        let cfg = parse_ok!(input);
        assert_eq!(cfg.workspaces.len(), 2);
    }

    // ════════════════════════════════════════════════════════
    // 完整配置文件集成测试
    // ════════════════════════════════════════════════════════

    #[test]
    fn full_sway_config_parses_successfully() {
        let config = r#"
# rway 配置文件

set $mod Mod4
set $term foot

bindsym $mod+Return exec $term
bindsym $mod+Shift+q kill
bindsym $mod+d exec wofi --show drun
bindsym $mod+Left focus left
bindsym $mod+Right focus right
bindsym $mod+Up focus up
bindsym $mod+Down focus down
bindsym $mod+Shift+Left move left
bindsym $mod+1 workspace number 1
bindsym $mod+Shift+1 move container to workspace number 1
bindsym $mod+h splith
bindsym $mod+v splitv
bindsym $mod+s layout stacking
bindsym $mod+w layout tabbed
bindsym $mod+e layout toggle split
bindsym $mod+f fullscreen
bindsym $mod+Shift+space floating toggle
bindsym $mod+Shift+c reload
bindsym $mod+Shift+e exit

output DP-1 resolution 2560x1440@144Hz position 0,0 scale 1.5
output HDMI-A-1 resolution 1920x1080 position 2560,0

input "type:keyboard" xkb_layout us
input "type:pointer" accel_profile adaptive

gaps inner 10
gaps outer 5
default_border pixel 2

exec_always waybar
exec foot

for_window [app_id="firefox"] floating enable
for_window [class="Steam"] move to workspace 5

workspace 1 output DP-1
workspace 2 output DP-1 HDMI-A-1

font pango:monospace 10
focus_follows_mouse yes
"#;
        let cfg = parse_ok!(config);

        // 验证变量
        assert_eq!(cfg.variables.get("mod"), Some(&"Mod4".to_string()));
        assert_eq!(cfg.variables.get("term"), Some(&"foot".to_string()));

        // 验证按键绑定数量
        assert_eq!(cfg.keybindings.len(), 19);

        // 验证第一个按键绑定（$mod 已替换为 Mod4）
        assert_eq!(cfg.keybindings[0].modifiers, vec![Modifier::Mod4]);
        assert_eq!(cfg.keybindings[0].key, "Return");
        assert_eq!(cfg.keybindings[0].action, Action::Exec("foot".to_string()));

        // 验证输出
        assert_eq!(cfg.outputs.len(), 2);
        assert_eq!(cfg.outputs[0].refresh_rate, Some(144.0));
        assert_eq!(cfg.outputs[0].scale, Some(1.5));

        // 验证输入
        assert_eq!(cfg.inputs.len(), 2);

        // 验证间距
        assert_eq!(cfg.gaps, GapsConfig { inner: 10, outer: 5 });

        // 验证边框
        assert_eq!(cfg.default_border, BorderStyle::Pixel(2));

        // 验证 exec
        assert_eq!(cfg.exec_always.len(), 1);
        assert_eq!(cfg.exec.len(), 1);

        // 验证窗口规则
        assert_eq!(cfg.window_rules.len(), 2);

        // 验证工作区（含 rway 扩展）
        assert_eq!(cfg.workspaces.len(), 2);
        assert_eq!(cfg.workspaces[1].outputs, vec!["DP-1", "HDMI-A-1"]);

        // 验证字体和焦点
        assert_eq!(cfg.font.as_deref(), Some("pango:monospace 10"));
        assert!(cfg.focus_follows_mouse);
    }

    // ════════════════════════════════════════════════════════
    // 边界情况
    // ════════════════════════════════════════════════════════

    #[test]
    fn unknown_directive_is_silently_ignored() {
        // Sway 配置中可能有 rway 不认识的指令，应宽容处理
        let cfg = parse_ok!("some_unknown_directive value1 value2");
        assert!(cfg.keybindings.is_empty());
    }

    #[test]
    fn mixed_case_focus_follows_mouse() {
        let cfg = parse_ok!("focus_follows_mouse YES");
        // "YES" 不在匹配列表中，应为 false（严格匹配）
        assert!(!cfg.focus_follows_mouse);
    }

    #[test]
    fn empty_output_name_is_ignored() {
        // output 后面没有名称
        let cfg = parse_ok!("output");
        assert!(cfg.outputs.is_empty());
    }

    #[test]
    fn very_large_gaps_value() {
        let cfg = parse_ok!("gaps inner 65535");
        assert_eq!(cfg.gaps.inner, 65535);
    }

    #[test]
    fn unicode_in_exec_command() {
        let cfg = parse_ok!("exec echo '你好世界'");
        assert_eq!(cfg.exec[0].command, "echo '你好世界'");
    }

    #[test]
    fn special_chars_in_exec() {
        let cfg = parse_ok!("exec bash -c 'echo hello && echo world'");
        assert_eq!(cfg.exec[0].command, "bash -c 'echo hello && echo world'");
    }

    #[test]
    fn multiple_dollar_vars_in_one_line() {
        let input = "set $mod Mod4\nset $term foot\nbindsym $mod+Return exec $term";
        let cfg = parse_ok!(input);
        assert_eq!(cfg.keybindings[0].action, Action::Exec("foot".to_string()));
        assert!(cfg.keybindings[0].modifiers.contains(&Modifier::Mod4));
    }

    // ════════════════════════════════════════════════════════
    // parse_file 测试
    // ════════════════════════════════════════════════════════

    #[test]
    fn parse_file_reads_from_disk() {
        let mut tmpfile = tempfile::NamedTempFile::new().expect("创建临时文件失败");
        writeln!(tmpfile, "gaps inner 15").expect("写入失败");
        writeln!(tmpfile, "gaps outer 7").expect("写入失败");
        tmpfile.flush().expect("刷新失败");

        let cfg = parse_file(tmpfile.path()).expect("parse_file 应成功");
        assert_eq!(cfg.gaps.inner, 15);
        assert_eq!(cfg.gaps.outer, 7);
    }

    #[test]
    fn parse_file_not_found_returns_io_error() {
        let err = parse_file(Path::new("/nonexistent/path/to/config"))
            .expect_err("不存在的文件应返回错误");
        assert!(matches!(err, ParseError::Io(_)));
    }

    // ════════════════════════════════════════════════════════
    // 内部工具函数单元测试
    // ════════════════════════════════════════════════════════

    #[test]
    fn split_first_with_spaces() {
        assert_eq!(split_first("hello world"), ("hello", "world"));
    }

    #[test]
    fn split_first_single_word() {
        assert_eq!(split_first("hello"), ("hello", ""));
    }

    #[test]
    fn split_first_empty() {
        assert_eq!(split_first(""), ("", ""));
    }

    #[test]
    fn split_first_leading_spaces() {
        assert_eq!(split_first("  hello  world  "), ("hello", "world"));
    }

    #[test]
    fn strip_inline_comment_no_comment() {
        assert_eq!(strip_inline_comment("gaps inner 10"), "gaps inner 10");
    }

    #[test]
    fn strip_inline_comment_with_comment() {
        assert_eq!(strip_inline_comment("gaps inner 10 # 这是注释"), "gaps inner 10");
    }

    #[test]
    fn strip_inline_comment_hash_in_double_quotes() {
        assert_eq!(
            strip_inline_comment(r#"exec echo "test # not comment""#),
            r#"exec echo "test # not comment""#
        );
    }

    #[test]
    fn substitute_vars_no_vars() {
        let vars = HashMap::new();
        assert_eq!(substitute_vars("no vars here", &vars), "no vars here");
    }

    #[test]
    fn substitute_vars_single() {
        let mut vars = HashMap::new();
        vars.insert("mod".to_string(), "Mod4".to_string());
        assert_eq!(substitute_vars("bindsym $mod+Return exec foot", &vars), "bindsym Mod4+Return exec foot");
    }

    #[test]
    fn substitute_vars_multiple() {
        let mut vars = HashMap::new();
        vars.insert("mod".to_string(), "Mod4".to_string());
        vars.insert("term".to_string(), "foot".to_string());
        assert_eq!(
            substitute_vars("bindsym $mod+Return exec $term", &vars),
            "bindsym Mod4+Return exec foot"
        );
    }

    #[test]
    fn substitute_vars_undefined_kept() {
        let vars = HashMap::new();
        assert_eq!(substitute_vars("exec $unknown", &vars), "exec $unknown");
    }

    #[test]
    fn substitute_vars_dollar_at_end() {
        let vars = HashMap::new();
        assert_eq!(substitute_vars("exec test$", &vars), "exec test$");
    }

    #[test]
    fn parse_key_combo_single_modifier() {
        let (mods, key) = parse_key_combo("Mod4+Return");
        assert_eq!(mods, vec![Modifier::Mod4]);
        assert_eq!(key, "Return");
    }

    #[test]
    fn parse_key_combo_two_modifiers() {
        let (mods, key) = parse_key_combo("Mod4+Shift+q");
        assert_eq!(mods, vec![Modifier::Mod4, Modifier::Shift]);
        assert_eq!(key, "q");
    }

    #[test]
    fn parse_key_combo_no_modifier() {
        let (mods, key) = parse_key_combo("F1");
        assert!(mods.is_empty());
        assert_eq!(key, "F1");
    }

    #[test]
    fn parse_resolution_simple() {
        let (w, h, hz) = parse_resolution("1920x1080").unwrap();
        assert_eq!((w, h), (1920, 1080));
        assert!(hz.is_none());
    }

    #[test]
    fn parse_resolution_with_hz() {
        let (w, h, hz) = parse_resolution("2560x1440@144Hz").unwrap();
        assert_eq!((w, h), (2560, 1440));
        assert_eq!(hz, Some(144.0));
    }

    #[test]
    fn parse_resolution_invalid() {
        assert!(parse_resolution("not_a_resolution").is_err());
    }

    #[test]
    fn parse_position_valid() {
        let (x, y) = parse_position("100,200").unwrap();
        assert_eq!((x, y), (100, 200));
    }

    #[test]
    fn parse_position_negative() {
        let (x, y) = parse_position("-1920,0").unwrap();
        assert_eq!((x, y), (-1920, 0));
    }

    #[test]
    fn parse_position_invalid() {
        assert!(parse_position("100-200").is_err());
    }
}
