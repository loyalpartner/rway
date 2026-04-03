// Arena-allocated N-ary tree for i3/Sway-compatible tiling layout

use crate::error::TilingError;

/// Gap configuration: controls spacing between windows and screen edges
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GapsConfig {
    /// Gap between adjacent windows (pixels)
    pub inner: i32,
    /// Gap between windows and screen edges (pixels)
    pub outer: i32,
}

/// Focus movement direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Resize axis (local to rway-tiling, no dependency on rway-config)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAxis {
    Width,
    Height,
}

/// Rectangle area describing a node's geometry and position
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if the rectangle has valid positive dimensions
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Node identifier (arena index wrapper)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    /// Return the raw arena index
    pub fn index(self) -> usize {
        self.0
    }
}

/// Layout type, corresponding to i3/Sway's four container layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    SplitH,
    SplitV,
    Tabbed,
    Stacked,
}

/// Business data carried by a node
#[derive(Debug, Clone)]
pub enum NodeData {
    /// Virtual root node, unique per tree
    Root,

    /// Physical output (monitor)
    Output { name: String, geometry: Rect },

    /// Workspace, belonging to an output
    Workspace {
        name: String,
        output: NodeId,
        is_visible: bool,
    },

    /// Split/Tabbed/Stacked container
    Container {
        layout: Layout,
        /// Proportional sizes of children (sum can be any positive number)
        sizes: Vec<f64>,
        /// Index of the currently focused child
        focused_child: usize,
    },

    /// Actual window
    Window {
        window_id: u64,
        floating: bool,
        fullscreen: bool,
        fullscreen_global: bool,
        sticky: bool,
        marks: Vec<String>,
        geometry: Rect,
        /// Saved geometry before entering floating/fullscreen
        saved_geometry: Option<Rect>,
    },
}

/// A single node in the arena tree
#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub data: NodeData,
}

/// Arena-allocated N-ary tree
///
/// Uses `Vec<Option<Node>>` for node storage with a free list for slot reuse.
pub struct Tree {
    nodes: Vec<Option<Node>>,
    free_list: Vec<NodeId>,
    root: NodeId,
    /// Current focus level (for focus_parent/focus_child), None means leaf window
    pub focus_node: Option<NodeId>,
}

impl Tree {
    /// Create a new tree with a root node
    pub fn new() -> Self {
        let root_node = Node {
            parent: None,
            children: Vec::new(),
            data: NodeData::Root,
        };
        Tree {
            nodes: vec![Some(root_node)],
            free_list: Vec::new(),
            root: NodeId(0),
            focus_node: None,
        }
    }

    /// Root node ID
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Add a child node under parent, returns the new node's ID
    pub fn add_node(&mut self, parent: NodeId, data: NodeData) -> NodeId {
        let id = if let Some(reused) = self.free_list.pop() {
            reused
        } else {
            let idx = self.nodes.len();
            self.nodes.push(None);
            NodeId(idx)
        };

        let node = Node {
            parent: Some(parent),
            children: Vec::new(),
            data,
        };
        self.nodes[id.0] = Some(node);

        // Add self to parent's children list
        if let Some(parent_node) = self.nodes[parent.0].as_mut() {
            parent_node.children.push(id);
        }

        id
    }

    /// Remove node and all descendants, freed slots go to free_list
    pub fn remove_node(&mut self, id: NodeId) {
        // Collect all descendants (including self) iteratively to avoid stack overflow
        let mut to_remove = Vec::new();
        let mut stack = vec![id];
        while let Some(cur) = stack.pop() {
            if let Some(node) = self.nodes[cur.0].as_ref() {
                for &child in &node.children {
                    stack.push(child);
                }
            }
            to_remove.push(cur);
        }

        // Remove id from parent's children list
        if let Some(node) = self.nodes[id.0].as_ref() {
            if let Some(parent_id) = node.parent {
                if let Some(parent_node) = self.nodes[parent_id.0].as_mut() {
                    parent_node.children.retain(|&c| c != id);
                }
            }
        }

        // Clear slots and recycle
        for rid in to_remove {
            self.nodes[rid.0] = None;
            self.free_list.push(rid);
        }
    }

    /// Immutable reference to node
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)?.as_ref()
    }

    /// Mutable reference to node
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.0)?.as_mut()
    }

    /// Return child node ID slice
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        match self.nodes.get(id.0) {
            Some(Some(node)) => &node.children,
            _ => &[],
        }
    }

    /// Return parent node ID
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes.get(id.0)?.as_ref()?.parent
    }

    /// Number of currently alive nodes (for testing/debugging)
    pub fn node_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_some()).count()
    }
}

// ── Lightweight enum for borrow-safe layout dispatch ────────────

enum LayoutInfo {
    PassThrough {
        children: Vec<NodeId>,
        area: Rect,
    },
    Split {
        layout: Layout,
        sizes: Vec<f64>,
        focused_child: usize,
        children: Vec<NodeId>,
    },
    Leaf {
        floating: bool,
    },
    Skip,
}

// ── Container methods ───────────────────────────────────────────

impl Tree {
    /// Append a 1.0 size entry to a container's sizes vec
    pub fn add_container_size(&mut self, container: NodeId) -> Result<(), TilingError> {
        let node = self
            .get_mut(container)
            .ok_or(TilingError::NodeNotFound(container))?;
        if let NodeData::Container { ref mut sizes, .. } = node.data {
            sizes.push(1.0);
            Ok(())
        } else {
            Err(TilingError::NotAContainer(container))
        }
    }

    /// Remove the size at `child_index` and adjust focused_child
    pub fn remove_container_size(
        &mut self,
        container: NodeId,
        child_index: usize,
    ) -> Result<(), TilingError> {
        let node = self
            .get_mut(container)
            .ok_or(TilingError::NodeNotFound(container))?;
        if let NodeData::Container {
            ref mut sizes,
            ref mut focused_child,
            ..
        } = node.data
        {
            if child_index < sizes.len() {
                sizes.remove(child_index);
            }
            if !sizes.is_empty() && *focused_child >= sizes.len() {
                *focused_child = sizes.len() - 1;
            } else if sizes.is_empty() {
                *focused_child = 0;
            }
            Ok(())
        } else {
            Err(TilingError::NotAContainer(container))
        }
    }

    /// Set the layout type of a container
    pub fn set_layout(&mut self, container: NodeId, layout: Layout) -> Result<(), TilingError> {
        let node = self
            .get_mut(container)
            .ok_or(TilingError::NodeNotFound(container))?;
        if let NodeData::Container {
            layout: ref mut l, ..
        } = node.data
        {
            *l = layout;
            Ok(())
        } else {
            Err(TilingError::NotAContainer(container))
        }
    }

    /// Get the currently focused child of a container
    pub fn focused_child(&self, container: NodeId) -> Result<Option<NodeId>, TilingError> {
        let node = self
            .get(container)
            .ok_or(TilingError::NodeNotFound(container))?;
        let focused_index = match &node.data {
            NodeData::Container { focused_child, .. } => *focused_child,
            _ => return Err(TilingError::NotAContainer(container)),
        };
        let children = self.children(container);
        Ok(children.get(focused_index).copied())
    }

    /// Set the focused child of a container to the given child node
    pub fn set_focused_child(
        &mut self,
        container: NodeId,
        child: NodeId,
    ) -> Result<(), TilingError> {
        let children = self.children(container).to_vec();
        let index = children
            .iter()
            .position(|&c| c == child)
            .ok_or(TilingError::NotAContainer(container))?;

        let node = self
            .get_mut(container)
            .ok_or(TilingError::NodeNotFound(container))?;
        if let NodeData::Container {
            ref mut focused_child,
            ..
        } = node.data
        {
            *focused_child = index;
            Ok(())
        } else {
            Err(TilingError::NotAContainer(container))
        }
    }
}

// ── Workspace methods ───────────────────────────────────────────

impl Tree {
    /// Register a physical output (monitor) under the root node
    pub fn add_output(&mut self, name: &str, geometry: Rect) -> NodeId {
        let root = self.root();
        self.add_node(
            root,
            NodeData::Output {
                name: name.to_string(),
                geometry,
            },
        )
    }

    /// Create a workspace under the given output. Returns existing if same name exists.
    pub fn add_workspace(&mut self, output: NodeId, name: &str) -> NodeId {
        let existing: Vec<NodeId> = self.children(output).to_vec();
        for child_id in existing {
            if let Some(node) = self.get(child_id) {
                if let NodeData::Workspace { name: ws_name, .. } = &node.data {
                    if ws_name == name {
                        return child_id;
                    }
                }
            }
        }

        self.add_node(
            output,
            NodeData::Workspace {
                name: name.to_string(),
                output,
                is_visible: true,
            },
        )
    }

    /// Get the currently visible (focused) workspace
    pub fn focused_workspace(&self) -> Option<NodeId> {
        let root = self.root();
        for &output_id in self.children(root) {
            for &ws_id in self.children(output_id) {
                if let Some(node) = self.get(ws_id) {
                    if let NodeData::Workspace { is_visible, .. } = &node.data {
                        if *is_visible {
                            return Some(ws_id);
                        }
                    }
                }
            }
        }
        None
    }

    /// Switch to the workspace with the given name
    pub fn switch_workspace(&mut self, name: &str) -> bool {
        let target = self.find_workspace_by_name(name);
        let (target_id, output_id) = match target {
            Some(v) => v,
            None => return false,
        };

        let siblings: Vec<NodeId> = self.children(output_id).to_vec();
        for ws_id in siblings {
            if let Some(node) = self.get_mut(ws_id) {
                if let NodeData::Workspace {
                    ref mut is_visible, ..
                } = node.data
                {
                    *is_visible = ws_id == target_id;
                }
            }
        }
        true
    }

    /// Return all workspaces as `(id, name, is_visible)` tuples
    pub fn workspaces(&self) -> Vec<(NodeId, String, bool)> {
        let root = self.root();
        let mut result = Vec::new();

        for &output_id in self.children(root) {
            for &ws_id in self.children(output_id) {
                if let Some(node) = self.get(ws_id) {
                    if let NodeData::Workspace {
                        name, is_visible, ..
                    } = &node.data
                    {
                        result.push((ws_id, name.clone(), *is_visible));
                    }
                }
            }
        }
        result
    }

    /// Span a workspace across multiple outputs
    pub fn span_workspace(&mut self, workspace: NodeId, outputs: &[NodeId]) -> bool {
        let ws_name = match self.get(workspace) {
            Some(node) => match &node.data {
                NodeData::Workspace { name, .. } => name.clone(),
                _ => return false,
            },
            None => return false,
        };

        let mut any_added = false;
        for &output_id in outputs {
            let is_output = self
                .get(output_id)
                .map(|n| matches!(n.data, NodeData::Output { .. }))
                .unwrap_or(false);
            if !is_output {
                continue;
            }

            let same_output = self
                .get(workspace)
                .and_then(|n| {
                    if let NodeData::Workspace { output, .. } = &n.data {
                        Some(*output == output_id)
                    } else {
                        None
                    }
                })
                .unwrap_or(false);
            if same_output {
                continue;
            }

            let children_before = self.children(output_id).len();
            let _new_ws = self.add_workspace(output_id, &ws_name);
            let children_after = self.children(output_id).len();
            if children_after > children_before {
                any_added = true;
            }
        }
        any_added
    }

    /// Rename a workspace
    pub fn rename_workspace(&mut self, old_name: &str, new_name: &str) -> bool {
        let root = self.root();
        let outputs: Vec<NodeId> = self.children(root).to_vec();
        for output_id in outputs {
            let ws_ids: Vec<NodeId> = self.children(output_id).to_vec();
            for ws_id in ws_ids {
                if let Some(node) = self.get_mut(ws_id) {
                    if let NodeData::Workspace { ref mut name, .. } = node.data {
                        if name.as_str() == old_name {
                            *name = new_name.to_string();
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Find a workspace by name, returns (workspace_id, output_id)
    fn find_workspace_by_name(&self, name: &str) -> Option<(NodeId, NodeId)> {
        let root = self.root();
        for &output_id in self.children(root) {
            for &ws_id in self.children(output_id) {
                if let Some(node) = self.get(ws_id) {
                    if let NodeData::Workspace { name: ws_name, .. } = &node.data {
                        if ws_name == name {
                            return Some((ws_id, output_id));
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Command methods ─────────────────────────────────────────────

impl Tree {
    /// Insert a new window in the focused workspace's focused position
    pub fn insert_window(&mut self, window_id: u64) -> NodeId {
        self.insert_window_with_layout(window_id, Layout::SplitH)
    }

    /// Insert a new window using the specified layout for the wrapping container
    ///
    /// In Sway, `splith`/`splitv` sets the direction for the next window insertion.
    /// This method allows specifying that direction.
    pub fn insert_window_with_layout(&mut self, window_id: u64, layout: Layout) -> NodeId {
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => self.root(),
        };

        self.insert_window_into(ws_id, window_id, layout)
    }

    /// Remove the window with the given window_id from the tree
    pub fn remove_window(&mut self, window_id: u64) -> bool {
        let win_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };

        let parent_id = self.parent(win_id);
        let child_index =
            parent_id.and_then(|pid| self.children(pid).iter().position(|&c| c == win_id));

        self.remove_node(win_id);

        if let (Some(pid), Some(idx)) = (parent_id, child_index) {
            // Ignore errors from remove_container_size (node might not be a container)
            let _ = self.remove_container_size(pid, idx);
            self.cleanup_empty_container(pid);
        }

        true
    }

    /// Move focus in the given direction within the focused workspace
    pub fn move_focus(&mut self, direction: Direction) -> bool {
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => return false,
        };

        self.move_focus_in(ws_id, direction)
    }

    /// Change the layout of the focused container
    pub fn split(&mut self, layout: Layout) {
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => return,
        };

        if let Some(container_id) = self.find_focused_container(ws_id) {
            let _ = self.set_layout(container_id, layout);
        }
    }

    /// Toggle the floating state of a window
    pub fn toggle_floating(&mut self, window_id: u64) -> bool {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };
        if let Some(node) = self.get_mut(win_node_id) {
            if let NodeData::Window {
                ref mut floating, ..
            } = node.data
            {
                *floating = !*floating;
                return true;
            }
        }
        false
    }

    /// Find the window_id of the focused leaf window
    pub fn focused_window_id(&self) -> Option<u64> {
        let ws_id = self.focused_workspace()?;
        self.find_focused_leaf(ws_id)
    }

    /// Move the focused window in the given direction (Sway behavior).
    ///
    /// Finds the actually-focused leaf window, then swaps it with its
    /// neighbor in the parent container that matches the direction axis.
    pub fn move_window(&mut self, direction: Direction) -> bool {
        // 1. Find the focused leaf window node
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => return false,
        };
        let leaf_id = match self.find_focused_leaf_node(ws_id) {
            Some(id) => id,
            None => return false,
        };

        // 2. Walk up from the leaf to find the nearest container whose layout
        //    matches the direction axis (SplitH for Left/Right, SplitV for Up/Down)
        let mut current = leaf_id;
        loop {
            let parent_id = match self.parent(current) {
                Some(id) => id,
                None => return false,
            };

            let parent_info = match self.get(parent_id) {
                Some(n) => match &n.data {
                    NodeData::Container { layout, .. } => Some((*layout, n.children.to_vec())),
                    NodeData::Workspace { .. } => None,
                    _ => None,
                },
                None => return false,
            };

            let Some((layout, children)) = parent_info else {
                return false; // reached workspace without finding matching container
            };

            let axis_matches = matches!(
                (layout, direction),
                (Layout::SplitH, Direction::Left | Direction::Right)
                    | (Layout::SplitV, Direction::Up | Direction::Down)
            );

            if axis_matches {
                // Found the right container — swap `current` with its neighbor
                let Some(idx) = children.iter().position(|&c| c == current) else {
                    return false;
                };

                let new_pos = match direction {
                    Direction::Left | Direction::Up => {
                        if idx > 0 { idx - 1 } else { return false; }
                    }
                    Direction::Right | Direction::Down => {
                        if idx + 1 < children.len() { idx + 1 } else { return false; }
                    }
                };

                if let Some(node) = self.get_mut(parent_id) {
                    node.children.swap(idx, new_pos);
                    if let NodeData::Container {
                        ref mut sizes,
                        ref mut focused_child,
                        ..
                    } = node.data
                    {
                        if idx < sizes.len() && new_pos < sizes.len() {
                            sizes.swap(idx, new_pos);
                        }
                        *focused_child = new_pos;
                    }
                }
                return true;
            }

            // This container's axis doesn't match — walk up
            current = parent_id;
        }
    }

    /// Move the focused window to the target workspace
    pub fn move_to_workspace(&mut self, target_ws: &str) -> bool {
        let win_id = match self.focused_window_id() {
            Some(id) => id,
            None => return false,
        };

        let target_ws_id = self.find_workspace_by_name_global(target_ws);
        let target_ws_id = match target_ws_id {
            Some(id) => id,
            None => return false,
        };

        self.remove_window(win_id);
        self.insert_window_into(target_ws_id, win_id, Layout::SplitH);
        true
    }

    /// Resize a node's proportion in its parent container
    pub fn resize_container(&mut self, node_id: NodeId, axis: ResizeAxis, delta_ppt: f64) -> bool {
        let parent_id = match self.parent(node_id) {
            Some(id) => id,
            None => return false,
        };

        let layout_matches = match self.get(parent_id) {
            Some(n) => match &n.data {
                NodeData::Container { layout, .. } => matches!(
                    (axis, layout),
                    (ResizeAxis::Width, Layout::SplitH) | (ResizeAxis::Height, Layout::SplitV)
                ),
                _ => false,
            },
            None => false,
        };
        if !layout_matches {
            return false;
        }

        let children: Vec<NodeId> = self.children(parent_id).to_vec();
        let my_index = match children.iter().position(|&c| c == node_id) {
            Some(i) => i,
            None => return false,
        };

        let sibling_index = if my_index + 1 < children.len() {
            my_index + 1
        } else if my_index > 0 {
            my_index - 1
        } else {
            return false;
        };

        if let Some(node) = self.get_mut(parent_id) {
            if let NodeData::Container { ref mut sizes, .. } = node.data {
                if my_index < sizes.len() && sibling_index < sizes.len() {
                    let delta_norm = delta_ppt / 100.0;
                    sizes[my_index] += delta_norm;
                    sizes[sibling_index] -= delta_norm;
                    if sizes[my_index] < 0.05 {
                        let diff = 0.05 - sizes[my_index];
                        sizes[my_index] = 0.05;
                        sizes[sibling_index] -= diff;
                    }
                    if sizes[sibling_index] < 0.05 {
                        let diff = 0.05 - sizes[sibling_index];
                        sizes[sibling_index] = 0.05;
                        sizes[my_index] -= diff;
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Move focus up to parent container
    pub fn focus_parent(&mut self) -> bool {
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => return false,
        };

        let current = self
            .focus_node
            .unwrap_or_else(|| self.find_focused_leaf_node(ws_id).unwrap_or(ws_id));

        let parent = self.parent(current);
        match parent {
            Some(pid) if pid != self.root() => {
                self.focus_node = Some(pid);
                true
            }
            _ => false,
        }
    }

    /// Move focus down to focused child
    pub fn focus_child(&mut self) -> bool {
        let current = match self.focus_node {
            Some(id) => id,
            None => return false,
        };

        // Extract what we need without cloning the whole NodeData
        let action = match self.get(current) {
            Some(n) => match &n.data {
                NodeData::Container { focused_child, .. } => {
                    let children = self.children(current).to_vec();
                    let idx = (*focused_child).min(children.len().saturating_sub(1));
                    Some(('c', children, idx))
                }
                NodeData::Workspace { .. } => {
                    let children = self.children(current).to_vec();
                    Some(('w', children, 0))
                }
                _ => None,
            },
            None => None,
        };

        match action {
            Some(('c', children, idx)) => {
                if let Some(&child_id) = children.get(idx) {
                    let is_leaf = self
                        .get(child_id)
                        .map(|n| matches!(n.data, NodeData::Window { .. }))
                        .unwrap_or(false);
                    self.focus_node = if is_leaf { None } else { Some(child_id) };
                    true
                } else {
                    false
                }
            }
            Some(('w', children, _)) => {
                if let Some(&child_id) = children.first() {
                    let is_leaf = self
                        .get(child_id)
                        .map(|n| matches!(n.data, NodeData::Window { .. }))
                        .unwrap_or(false);
                    self.focus_node = if is_leaf { None } else { Some(child_id) };
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Set window floating state
    pub fn set_floating(&mut self, window_id: u64, enable: bool) -> bool {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };
        if let Some(node) = self.get_mut(win_node_id) {
            if let NodeData::Window {
                ref mut floating, ..
            } = node.data
            {
                *floating = enable;
                return true;
            }
        }
        false
    }

    /// Set window fullscreen state
    pub fn set_fullscreen(&mut self, window_id: u64, enable: bool) -> bool {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };
        if let Some(node) = self.get_mut(win_node_id) {
            if let NodeData::Window {
                ref mut fullscreen,
                ref mut saved_geometry,
                ref geometry,
                ..
            } = node.data
            {
                if enable && !*fullscreen {
                    *saved_geometry = Some(*geometry);
                }
                *fullscreen = enable;
                return true;
            }
        }
        false
    }

    /// Toggle window fullscreen state
    pub fn toggle_fullscreen(&mut self, window_id: u64) -> bool {
        let current = self.find_window_by_id(window_id).and_then(|id| {
            self.get(id).and_then(|n| {
                if let NodeData::Window { fullscreen, .. } = &n.data {
                    Some(!*fullscreen)
                } else {
                    None
                }
            })
        });
        match current {
            Some(new_state) => self.set_fullscreen(window_id, new_state),
            None => false,
        }
    }

    /// Public version of find_window_by_id (for external crates)
    pub fn find_node_by_window_id(&self, window_id: u64) -> Option<NodeId> {
        self.find_window_by_id(window_id)
    }

    /// Toggle focused container's layout type in cycle:
    /// SplitH -> SplitV -> Tabbed -> Stacked -> SplitH
    pub fn layout_toggle(&mut self) {
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => return,
        };

        let container_id = match self.find_focused_container(ws_id) {
            Some(id) => id,
            None => return,
        };

        let next_layout = match self.get(container_id) {
            Some(node) => match &node.data {
                NodeData::Container { layout, .. } => match layout {
                    Layout::SplitH => Layout::SplitV,
                    Layout::SplitV => Layout::Tabbed,
                    Layout::Tabbed => Layout::Stacked,
                    Layout::Stacked => Layout::SplitH,
                },
                _ => return,
            },
            None => return,
        };

        let _ = self.set_layout(container_id, next_layout);
    }

    /// Set window sticky flag
    pub fn set_sticky(&mut self, window_id: u64, enable: bool) -> bool {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };
        if let Some(node) = self.get_mut(win_node_id) {
            if let NodeData::Window { ref mut sticky, .. } = node.data {
                *sticky = enable;
                return true;
            }
        }
        false
    }

    /// Toggle window sticky flag
    pub fn toggle_sticky(&mut self, window_id: u64) -> bool {
        let current = self.find_window_by_id(window_id).and_then(|id| {
            self.get(id).and_then(|n| {
                if let NodeData::Window { sticky, .. } = &n.data {
                    Some(!*sticky)
                } else {
                    None
                }
            })
        });
        match current {
            Some(new_state) => self.set_sticky(window_id, new_state),
            None => false,
        }
    }

    /// Add a mark to a window
    pub fn add_mark(&mut self, window_id: u64, mark: &str) -> bool {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };
        if let Some(node) = self.get_mut(win_node_id) {
            if let NodeData::Window { ref mut marks, .. } = node.data {
                if !marks.iter().any(|m| m == mark) {
                    marks.push(mark.to_string());
                }
                return true;
            }
        }
        false
    }

    /// Remove a mark from a window. If `mark` is None, remove all marks.
    pub fn remove_mark(&mut self, window_id: u64, mark: Option<&str>) -> bool {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };
        if let Some(node) = self.get_mut(win_node_id) {
            if let NodeData::Window { ref mut marks, .. } = node.data {
                match mark {
                    Some(m) => marks.retain(|existing| existing != m),
                    None => marks.clear(),
                }
                return true;
            }
        }
        false
    }

    /// Get all marks on a window
    pub fn get_marks(&self, window_id: u64) -> Vec<String> {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return Vec::new(),
        };
        match self.get(win_node_id) {
            Some(node) => match &node.data {
                NodeData::Window { marks, .. } => marks.clone(),
                _ => Vec::new(),
            },
            None => Vec::new(),
        }
    }

    /// Set window fullscreen_global flag
    pub fn set_fullscreen_global(&mut self, window_id: u64, enable: bool) -> bool {
        let win_node_id = match self.find_window_by_id(window_id) {
            Some(id) => id,
            None => return false,
        };
        if let Some(node) = self.get_mut(win_node_id) {
            if let NodeData::Window {
                ref mut fullscreen_global,
                ref mut saved_geometry,
                ref geometry,
                ..
            } = node.data
            {
                if enable && !*fullscreen_global {
                    *saved_geometry = Some(*geometry);
                }
                *fullscreen_global = enable;
                return true;
            }
        }
        false
    }

    /// Toggle window fullscreen_global flag
    pub fn toggle_fullscreen_global(&mut self, window_id: u64) -> bool {
        let current = self.find_window_by_id(window_id).and_then(|id| {
            self.get(id).and_then(|n| {
                if let NodeData::Window {
                    fullscreen_global, ..
                } = &n.data
                {
                    Some(!*fullscreen_global)
                } else {
                    None
                }
            })
        });
        match current {
            Some(new_state) => self.set_fullscreen_global(window_id, new_state),
            None => false,
        }
    }

    /// Swap two sibling nodes under the same parent
    pub fn swap_containers(&mut self, node_a: NodeId, node_b: NodeId) -> bool {
        let parent_a = self.parent(node_a);
        let parent_b = self.parent(node_b);
        let parent_id = match (parent_a, parent_b) {
            (Some(a), Some(b)) if a == b => a,
            _ => return false,
        };

        let children: Vec<NodeId> = self.children(parent_id).to_vec();
        let idx_a = match children.iter().position(|&c| c == node_a) {
            Some(i) => i,
            None => return false,
        };
        let idx_b = match children.iter().position(|&c| c == node_b) {
            Some(i) => i,
            None => return false,
        };

        if idx_a == idx_b {
            return false;
        }

        if let Some(parent_node) = self.get_mut(parent_id) {
            parent_node.children.swap(idx_a, idx_b);
            if let NodeData::Container { ref mut sizes, .. } = parent_node.data {
                if idx_a < sizes.len() && idx_b < sizes.len() {
                    sizes.swap(idx_a, idx_b);
                }
            }
        }

        true
    }

    /// Move focus to the next sibling in the current container
    pub fn focus_next_sibling(&mut self) -> bool {
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => return false,
        };

        let leaf_id = match self.find_focused_leaf_node(ws_id) {
            Some(id) => id,
            None => return false,
        };

        let parent_id = match self.parent(leaf_id) {
            Some(id) => id,
            None => return false,
        };

        let children: Vec<NodeId> = self.children(parent_id).to_vec();
        let current_idx = match children.iter().position(|&c| c == leaf_id) {
            Some(i) => i,
            None => return false,
        };

        if current_idx + 1 >= children.len() {
            return false;
        }

        let new_idx = current_idx + 1;
        if let Some(node) = self.get_mut(parent_id) {
            if let NodeData::Container {
                ref mut focused_child,
                ..
            } = node.data
            {
                *focused_child = new_idx;
                return true;
            }
        }
        false
    }

    /// Move focus to the previous sibling in the current container
    pub fn focus_prev_sibling(&mut self) -> bool {
        let ws_id = match self.focused_workspace() {
            Some(id) => id,
            None => return false,
        };

        let leaf_id = match self.find_focused_leaf_node(ws_id) {
            Some(id) => id,
            None => return false,
        };

        let parent_id = match self.parent(leaf_id) {
            Some(id) => id,
            None => return false,
        };

        let children: Vec<NodeId> = self.children(parent_id).to_vec();
        let current_idx = match children.iter().position(|&c| c == leaf_id) {
            Some(i) => i,
            None => return false,
        };

        if current_idx == 0 {
            return false;
        }

        let new_idx = current_idx - 1;
        if let Some(node) = self.get_mut(parent_id) {
            if let NodeData::Container {
                ref mut focused_child,
                ..
            } = node.data
            {
                *focused_child = new_idx;
                return true;
            }
        }
        false
    }

    // ── Private command helpers ──────────────────────────────────

    fn insert_window_into(
        &mut self,
        parent_id: NodeId,
        window_id: u64,
        split_layout: Layout,
    ) -> NodeId {
        let new_win_data = NodeData::Window {
            window_id,
            floating: false,
            fullscreen: false,
            fullscreen_global: false,
            sticky: false,
            marks: Vec::new(),
            geometry: Rect::new(0, 0, 0, 0),
            saved_geometry: None,
        };

        let children: Vec<NodeId> = self.children(parent_id).to_vec();

        if children.is_empty() {
            return self.add_node(parent_id, new_win_data);
        }

        // Check if parent is a Container with the same layout direction
        let parent_layout = self.get(parent_id).and_then(|n| match &n.data {
            NodeData::Container { layout, .. } => Some(*layout),
            _ => None,
        });

        let focused_idx = match self.get(parent_id) {
            Some(n) => match &n.data {
                NodeData::Container { focused_child, .. } => *focused_child,
                NodeData::Workspace { .. } => 0,
                _ => 0,
            },
            None => 0,
        };
        let focused_id = children.get(focused_idx).copied().unwrap_or(children[0]);

        let is_win = self
            .get(focused_id)
            .map(|n| matches!(n.data, NodeData::Window { .. }))
            .unwrap_or(false);

        if is_win {
            // Sway behavior: if parent container has the same layout direction,
            // add new window as a sibling (not nested). Only wrap in a new
            // sub-container when the split direction differs.
            if parent_layout == Some(split_layout) {
                // Add as sibling in the same container — equal share
                let new_id = self.add_node(parent_id, new_win_data);
                let _ = self.add_container_size(parent_id);
                // Update focused_child to the new window
                let new_count = self.children(parent_id).len();
                if let Some(node) = self.get_mut(parent_id) {
                    if let NodeData::Container {
                        ref mut focused_child,
                        ..
                    } = node.data
                    {
                        *focused_child = new_count - 1;
                    }
                }
                return new_id;
            }
            self.wrap_with_container(
                parent_id,
                focused_id,
                focused_idx,
                new_win_data,
                split_layout,
            )
        } else {
            self.insert_window_into(focused_id, window_id, split_layout)
        }
    }

    fn wrap_with_container(
        &mut self,
        parent_id: NodeId,
        existing_win: NodeId,
        child_index: usize,
        new_win_data: NodeData,
        split_layout: Layout,
    ) -> NodeId {
        // 1. Remove existing window from parent's children list (keep the slot for replacement)
        if let Some(parent_node) = self.get_mut(parent_id) {
            parent_node.children.retain(|&c| c != existing_win);
        }
        // Remove the size entry — we'll re-insert at the same position
        let _ = self.remove_container_size(parent_id, child_index);

        // 2. Orphan the existing window temporarily
        if let Some(win_node) = self.get_mut(existing_win) {
            win_node.parent = None;
        }

        // 3. Create new container (allocate node, but DON'T use add_node which appends to end)
        let container_data = NodeData::Container {
            layout: split_layout,
            sizes: vec![1.0, 1.0],
            focused_child: 1, // new window is focused
        };

        // Allocate the container node ID
        let container_id = if let Some(reused) = self.free_list.pop() {
            reused
        } else {
            let idx = self.nodes.len();
            self.nodes.push(None);
            NodeId(idx)
        };
        self.nodes[container_id.0] = Some(Node {
            parent: Some(parent_id),
            children: Vec::new(),
            data: container_data,
        });

        // 4. Insert container at the ORIGINAL position in parent (not appended to end)
        if let Some(parent_node) = self.get_mut(parent_id) {
            let insert_pos = child_index.min(parent_node.children.len());
            parent_node.children.insert(insert_pos, container_id);
            // Re-add size entry at the same position
            if let NodeData::Container {
                ref mut sizes,
                ref mut focused_child,
                ..
            } = parent_node.data
            {
                sizes.insert(insert_pos, 1.0);
                *focused_child = insert_pos;
            }
        }

        // 5. Re-parent existing window under container
        if let Some(win_node) = self.get_mut(existing_win) {
            win_node.parent = Some(container_id);
        }
        if let Some(container_node) = self.get_mut(container_id) {
            container_node.children.push(existing_win);
        }

        // 6. Add new window to container
        let new_win_id = self.add_node(container_id, new_win_data);

        new_win_id
    }

    fn find_window_by_id(&self, window_id: u64) -> Option<NodeId> {
        self.find_window_recursive(self.root(), window_id)
    }

    fn find_window_recursive(&self, node_id: NodeId, window_id: u64) -> Option<NodeId> {
        let node = self.get(node_id)?;
        if let NodeData::Window { window_id: wid, .. } = &node.data {
            if *wid == window_id {
                return Some(node_id);
            }
        }
        let len = node.children.len();
        for i in 0..len {
            let child = self.children(node_id)[i];
            if let Some(found) = self.find_window_recursive(child, window_id) {
                return Some(found);
            }
        }
        None
    }

    fn cleanup_empty_container(&mut self, node_id: NodeId) {
        let is_empty_container = self
            .get(node_id)
            .map(|n| matches!(n.data, NodeData::Container { .. }) && n.children.is_empty())
            .unwrap_or(false);

        if is_empty_container {
            let parent = self.parent(node_id);
            let child_index =
                parent.and_then(|pid| self.children(pid).iter().position(|&c| c == node_id));
            self.remove_node(node_id);
            if let (Some(pid), Some(idx)) = (parent, child_index) {
                let _ = self.remove_container_size(pid, idx);
                self.cleanup_empty_container(pid);
            }
        }
    }

    fn move_focus_in(&mut self, node_id: NodeId, direction: Direction) -> bool {
        // Extract info without cloning NodeData
        let info = match self.get(node_id) {
            Some(n) => match &n.data {
                NodeData::Container {
                    layout,
                    focused_child,
                    ..
                } => {
                    let layout = *layout;
                    let focused = *focused_child;
                    let children: Vec<NodeId> = self.children(node_id).to_vec();
                    Some(('c', layout, focused, children))
                }
                NodeData::Workspace { .. } => {
                    let children: Vec<NodeId> = self.children(node_id).to_vec();
                    Some(('w', Layout::SplitH, 0, children))
                }
                _ => None,
            },
            None => None,
        };

        match info {
            Some(('c', layout, focused, children)) => {
                // Try recursing into focused child first
                if let Some(&focused_id) = children.get(focused) {
                    if self.move_focus_in(focused_id, direction) {
                        return true;
                    }
                }

                let can_move = matches!(
                    (layout, direction),
                    (Layout::SplitH, Direction::Left)
                        | (Layout::SplitH, Direction::Right)
                        | (Layout::SplitV, Direction::Up)
                        | (Layout::SplitV, Direction::Down)
                );

                if can_move && !children.is_empty() {
                    let new_focus = match direction {
                        Direction::Left | Direction::Up => {
                            if focused > 0 {
                                focused - 1
                            } else {
                                return false;
                            }
                        }
                        Direction::Right | Direction::Down => {
                            if focused + 1 < children.len() {
                                focused + 1
                            } else {
                                return false;
                            }
                        }
                    };

                    if let Some(node) = self.get_mut(node_id) {
                        if let NodeData::Container {
                            ref mut focused_child,
                            ..
                        } = node.data
                        {
                            *focused_child = new_focus;
                        }
                    }
                    return true;
                }
                false
            }
            Some(('w', _, _, children)) => {
                for &child in &children {
                    if self.move_focus_in(child, direction) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn find_focused_container(&self, node_id: NodeId) -> Option<NodeId> {
        let info = match self.get(node_id) {
            Some(n) => match &n.data {
                NodeData::Container { focused_child, .. } => Some(('c', *focused_child)),
                NodeData::Workspace { .. } => Some(('w', 0)),
                _ => None,
            },
            None => None,
        };

        match info {
            Some(('c', focused_child)) => {
                let children: Vec<NodeId> = self.children(node_id).to_vec();
                if let Some(&focused_id) = children.get(focused_child) {
                    if let Some(deeper) = self.find_focused_container(focused_id) {
                        return Some(deeper);
                    }
                }
                Some(node_id)
            }
            Some(('w', _)) => {
                let children: Vec<NodeId> = self.children(node_id).to_vec();
                for &child in &children {
                    if let Some(found) = self.find_focused_container(child) {
                        return Some(found);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn find_focused_leaf(&self, node_id: NodeId) -> Option<u64> {
        let node = self.get(node_id)?;
        match &node.data {
            NodeData::Window { window_id, .. } => Some(*window_id),
            NodeData::Container { focused_child, .. } => {
                let children = self.children(node_id);
                let idx = (*focused_child).min(children.len().saturating_sub(1));
                children.get(idx).and_then(|&c| self.find_focused_leaf(c))
            }
            _ => {
                let children = self.children(node_id);
                children.first().and_then(|&c| self.find_focused_leaf(c))
            }
        }
    }

    fn find_focused_leaf_node(&self, node_id: NodeId) -> Option<NodeId> {
        let node = self.get(node_id)?;
        match &node.data {
            NodeData::Window { .. } => Some(node_id),
            NodeData::Container { focused_child, .. } => {
                let children = self.children(node_id);
                let idx = (*focused_child).min(children.len().saturating_sub(1));
                children
                    .get(idx)
                    .and_then(|&c| self.find_focused_leaf_node(c))
            }
            _ => {
                let children = self.children(node_id);
                children
                    .first()
                    .and_then(|&c| self.find_focused_leaf_node(c))
            }
        }
    }

    // move_window_in removed — replaced by walk-up approach in move_window()

    fn find_workspace_by_name_global(&self, name: &str) -> Option<NodeId> {
        let root = self.root();
        for &output_id in self.children(root) {
            for &ws_id in self.children(output_id) {
                if let Some(node) = self.get(ws_id) {
                    if let NodeData::Workspace { name: ws_name, .. } = &node.data {
                        if ws_name == name {
                            return Some(ws_id);
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Layout methods ──────────────────────────────────────────────

/// Shrink a rectangle by `amount` pixels on all sides
pub fn shrink_rect(r: Rect, amount: i32) -> Rect {
    if amount == 0 {
        return r;
    }
    Rect {
        x: r.x + amount,
        y: r.y + amount,
        width: (r.width - 2 * amount).max(1),
        height: (r.height - 2 * amount).max(1),
    }
}

impl Tree {
    /// Top-down recursive layout computation from node_id
    pub fn compute_layout(&mut self, node_id: NodeId, available: Rect, gaps: &GapsConfig) {
        // Extract lightweight info to avoid borrowing issues
        let info = match self.get(node_id) {
            Some(n) => match &n.data {
                NodeData::Root => {
                    let children: Vec<NodeId> = n.children.to_vec();
                    LayoutInfo::PassThrough {
                        children,
                        area: available,
                    }
                }
                NodeData::Output { geometry, .. } => {
                    let children: Vec<NodeId> = n.children.to_vec();
                    LayoutInfo::PassThrough {
                        children,
                        area: *geometry,
                    }
                }
                NodeData::Workspace { .. } => {
                    let children: Vec<NodeId> = n.children.to_vec();
                    let inner_rect = shrink_rect(available, gaps.outer);
                    LayoutInfo::PassThrough {
                        children,
                        area: inner_rect,
                    }
                }
                NodeData::Container {
                    layout,
                    sizes,
                    focused_child,
                } => {
                    let children: Vec<NodeId> = n.children.to_vec();
                    LayoutInfo::Split {
                        layout: *layout,
                        sizes: sizes.clone(),
                        focused_child: *focused_child,
                        children,
                    }
                }
                NodeData::Window { floating, .. } => LayoutInfo::Leaf {
                    floating: *floating,
                },
            },
            None => LayoutInfo::Skip,
        };

        match info {
            LayoutInfo::PassThrough { children, area } => {
                for child in children {
                    self.compute_layout(child, area, gaps);
                }
            }
            LayoutInfo::Split {
                layout,
                sizes,
                focused_child,
                children,
            } => {
                if children.is_empty() {
                    return;
                }
                match layout {
                    Layout::SplitH => {
                        self.layout_split_h(&children, &sizes, available, gaps);
                    }
                    Layout::SplitV => {
                        self.layout_split_v(&children, &sizes, available, gaps);
                    }
                    Layout::Tabbed | Layout::Stacked => {
                        let focus_idx = focused_child.min(children.len().saturating_sub(1));
                        let focused_id = children[focus_idx];
                        self.compute_layout(focused_id, available, gaps);
                    }
                }
            }
            LayoutInfo::Leaf { floating } => {
                if !floating {
                    let final_rect = shrink_rect(available, gaps.inner / 2);
                    if let Some(node) = self.get_mut(node_id) {
                        if let NodeData::Window {
                            ref mut geometry, ..
                        } = node.data
                        {
                            *geometry = final_rect;
                        }
                    }
                }
            }
            LayoutInfo::Skip => {}
        }
    }

    /// Collect all window geometries in the tree
    pub fn window_geometries(&self) -> Vec<(u64, Rect)> {
        let mut result = Vec::new();
        self.collect_windows(self.root(), &mut result);
        result
    }

    fn layout_split_h(
        &mut self,
        children: &[NodeId],
        sizes: &[f64],
        available: Rect,
        gaps: &GapsConfig,
    ) {
        let total: f64 = if sizes.is_empty() {
            0.0
        } else {
            sizes.iter().sum()
        };

        let n = children.len();
        let mut x_offset = available.x;

        for (i, &child) in children.iter().enumerate() {
            let ratio = if total > 0.0 && i < sizes.len() {
                sizes[i] / total
            } else {
                1.0 / n as f64
            };

            let child_width = if i == n - 1 {
                available.x + available.width - x_offset
            } else {
                (available.width as f64 * ratio).round() as i32
            };

            let child_rect = Rect {
                x: x_offset,
                y: available.y,
                width: child_width.max(1),
                height: available.height,
            };

            self.compute_layout(child, child_rect, gaps);
            x_offset += child_width.max(1);
        }
    }

    fn layout_split_v(
        &mut self,
        children: &[NodeId],
        sizes: &[f64],
        available: Rect,
        gaps: &GapsConfig,
    ) {
        let total: f64 = if sizes.is_empty() {
            0.0
        } else {
            sizes.iter().sum()
        };

        let n = children.len();
        let mut y_offset = available.y;

        for (i, &child) in children.iter().enumerate() {
            let ratio = if total > 0.0 && i < sizes.len() {
                sizes[i] / total
            } else {
                1.0 / n as f64
            };

            let child_height = if i == n - 1 {
                available.y + available.height - y_offset
            } else {
                (available.height as f64 * ratio).round() as i32
            };

            let child_rect = Rect {
                x: available.x,
                y: y_offset,
                width: available.width,
                height: child_height.max(1),
            };

            self.compute_layout(child, child_rect, gaps);
            y_offset += child_height.max(1);
        }
    }

    fn collect_windows(&self, node_id: NodeId, out: &mut Vec<(u64, Rect)>) {
        if let Some(node) = self.get(node_id) {
            match &node.data {
                NodeData::Window {
                    window_id,
                    geometry,
                    ..
                } => {
                    out.push((*window_id, *geometry));
                }
                _ => {
                    let len = node.children.len();
                    for i in 0..len {
                        let child = self.children(node_id)[i];
                        self.collect_windows(child, out);
                    }
                }
            }
        }
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 单元测试（RED → GREEN → IMPROVE）
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── 辅助函数 ────────────────────────────────────────────

    fn sample_rect() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    fn output_data(name: &str) -> NodeData {
        NodeData::Output {
            name: name.to_string(),
            geometry: sample_rect(),
        }
    }

    fn window_data(id: u64) -> NodeData {
        NodeData::Window {
            window_id: id,
            floating: false,
            fullscreen: false,
            fullscreen_global: false,
            sticky: false,
            marks: Vec::new(),
            geometry: Rect::new(0, 0, 0, 0),
            saved_geometry: None,
        }
    }

    fn container_data(layout: Layout) -> NodeData {
        NodeData::Container {
            layout,
            sizes: Vec::new(),
            focused_child: 0,
        }
    }

    // ── Rect 测试 ──────────────────────────────────────────

    #[test]
    fn rect_new_stores_fields() {
        let r = Rect::new(10, 20, 800, 600);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 800);
        assert_eq!(r.height, 600);
    }

    #[test]
    fn rect_is_valid_positive_dimensions() {
        assert!(Rect::new(0, 0, 1, 1).is_valid());
        assert!(Rect::new(-100, -100, 800, 600).is_valid());
    }

    #[test]
    fn rect_is_invalid_zero_width() {
        assert!(!Rect::new(0, 0, 0, 600).is_valid());
    }

    #[test]
    fn rect_is_invalid_zero_height() {
        assert!(!Rect::new(0, 0, 800, 0).is_valid());
    }

    #[test]
    fn rect_is_invalid_negative_dimensions() {
        assert!(!Rect::new(0, 0, -1, 600).is_valid());
        assert!(!Rect::new(0, 0, 800, -1).is_valid());
    }

    #[test]
    fn rect_equality() {
        let a = Rect::new(0, 0, 100, 100);
        let b = Rect::new(0, 0, 100, 100);
        assert_eq!(a, b);
    }

    // ── Tree::new 测试 ─────────────────────────────────────

    #[test]
    fn tree_new_has_root() {
        let tree = Tree::new();
        let root = tree.root();
        assert!(tree.get(root).is_some());
    }

    #[test]
    fn tree_new_root_is_root_data() {
        let tree = Tree::new();
        let node = tree.get(tree.root()).unwrap();
        assert!(matches!(node.data, NodeData::Root));
    }

    #[test]
    fn tree_new_root_has_no_parent() {
        let tree = Tree::new();
        assert!(tree.parent(tree.root()).is_none());
    }

    #[test]
    fn tree_new_root_has_no_children() {
        let tree = Tree::new();
        assert!(tree.children(tree.root()).is_empty());
    }

    #[test]
    fn tree_new_node_count_is_one() {
        let tree = Tree::new();
        assert_eq!(tree.node_count(), 1);
    }

    // ── add_node 测试 ──────────────────────────────────────

    #[test]
    fn add_node_returns_valid_id() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        assert!(tree.get(child).is_some());
    }

    #[test]
    fn add_node_parent_has_child() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        assert!(tree.children(root).contains(&child));
    }

    #[test]
    fn add_node_child_has_parent() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        assert_eq!(tree.parent(child), Some(root));
    }

    #[test]
    fn add_node_increments_count() {
        let mut tree = Tree::new();
        let root = tree.root();
        tree.add_node(root, output_data("eDP-1"));
        assert_eq!(tree.node_count(), 2);
    }

    #[test]
    fn add_multiple_children() {
        let mut tree = Tree::new();
        let root = tree.root();
        let c1 = tree.add_node(root, output_data("eDP-1"));
        let c2 = tree.add_node(root, output_data("HDMI-1"));
        let children = tree.children(root);
        assert!(children.contains(&c1));
        assert!(children.contains(&c2));
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn add_nested_nodes() {
        let mut tree = Tree::new();
        let root = tree.root();
        let output = tree.add_node(root, output_data("eDP-1"));
        let ws = tree.add_node(
            output,
            NodeData::Workspace {
                name: "1".into(),
                output,
                is_visible: true,
            },
        );
        let win = tree.add_node(ws, window_data(42));

        assert_eq!(tree.parent(win), Some(ws));
        assert_eq!(tree.parent(ws), Some(output));
        assert_eq!(tree.parent(output), Some(root));
    }

    // ── remove_node 测试 ───────────────────────────────────

    #[test]
    fn remove_node_leaf_decrements_count() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        tree.remove_node(child);
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn remove_node_leaf_not_found_after_removal() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        tree.remove_node(child);
        assert!(tree.get(child).is_none());
    }

    #[test]
    fn remove_node_removed_from_parent_children() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        tree.remove_node(child);
        assert!(!tree.children(root).contains(&child));
    }

    #[test]
    fn remove_node_also_removes_descendants() {
        let mut tree = Tree::new();
        let root = tree.root();
        let output = tree.add_node(root, output_data("eDP-1"));
        let ws = tree.add_node(
            output,
            NodeData::Workspace {
                name: "1".into(),
                output,
                is_visible: true,
            },
        );
        let win = tree.add_node(ws, window_data(1));

        tree.remove_node(output);

        assert!(tree.get(output).is_none());
        assert!(tree.get(ws).is_none());
        assert!(tree.get(win).is_none());
        assert_eq!(tree.node_count(), 1); // 只剩 root
    }

    #[test]
    fn remove_node_recycles_slot_for_new_node() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, output_data("eDP-1"));
        let child_idx = child.0;
        tree.remove_node(child);

        // 新添加的节点应复用已释放的槽位
        let new_child = tree.add_node(root, output_data("HDMI-1"));
        assert_eq!(new_child.0, child_idx);
    }

    #[test]
    fn remove_node_sibling_remains() {
        let mut tree = Tree::new();
        let root = tree.root();
        let c1 = tree.add_node(root, output_data("eDP-1"));
        let c2 = tree.add_node(root, output_data("HDMI-1"));
        tree.remove_node(c1);
        assert!(tree.get(c2).is_some());
        assert!(tree.children(root).contains(&c2));
    }

    // ── get / get_mut 测试 ─────────────────────────────────

    #[test]
    fn get_returns_none_for_invalid_id() {
        let tree = Tree::new();
        assert!(tree.get(NodeId(999)).is_none());
    }

    #[test]
    fn get_mut_modifies_node() {
        let mut tree = Tree::new();
        let root = tree.root();
        let child = tree.add_node(root, window_data(1));

        if let Some(node) = tree.get_mut(child) {
            if let NodeData::Window {
                ref mut floating, ..
            } = node.data
            {
                *floating = true;
            }
        }

        if let Some(node) = tree.get(child) {
            if let NodeData::Window { floating, .. } = node.data {
                assert!(floating);
            } else {
                panic!("节点数据类型错误");
            }
        }
    }

    // ── children / parent 边界测试 ─────────────────────────

    #[test]
    fn children_returns_empty_for_invalid_id() {
        let tree = Tree::new();
        assert!(tree.children(NodeId(999)).is_empty());
    }

    #[test]
    fn parent_returns_none_for_invalid_id() {
        let tree = Tree::new();
        assert!(tree.parent(NodeId(999)).is_none());
    }

    // ── NodeData 克隆测试 ──────────────────────────────────

    #[test]
    fn node_data_container_clone() {
        let data = container_data(Layout::SplitH);
        let cloned = data.clone();
        assert!(matches!(
            cloned,
            NodeData::Container {
                layout: Layout::SplitH,
                ..
            }
        ));
    }

    #[test]
    fn layout_equality() {
        assert_eq!(Layout::SplitH, Layout::SplitH);
        assert_ne!(Layout::SplitH, Layout::SplitV);
    }

    // ── NodeId 哈希/相等 ───────────────────────────────────

    #[test]
    fn node_id_equality() {
        assert_eq!(NodeId(0), NodeId(0));
        assert_ne!(NodeId(0), NodeId(1));
    }

    #[test]
    fn node_id_in_hashset() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NodeId(0));
        set.insert(NodeId(1));
        set.insert(NodeId(0)); // 重复
        assert_eq!(set.len(), 2);
    }

    // ── 大量节点压力测试 ───────────────────────────────────

    #[test]
    fn add_many_nodes_and_remove_all() {
        let mut tree = Tree::new();
        let root = tree.root();
        let mut ids = Vec::new();
        for i in 0..1000u64 {
            ids.push(tree.add_node(root, window_data(i)));
        }
        assert_eq!(tree.node_count(), 1001);

        for id in &ids {
            tree.remove_node(*id);
        }
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn free_list_reuse_across_many_cycles() {
        let mut tree = Tree::new();
        let root = tree.root();

        // 添加 → 删除 → 再添加，验证复用
        for cycle in 0..10u64 {
            let id = tree.add_node(root, window_data(cycle));
            tree.remove_node(id);
        }
        // 最终只剩 root
        assert_eq!(tree.node_count(), 1);
    }

    // ── Sway-compatible sibling insertion tests ────────────────

    fn make_ws_tree() -> Tree {
        let mut tree = Tree::new();
        let root = tree.root();
        let out = tree.add_node(root, output_data("eDP-1"));
        tree.add_node(
            out,
            NodeData::Workspace {
                name: "1".into(),
                output: out,
                is_visible: true,
            },
        );
        tree
    }

    #[test]
    fn three_windows_same_direction_are_siblings() {
        // In Sway: opening 3 windows with same split direction
        // should produce [A, B, C] as siblings, not [A, [B, C]]
        let mut tree = make_ws_tree();
        tree.insert_window(1);
        tree.insert_window(2);
        tree.insert_window(3);

        // Find the container holding the windows
        let ws = tree.focused_workspace().unwrap();
        let container_id = tree.children(ws)[0];
        let siblings = tree.children(container_id);

        // All 3 windows should be direct children of the same container
        assert_eq!(
            siblings.len(),
            3,
            "3 windows should be siblings, not nested"
        );
    }

    #[test]
    fn three_siblings_get_equal_layout() {
        let mut tree = make_ws_tree();
        tree.insert_window(1);
        tree.insert_window(2);
        tree.insert_window(3);

        let root = tree.root();
        tree.compute_layout(root, sample_rect(), &GapsConfig::default());
        let geoms = tree.window_geometries();

        assert_eq!(geoms.len(), 3);
        // Each window should get ~1/3 of 1920 = 640
        for (_, g) in &geoms {
            assert!(
                g.width > 600 && g.width < 700,
                "each window should be ~640px wide, got {}",
                g.width
            );
        }
    }

    #[test]
    fn remove_window_siblings_redistribute_equally() {
        let mut tree = make_ws_tree();
        tree.insert_window(1);
        tree.insert_window(2);
        tree.insert_window(3);

        // Remove middle window
        tree.remove_window(2);

        let root = tree.root();
        tree.compute_layout(root, sample_rect(), &GapsConfig::default());
        let geoms = tree.window_geometries();

        assert_eq!(geoms.len(), 2);
        // Each should get 50% = 960px
        for (_, g) in &geoms {
            assert_eq!(g.width, 960, "after removing one, each should get half");
        }
    }

    #[test]
    fn wrap_container_preserves_position() {
        // Bug fix: wrapping window B in [A | B | C] should produce [A | [B,D] | C]
        // not [A | C | [B,D]]
        let mut tree = make_ws_tree();
        tree.insert_window(1); // A
        tree.insert_window(2); // B
        tree.insert_window(3); // C — now [A | B | C]

        // Focus B (index 1 in container)
        let ws = tree.focused_workspace().unwrap();
        let container = tree.children(ws)[0];
        if let Some(node) = tree.get_mut(container) {
            if let NodeData::Container {
                ref mut focused_child,
                ..
            } = node.data
            {
                *focused_child = 1;
            }
        }

        // Insert D with SplitV — should wrap B, not append to end
        tree.insert_window_with_layout(4, Layout::SplitV);

        // Check: container should still have 3 children [A, nested, C]
        let top_children = tree.children(container);
        assert_eq!(top_children.len(), 3, "top container should have 3 children");

        // The middle child (index 1) should be the SplitV sub-container
        let middle = top_children[1];
        assert!(
            matches!(
                tree.get(middle).map(|n| &n.data),
                Some(NodeData::Container {
                    layout: Layout::SplitV,
                    ..
                })
            ),
            "middle child should be a SplitV container"
        );

        // First child should still be window A (id=1)
        let first = top_children[0];
        assert!(
            matches!(
                tree.get(first).map(|n| &n.data),
                Some(NodeData::Window { window_id: 1, .. })
            ),
            "first child should be window A"
        );

        // Last child should still be window C (id=3)
        let last = top_children[2];
        assert!(
            matches!(
                tree.get(last).map(|n| &n.data),
                Some(NodeData::Window { window_id: 3, .. })
            ),
            "last child should be window C"
        );
    }

    #[test]
    fn move_down_swaps_within_column_only() {
        // Layout:  SplitH [ SplitV[A,C], SplitV[B,D] ]
        // Focus A, move down → SplitH [ SplitV[C,A], SplitV[B,D] ]
        let mut tree = make_ws_tree();
        tree.insert_window(1); // A
        tree.insert_window_with_layout(2, Layout::SplitV); // wrap: SplitV[A, B]
        // Hmm, this creates SplitH[SplitV[A,B]] — not what we want.
        // We need: SplitH[ SplitV[A,C], SplitV[B,D] ]

        // Build manually:
        let mut tree = Tree::new();
        let root = tree.root();
        let out = tree.add_node(root, output_data("eDP-1"));
        let ws = tree.add_node(out, NodeData::Workspace {
            name: "1".into(),
            output: out,
            is_visible: true,
        });
        let splith = tree.add_node(ws, NodeData::Container {
            layout: Layout::SplitH,
            sizes: vec![1.0, 1.0],
            focused_child: 0,
        });
        let col1 = tree.add_node(splith, NodeData::Container {
            layout: Layout::SplitV,
            sizes: vec![1.0, 1.0],
            focused_child: 0, // focus on A
        });
        let col2 = tree.add_node(splith, NodeData::Container {
            layout: Layout::SplitV,
            sizes: vec![1.0, 1.0],
            focused_child: 0,
        });
        let a = tree.add_node(col1, window_data(1)); // A
        let c = tree.add_node(col1, window_data(3)); // C
        let b = tree.add_node(col2, window_data(2)); // B
        let d = tree.add_node(col2, window_data(4)); // D

        // Focus is on A (col1.focused_child=0, splith.focused_child=0)
        // Move down: A should swap with C in col1
        let moved = tree.move_window(Direction::Down);
        assert!(moved, "move down should succeed");

        // col1 children should now be [C, A]
        let col1_children = tree.children(col1);
        assert_eq!(col1_children[0], c, "C should be first after move down");
        assert_eq!(col1_children[1], a, "A should be second after move down");

        // col2 should be unchanged: [B, D]
        let col2_children = tree.children(col2);
        assert_eq!(col2_children[0], b, "B should still be first");
        assert_eq!(col2_children[1], d, "D should still be second");
    }

    #[test]
    fn move_right_swaps_columns_not_rows() {
        // SplitH [ SplitV[A,C], SplitV[B,D] ]
        // Focus A, move right → SplitH [ SplitV[B,D], SplitV[A,C] ]
        // (the whole column swaps, because A's ancestor at SplitH level is col1)
        let mut tree = Tree::new();
        let root = tree.root();
        let out = tree.add_node(root, output_data("eDP-1"));
        let ws = tree.add_node(out, NodeData::Workspace {
            name: "1".into(), output: out, is_visible: true,
        });
        let splith = tree.add_node(ws, NodeData::Container {
            layout: Layout::SplitH, sizes: vec![1.0, 1.0], focused_child: 0,
        });
        let col1 = tree.add_node(splith, NodeData::Container {
            layout: Layout::SplitV, sizes: vec![1.0, 1.0], focused_child: 0,
        });
        let col2 = tree.add_node(splith, NodeData::Container {
            layout: Layout::SplitV, sizes: vec![1.0, 1.0], focused_child: 0,
        });
        tree.add_node(col1, window_data(1)); // A
        tree.add_node(col1, window_data(3)); // C
        tree.add_node(col2, window_data(2)); // B
        tree.add_node(col2, window_data(4)); // D

        // Focus on A, move right
        let moved = tree.move_window(Direction::Right);
        assert!(moved);

        // SplitH children should now be [col2, col1] (columns swapped)
        let h_children = tree.children(splith);
        assert_eq!(h_children[0], col2, "col2 should now be first (left)");
        assert_eq!(h_children[1], col1, "col1 should now be second (right)");
    }

    #[test]
    fn different_split_direction_creates_nesting() {
        // splitv after splith should create a nested container
        let mut tree = make_ws_tree();
        tree.insert_window(1);
        tree.insert_window(2); // default SplitH: [1 | 2]

        // Now insert with SplitV — should nest under focused window
        tree.insert_window_with_layout(3, Layout::SplitV);

        // Tree should be: Container(SplitH) [ win1, Container(SplitV) [win2, win3] ]
        let ws = tree.focused_workspace().unwrap();
        let top_container = tree.children(ws)[0];
        let top_children = tree.children(top_container);
        assert_eq!(
            top_children.len(),
            2,
            "top level should still have 2 children"
        );

        // One of them should be a nested SplitV container
        let nested = top_children.iter().find(|&&id| {
            tree.get(id)
                .map(|n| {
                    matches!(
                        n.data,
                        NodeData::Container {
                            layout: Layout::SplitV,
                            ..
                        }
                    )
                })
                .unwrap_or(false)
        });
        assert!(nested.is_some(), "should have a nested SplitV container");
    }
}
