// 配置类型：按键绑定、输出设置、输入设置等

use std::collections::HashMap;

// ────────────────────────────────────────────────────────────
// 修饰键
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    Mod1,
    Mod4,
    Shift,
    Control,
    Alt,
}

// ────────────────────────────────────────────────────────────
// 动作
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutType {
    SplitH,
    SplitV,
    Tabbed,
    Stacked,
    Toggle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Exec(String),
    Focus(Direction),
    Move(Direction),
    Workspace(String),
    MoveToWorkspace(String),
    Split(SplitDirection),
    Layout(LayoutType),
    ToggleFloating,
    Fullscreen,
    Kill,
    Reload,
    Exit,
    /// 未识别的指令原样透传
    Raw(String),
}

// ────────────────────────────────────────────────────────────
// 按键绑定
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Keybinding {
    pub modifiers: Vec<Modifier>,
    pub key: String,
    pub action: Action,
}

// ────────────────────────────────────────────────────────────
// 输出配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub name: String,
    pub resolution: Option<(u32, u32)>,
    pub refresh_rate: Option<f64>,
    pub position: Option<(i32, i32)>,
    pub scale: Option<f64>,
    pub transform: Option<String>,
}

// ────────────────────────────────────────────────────────────
// 输入配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InputConfig {
    pub identifier: String,
    pub settings: HashMap<String, String>,
}

// ────────────────────────────────────────────────────────────
// 执行命令
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ExecCommand {
    pub command: String,
}

// ────────────────────────────────────────────────────────────
// 边框样式
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyle {
    Normal(u32),
    Pixel(u32),
    None,
}

// ────────────────────────────────────────────────────────────
// 间距配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct GapsConfig {
    pub inner: u32,
    pub outer: u32,
}

impl Default for GapsConfig {
    fn default() -> Self {
        GapsConfig { inner: 0, outer: 0 }
    }
}

// ────────────────────────────────────────────────────────────
// 窗口规则
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WindowCriteria {
    pub app_id: Option<String>,
    pub class: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowRuleAction {
    FloatingEnable,
    FloatingDisable,
    MoveToWorkspace(String),
    Resize(i32, i32),
}

#[derive(Debug, Clone)]
pub struct WindowRule {
    pub criteria: WindowCriteria,
    pub action: WindowRuleAction,
}

// ────────────────────────────────────────────────────────────
// 工作区配置（rway 扩展：支持多输出）
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub name: String,
    /// rway 扩展：工作区可跨多个输出
    pub outputs: Vec<String>,
}

// ────────────────────────────────────────────────────────────
// 顶层配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Config {
    pub variables: HashMap<String, String>,
    pub keybindings: Vec<Keybinding>,
    pub outputs: Vec<OutputConfig>,
    pub inputs: Vec<InputConfig>,
    pub exec: Vec<ExecCommand>,
    pub exec_always: Vec<ExecCommand>,
    pub default_border: BorderStyle,
    pub gaps: GapsConfig,
    pub focus_follows_mouse: bool,
    pub window_rules: Vec<WindowRule>,
    pub workspaces: Vec<WorkspaceConfig>,
    pub font: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            variables: HashMap::new(),
            keybindings: Vec::new(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            exec: Vec::new(),
            exec_always: Vec::new(),
            default_border: BorderStyle::Normal(2),
            gaps: GapsConfig::default(),
            focus_follows_mouse: false,
            window_rules: Vec::new(),
            workspaces: Vec::new(),
            font: None,
        }
    }
}

impl Config {
    /// 包含内置默认快捷键的配置（无配置文件时使用）
    pub fn with_defaults() -> Self {
        Config {
            keybindings: default_keybindings(),
            ..Default::default()
        }
    }
}

/// 内置默认快捷键（Mod4 = Super/Logo 键为默认 mod 键）
fn default_keybindings() -> Vec<Keybinding> {
    use Direction::*;
    use Modifier::*;

    let modkey = |key: &str, action: Action| Keybinding {
        modifiers: vec![Mod4],
        key: key.to_string(),
        action,
    };
    let modkey_shift = |key: &str, action: Action| Keybinding {
        modifiers: vec![Mod4, Shift],
        key: key.to_string(),
        action,
    };

    vec![
        // 启动终端（依次尝试 foot, alacritty, kitty, xterm）
        modkey("Return", Action::Exec("foot".into())),
        // 启动应用启动器
        modkey("d", Action::Exec("wofi --show drun".into())),
        // 关闭窗口
        modkey_shift("q", Action::Kill),
        // 焦点移动
        modkey("Left", Action::Focus(Left)),
        modkey("Right", Action::Focus(Right)),
        modkey("Up", Action::Focus(Up)),
        modkey("Down", Action::Focus(Down)),
        modkey("h", Action::Focus(Left)),
        modkey("l", Action::Focus(Right)),
        modkey("k", Action::Focus(Up)),
        modkey("j", Action::Focus(Down)),
        // 窗口移动
        modkey_shift("Left", Action::Move(Left)),
        modkey_shift("Right", Action::Move(Right)),
        modkey_shift("Up", Action::Move(Up)),
        modkey_shift("Down", Action::Move(Down)),
        // 工作区切换
        modkey("1", Action::Workspace("1".into())),
        modkey("2", Action::Workspace("2".into())),
        modkey("3", Action::Workspace("3".into())),
        modkey("4", Action::Workspace("4".into())),
        modkey("5", Action::Workspace("5".into())),
        modkey("6", Action::Workspace("6".into())),
        modkey("7", Action::Workspace("7".into())),
        modkey("8", Action::Workspace("8".into())),
        modkey("9", Action::Workspace("9".into())),
        // 移动窗口到工作区
        modkey_shift("1", Action::MoveToWorkspace("1".into())),
        modkey_shift("2", Action::MoveToWorkspace("2".into())),
        modkey_shift("3", Action::MoveToWorkspace("3".into())),
        modkey_shift("4", Action::MoveToWorkspace("4".into())),
        modkey_shift("5", Action::MoveToWorkspace("5".into())),
        // 布局
        modkey("b", Action::Split(SplitDirection::Horizontal)),
        modkey("v", Action::Split(SplitDirection::Vertical)),
        modkey("s", Action::Layout(LayoutType::Stacked)),
        modkey("w", Action::Layout(LayoutType::Tabbed)),
        modkey("e", Action::Layout(LayoutType::Toggle)),
        // 全屏 / 浮动
        modkey("f", Action::Fullscreen),
        modkey_shift("space", Action::ToggleFloating),
        // 系统
        modkey_shift("c", Action::Reload),
        modkey_shift("e", Action::Exit),
    ]
}

// ════════════════════════════════════════════════════════════
// 单元测试（TDD：先于实现编写）
// ════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Default 实现 ──────────────────────────────────────────

    #[test]
    fn config_default_has_empty_collections() {
        let cfg = Config::default();
        assert!(cfg.variables.is_empty(), "variables 应默认为空");
        assert!(cfg.keybindings.is_empty(), "keybindings 应默认为空");
        assert!(cfg.outputs.is_empty(), "outputs 应默认为空");
        assert!(cfg.inputs.is_empty(), "inputs 应默认为空");
        assert!(cfg.exec.is_empty(), "exec 应默认为空");
        assert!(cfg.exec_always.is_empty(), "exec_always 应默认为空");
        assert!(cfg.window_rules.is_empty(), "window_rules 应默认为空");
        assert!(cfg.workspaces.is_empty(), "workspaces 应默认为空");
    }

    #[test]
    fn config_default_border_is_normal_2() {
        let cfg = Config::default();
        assert_eq!(cfg.default_border, BorderStyle::Normal(2));
    }

    #[test]
    fn config_default_gaps_are_zero() {
        let cfg = Config::default();
        assert_eq!(cfg.gaps, GapsConfig { inner: 0, outer: 0 });
    }

    #[test]
    fn config_default_focus_follows_mouse_is_false() {
        let cfg = Config::default();
        assert!(!cfg.focus_follows_mouse);
    }

    #[test]
    fn config_default_font_is_none() {
        let cfg = Config::default();
        assert!(cfg.font.is_none());
    }

    // ── 类型可克隆/调试 ───────────────────────────────────────

    #[test]
    fn all_types_are_cloneable() {
        let _ = Action::Kill.clone();
        let _ = Direction::Left.clone();
        let _ = SplitDirection::Horizontal.clone();
        let _ = LayoutType::Tabbed.clone();
        let _ = BorderStyle::Pixel(2).clone();
        let _ = ExecCommand { command: "foot".into() }.clone();
    }

    #[test]
    fn modifier_variants_are_eq() {
        assert_eq!(Modifier::Mod4, Modifier::Mod4);
        assert_ne!(Modifier::Mod1, Modifier::Mod4);
        assert_ne!(Modifier::Shift, Modifier::Control);
    }

    #[test]
    fn border_style_none_eq() {
        assert_eq!(BorderStyle::None, BorderStyle::None);
        assert_ne!(BorderStyle::None, BorderStyle::Pixel(0));
    }

    #[test]
    fn gaps_config_equality() {
        assert_eq!(GapsConfig { inner: 10, outer: 5 }, GapsConfig { inner: 10, outer: 5 });
        assert_ne!(GapsConfig { inner: 10, outer: 5 }, GapsConfig { inner: 5, outer: 10 });
    }

    #[test]
    fn action_exec_eq() {
        assert_eq!(Action::Exec("foot".into()), Action::Exec("foot".into()));
        assert_ne!(Action::Exec("foot".into()), Action::Exec("alacritty".into()));
    }

    #[test]
    fn window_rule_action_variants() {
        assert_eq!(WindowRuleAction::FloatingEnable, WindowRuleAction::FloatingEnable);
        assert_eq!(
            WindowRuleAction::MoveToWorkspace("5".into()),
            WindowRuleAction::MoveToWorkspace("5".into())
        );
        assert_eq!(WindowRuleAction::Resize(800, 600), WindowRuleAction::Resize(800, 600));
        assert_ne!(WindowRuleAction::FloatingEnable, WindowRuleAction::FloatingDisable);
    }
}
