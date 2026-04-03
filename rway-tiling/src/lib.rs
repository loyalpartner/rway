pub mod container;
pub mod error;
pub mod tree;
pub use container::normalize_sizes;
pub use error::TilingError;
pub use tree::{
    Direction, GapsConfig, Layout, Node, NodeData, NodeId, Rect, ResizeAxis, TitleBar, Tree,
};

// Legacy module stubs - all functions migrated to impl Tree methods
pub mod commands;
pub mod layout;
pub mod workspace;
