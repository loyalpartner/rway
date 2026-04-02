pub mod tree;
pub mod error;
pub mod container;
pub use tree::{Direction, GapsConfig, Layout, Node, NodeData, NodeId, Rect, ResizeAxis, Tree};
pub use error::TilingError;
pub use container::normalize_sizes;

// Legacy module stubs - all functions migrated to impl Tree methods
pub mod workspace;
pub mod commands;
pub mod layout;
