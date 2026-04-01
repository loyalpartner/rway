// rway-config：Sway 兼容配置解析器

pub mod parser;
pub mod types;

// 重新导出核心类型和函数，方便外部使用
pub use parser::{parse, parse_file, ParseError};
pub use types::{
    Action, BorderStyle, Config, Direction, ExecCommand, GapsConfig, InputConfig, Keybinding,
    LayoutType, Modifier, OutputConfig, SplitDirection, WindowCriteria, WindowRule,
    WindowRuleAction, WorkspaceConfig,
};
