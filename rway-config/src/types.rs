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
pub enum FloatingAction {
    Enable,
    Disable,
    Toggle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FullscreenAction {
    Enable,
    Disable,
    Toggle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResizeAxis {
    Width,
    Height,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResizeUnit {
    Px,
    Ppt,
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
    Floating(FloatingAction),
    Fullscreen(FullscreenAction),
    FocusParent,
    FocusChild,
    Resize {
        grow: bool,
        axis: ResizeAxis,
        amount: i32,
        unit: ResizeUnit,
    },
    Kill,
    Reload,
    Exit,
    /// 边框样式变更
    Border(BorderAction),
    /// 透明度
    Opacity(OpacityAction),
    /// 精确设置大小
    ResizeSet { width: i32, height: i32 },
    /// 移动到绝对位置
    MovePosition { x: i32, y: i32 },
    /// 移动到屏幕中央
    MoveCenter,
    /// 粘滞窗口
    Sticky(StickyAction),
    /// 标记窗口
    Mark { name: String, add: bool },
    /// 取消标记
    Unmark(Option<String>),
    /// 移动到 scratchpad
    MoveToScratchpad,
    /// 显示 scratchpad
    ScratchpadShow,
    /// 交换容器
    SwapContainer,
    /// 空操作（注释用）
    Nop(String),
    /// 模式切换
    Mode(String),
    /// 未识别的指令原样透传
    Raw(String),
}

// ────────────────────────────────────────────────────────────
// 动作子类型
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum BorderAction {
    None,
    Normal(u32),
    Pixel(u32),
    Toggle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpacityAction {
    Set(f64),
    Plus(f64),
    Minus(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StickyAction {
    Enable,
    Disable,
    Toggle,
}

// ────────────────────────────────────────────────────────────
// 按键绑定
// ────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────
// 绑定标志
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BindingFlags {
    pub release: bool,
    pub locked: bool,
    pub whole_window: bool,
    pub no_repeat: bool,
}

#[derive(Debug, Clone)]
pub struct Keybinding {
    pub modifiers: Vec<Modifier>,
    pub key: String,
    pub action: Action,
    pub flags: BindingFlags,
}

// ────────────────────────────────────────────────────────────
// 输出配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OutputBackground {
    pub source: String,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub name: String,
    pub resolution: Option<(u32, u32)>,
    pub refresh_rate: Option<f64>,
    pub position: Option<(i32, i32)>,
    pub scale: Option<f64>,
    pub transform: Option<String>,
    pub enable: Option<bool>,
    pub power: Option<bool>,
    pub dpms: Option<bool>,
    pub bg: Option<OutputBackground>,
    pub scale_filter: Option<String>,
    pub adaptive_sync: Option<bool>,
    pub subpixel: Option<String>,
    pub max_render_time: Option<i32>,
    pub color_profile: Option<String>,
    pub allow_tearing: Option<bool>,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GapsConfig {
    pub inner: u32,
    pub outer: u32,
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
// 颜色配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ColorConfig {
    pub border: String,
    pub background: String,
    pub text: String,
    pub indicator: String,
    pub child_border: String,
}

// ────────────────────────────────────────────────────────────
// 隐藏边缘边框
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum HideEdgeBorders {
    None,
    Vertical,
    Horizontal,
    Both,
    Smart,
}

// ────────────────────────────────────────────────────────────
// 模式块
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModeBlock {
    pub name: String,
    pub keybindings: Vec<Keybinding>,
}

// ────────────────────────────────────────────────────────────
// 窗口分配规则
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AssignRule {
    pub criteria: WindowCriteria,
    pub workspace: String,
}

// ────────────────────────────────────────────────────────────
// 状态栏配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct BarConfig {
    pub status_command: Option<String>,
    pub position: Option<String>,
}

// ────────────────────────────────────────────────────────────
// Seat 配置
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SeatConfig {
    pub name: String,
    pub settings: HashMap<String, String>,
}

// ────────────────────────────────────────────────────────────
// Bindcode 支持
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Bindcode {
    pub modifiers: Vec<Modifier>,
    pub code: u32,
    pub action: Action,
    pub flags: BindingFlags,
}

// ────────────────────────────────────────────────────────────
// focus_follows_mouse 三态
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusFollowsMouse {
    Yes,
    No,
    Always,
}

impl FocusFollowsMouse {
    /// yes 和 always 都算"启用"
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Yes | Self::Always)
    }
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
    pub default_floating_border: BorderStyle,
    pub gaps: GapsConfig,
    pub focus_follows_mouse: FocusFollowsMouse,
    pub floating_modifier: Option<Modifier>,
    pub floating_modifier_inverse: bool,
    pub window_rules: Vec<WindowRule>,
    pub workspaces: Vec<WorkspaceConfig>,
    pub font: Option<String>,
    pub hide_edge_borders: HideEdgeBorders,
    pub smart_borders: bool,
    pub smart_gaps: bool,
    pub title_format: Option<String>,
    pub titlebar_border_thickness: Option<u32>,
    pub titlebar_padding: Option<(u32, u32)>,
    pub client_focused: Option<ColorConfig>,
    pub client_unfocused: Option<ColorConfig>,
    pub client_focused_inactive: Option<ColorConfig>,
    pub client_urgent: Option<ColorConfig>,
    pub client_placeholder: Option<ColorConfig>,
    pub client_background: Option<String>,
    pub modes: Vec<ModeBlock>,
    pub assigns: Vec<AssignRule>,
    pub workspace_auto_back_and_forth: bool,
    pub seat_configs: Vec<SeatConfig>,
    pub bindcodes: Vec<Bindcode>,
    pub default_orientation: Option<String>,
    pub workspace_layout: Option<String>,
    pub xwayland: Option<String>,
    pub swaybg_command: Option<String>,
    pub swaynag_command: Option<String>,
    pub mouse_warping: Option<String>,
    pub focus_wrapping: Option<String>,
    pub focus_on_window_activation: Option<String>,
    pub popup_during_fullscreen: Option<String>,
    pub show_marks: Option<bool>,
    pub tiling_drag: Option<String>,
    pub title_align: Option<String>,
    pub floating_maximum_size: Option<(i32, i32)>,
    pub floating_minimum_size: Option<(i32, i32)>,
    pub no_focus_rules: Vec<WindowCriteria>,
    pub includes: Vec<String>,
    pub bar: Option<BarConfig>,
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
            default_floating_border: BorderStyle::Normal(2),
            gaps: GapsConfig::default(),
            focus_follows_mouse: FocusFollowsMouse::No,
            floating_modifier: None,
            floating_modifier_inverse: false,
            window_rules: Vec::new(),
            workspaces: Vec::new(),
            font: None,
            hide_edge_borders: HideEdgeBorders::None,
            smart_borders: false,
            smart_gaps: false,
            title_format: None,
            titlebar_border_thickness: None,
            titlebar_padding: None,
            client_focused: None,
            client_unfocused: None,
            client_focused_inactive: None,
            client_urgent: None,
            client_placeholder: None,
            client_background: None,
            modes: Vec::new(),
            assigns: Vec::new(),
            workspace_auto_back_and_forth: false,
            seat_configs: Vec::new(),
            bindcodes: Vec::new(),
            default_orientation: None,
            workspace_layout: None,
            xwayland: None,
            swaybg_command: None,
            swaynag_command: None,
            mouse_warping: None,
            focus_wrapping: None,
            focus_on_window_activation: None,
            popup_during_fullscreen: None,
            show_marks: None,
            tiling_drag: None,
            title_align: None,
            floating_maximum_size: None,
            floating_minimum_size: None,
            no_focus_rules: Vec::new(),
            includes: Vec::new(),
            bar: None,
        }
    }
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
    fn config_default_focus_follows_mouse_is_no() {
        let cfg = Config::default();
        assert_eq!(cfg.focus_follows_mouse, FocusFollowsMouse::No);
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
