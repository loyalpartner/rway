// Workspace management: all operations migrated to impl Tree methods in tree.rs.
//
// These free functions are kept as thin delegation wrappers for backward
// compatibility with external callers that still use the module-level API.

use crate::tree::{NodeId, Rect, Tree};

pub fn add_output(tree: &mut Tree, name: &str, geometry: Rect) -> NodeId {
    tree.add_output(name, geometry)
}

pub fn add_workspace(tree: &mut Tree, output_id: NodeId, name: &str) -> NodeId {
    tree.add_workspace(output_id, name)
}

pub fn get_focused_workspace(tree: &Tree) -> Option<NodeId> {
    tree.focused_workspace()
}

pub fn switch_workspace(tree: &mut Tree, workspace_name: &str) -> bool {
    tree.switch_workspace(workspace_name)
}

pub fn get_workspaces(tree: &Tree) -> Vec<(NodeId, String, bool)> {
    tree.workspaces()
}

pub fn span_workspace_outputs(tree: &mut Tree, workspace: NodeId, outputs: &[NodeId]) -> bool {
    tree.span_workspace(workspace, outputs)
}

pub fn rename_workspace(tree: &mut Tree, old_name: &str, new_name: &str) -> bool {
    tree.rename_workspace(old_name, new_name)
}
