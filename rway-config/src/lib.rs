// rway-config：Sway 兼容配置解析器

pub mod parser;
pub mod types;

// 重新导出核心类型和函数，方便外部使用
pub use parser::{parse, parse_file, ParseError};
pub use types::{
    Action, AssignRule, BarConfig, Bindcode, BindingFlags, BorderAction, BorderStyle, ColorConfig,
    Config, Direction, ExecCommand, FloatingAction, FocusFollowsMouse, FullscreenAction,
    GapsConfig, HideEdgeBorders, InputConfig, Keybinding, LayoutType, ModeBlock, Modifier,
    OpacityAction, OutputBackground, OutputConfig, ResizeAxis, ResizeUnit, SeatConfig,
    SplitDirection, StickyAction, WindowCriteria, WindowRule, WindowRuleAction, WorkspaceConfig,
};
