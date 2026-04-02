use crate::tree::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum TilingError {
    #[error("node {0:?} not found")]
    NodeNotFound(NodeId),
    #[error("node {0:?} is not a container")]
    NotAContainer(NodeId),
    #[error("node {0:?} is not a workspace")]
    NotAWorkspace(NodeId),
    #[error("node {0:?} is not an output")]
    NotAnOutput(NodeId),
    #[error("node {0:?} is not a window")]
    NotAWindow(NodeId),
    #[error("window {0} not found")]
    WindowNotFound(u64),
    #[error("no focused workspace")]
    NoFocusedWorkspace,
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),
    #[error("child index {index} out of bounds (len {len}) in container {container:?}")]
    ChildIndexOutOfBounds {
        container: NodeId,
        index: usize,
        len: usize,
    },
}
