//! Sway-compatible tiling behavior tests.
//!
//! These tests define the EXACT expected behavior of Sway's tiling engine.
//! Implementation must make ALL these tests pass.

use rway_tiling::*;

// ── Helpers ─────────────────────────────────────────────────────

fn screen() -> Rect {
    Rect::new(0, 0, 1920, 1080)
}

fn make_tree() -> Tree {
    let mut tree = Tree::new();
    let root = tree.root();
    let out = tree.add_node(
        root,
        NodeData::Output {
            name: "eDP-1".into(),
            geometry: screen(),
        },
    );
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

fn window_ids(tree: &Tree) -> Vec<u64> {
    tree.window_geometries()
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

fn children_window_ids(tree: &Tree, container: NodeId) -> Vec<u64> {
    tree.children(container)
        .iter()
        .filter_map(|&id| match &tree.get(id)?.data {
            NodeData::Window { window_id, .. } => Some(*window_id),
            _ => None,
        })
        .collect()
}

fn find_container_with_layout(tree: &Tree, node: NodeId, layout: Layout) -> Option<NodeId> {
    if let Some(n) = tree.get(node) {
        if let NodeData::Container {
            layout: l, ..
        } = &n.data
        {
            if *l == layout {
                return Some(node);
            }
        }
        for i in 0..tree.children(node).len() {
            let child = tree.children(node)[i];
            if let Some(found) = find_container_with_layout(tree, child, layout) {
                return Some(found);
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════
// 1. WINDOW INSERTION
// ═══════════════════════════════════════════════════════════════

#[test]
fn insert_first_window() {
    let mut tree = make_tree();
    let id = tree.insert_window(1);
    assert!(tree.get(id).is_some());
    assert_eq!(tree.focused_window_id(), Some(1));
}

#[test]
fn insert_two_windows_creates_splith_container() {
    // Default split is horizontal
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);

    // Both windows should exist
    let ids = window_ids(&tree);
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));

    // They should be siblings in a SplitH container
    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    assert!(matches!(
        tree.get(container).unwrap().data,
        NodeData::Container {
            layout: Layout::SplitH,
            ..
        }
    ));
    assert_eq!(tree.children(container).len(), 2);
}

#[test]
fn insert_three_same_direction_are_siblings() {
    // Sway: 3 windows with same split → flat siblings, not nested
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3);

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    assert_eq!(
        tree.children(container).len(),
        3,
        "3 windows should be siblings"
    );
}

#[test]
fn insert_after_focused_not_at_end() {
    // Sway: new window inserted AFTER focused, not appended to end
    // [A, B*, C] → insert D → [A, B, D*, C]
    let mut tree = make_tree();
    tree.insert_window(1); // A
    tree.insert_window(2); // B
    tree.insert_window(3); // C - now [A, B, C*]

    // Focus B (index 1)
    tree.focus_window(2);

    // Insert D — should go after B, before C
    tree.insert_window(4); // D

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    let ids = children_window_ids(&tree, container);

    assert_eq!(ids, vec![1, 2, 4, 3], "D should be inserted after B, before C");
}

#[test]
fn insert_equal_distribution() {
    // All siblings should get equal space
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3);

    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();

    assert_eq!(geoms.len(), 3);
    for (_, g) in &geoms {
        assert!(
            g.width > 600 && g.width < 700,
            "each of 3 should be ~640px, got {}",
            g.width
        );
    }
}

// ═══════════════════════════════════════════════════════════════
// 2. SPLIT COMMAND — immediate wrapping
// ═══════════════════════════════════════════════════════════════

#[test]
fn split_wraps_focused_window_immediately() {
    // Sway: `split v` on B in [A | B] → [A | SplitV[B]]
    let mut tree = make_tree();
    tree.insert_window(1); // A
    tree.insert_window(2); // B, focused

    tree.split(Layout::SplitV);

    // B should now be inside a SplitV container
    let ws = tree.focused_workspace().unwrap();
    let top = tree.children(ws)[0]; // SplitH container
    let children = tree.children(top);

    // One child should be a SplitV container containing B
    let splitv = children.iter().find(|&&id| {
        matches!(
            tree.get(id).map(|n| &n.data),
            Some(NodeData::Container {
                layout: Layout::SplitV,
                ..
            })
        )
    });
    assert!(splitv.is_some(), "split v should create a SplitV container wrapping B");
}

#[test]
fn split_then_insert_goes_into_new_container() {
    // [A | B*] → split v → [A | SplitV[B*]] → open C → [A | SplitV[B, C*]]
    let mut tree = make_tree();
    tree.insert_window(1); // A
    tree.insert_window(2); // B, focused

    tree.split(Layout::SplitV);
    tree.insert_window(3); // C

    // C should be below B in SplitV
    let ws = tree.focused_workspace().unwrap();
    let top = tree.children(ws)[0]; // SplitH

    // Find the SplitV container
    let splitv_id = find_container_with_layout(&tree, top, Layout::SplitV)
        .expect("should have SplitV");
    let sv_children = children_window_ids(&tree, splitv_id);
    assert_eq!(sv_children, vec![2, 3], "B and C should be in SplitV");
}

#[test]
fn build_2x2_grid() {
    // Standard Sway workflow for 2x2 grid:
    // ret → A
    // ret → [A | B*]
    // focus left → [A* | B]
    // split v → [SplitV[A*] | B]
    // ret → [SplitV[A, C*] | B]
    // focus right → [SplitV[A, C] | B*]
    // split v → [SplitV[A, C] | SplitV[B*]]
    // ret → [SplitV[A, C] | SplitV[B, D*]]
    let mut tree = make_tree();
    tree.insert_window(1); // A
    tree.insert_window(2); // B*

    tree.move_focus(Direction::Left); // focus A
    tree.split(Layout::SplitV); // wrap A in SplitV
    tree.insert_window(3); // C below A

    tree.move_focus(Direction::Right); // focus B
    tree.split(Layout::SplitV); // wrap B in SplitV
    tree.insert_window(4); // D below B

    // Verify 2x2 layout
    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();

    assert_eq!(geoms.len(), 4, "should have 4 windows");

    // Get geometries by window id
    let g = |id: u64| -> Rect {
        geoms.iter().find(|(wid, _)| *wid == id).unwrap().1
    };

    // A should be top-left, B top-right, C bottom-left, D bottom-right
    assert!(g(1).x < g(2).x, "A should be left of B");
    assert!(g(3).x < g(4).x, "C should be left of D");
    assert!(g(1).y < g(3).y, "A should be above C");
    assert!(g(2).y < g(4).y, "B should be above D");

    // Each window should be approximately quarter-screen
    for id in [1, 2, 3, 4] {
        let geo = g(id);
        assert!(geo.width > 900 && geo.width < 1000, "width ~960, got {}", geo.width);
        assert!(geo.height > 500 && geo.height < 560, "height ~540, got {}", geo.height);
    }
}

// ═══════════════════════════════════════════════════════════════
// 3. MOVE WINDOW
// ═══════════════════════════════════════════════════════════════

#[test]
fn move_swaps_siblings_same_container() {
    // [A, B*, C] → move left → [B*, A, C]
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3);

    tree.focus_window(2); // focus B
    tree.move_window(Direction::Left);

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    let ids = children_window_ids(&tree, container);
    assert_eq!(ids, vec![2, 1, 3], "B should swap with A");
}

#[test]
fn move_at_boundary_returns_false() {
    // [A*, B] → move left → no change (already leftmost)
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.focus_window(1);

    let moved = tree.move_window(Direction::Left);
    assert!(!moved, "can't move further left");
}

#[test]
fn move_down_in_2x2_swaps_within_column() {
    // 2x2 grid: [SplitV[A*, C] | SplitV[B, D]]
    // move A down → [SplitV[C, A*] | SplitV[B, D]]
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.move_focus(Direction::Left);
    tree.split(Layout::SplitV);
    tree.insert_window(3);
    tree.move_focus(Direction::Right);
    tree.split(Layout::SplitV);
    tree.insert_window(4);

    // Focus A (top-left)
    tree.focus_window(1);
    tree.move_window(Direction::Down);

    // A and C should have swapped in the left column
    // Verify via layout positions
    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();

    let g = |id: u64| -> Rect {
        geoms.iter().find(|(wid, _)| *wid == id).unwrap().1
    };

    assert!(g(3).y < g(1).y, "C should be above A after move down");
    // B and D should be unchanged
    assert!(g(2).y < g(4).y, "B should still be above D");
}

#[test]
fn move_right_in_2x2_extracts_window() {
    // Sway behavior: move right extracts the window from its sub-container
    // 2x2: SplitH [ SplitV[A*, C], SplitV[B, D] ]
    // move A right → SplitH [ C, A*, SplitV[B, D] ]
    // A is pulled out of the left column and placed as a sibling at SplitH level
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.move_focus(Direction::Left);
    tree.split(Layout::SplitV);
    tree.insert_window(3);
    tree.move_focus(Direction::Right);
    tree.split(Layout::SplitV);
    tree.insert_window(4);

    tree.focus_window(1); // focus A
    let moved = tree.move_window(Direction::Right);
    assert!(moved, "move right should succeed");

    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();

    let g = |id: u64| -> Rect {
        geoms.iter().find(|(wid, _)| *wid == id).unwrap().1
    };

    // A should now be between C and the right column
    // C should be leftmost (took over the left column alone)
    assert!(g(3).x < g(1).x, "C should be left of A after extraction");
    assert_eq!(geoms.len(), 4, "all 4 windows should still exist");
}

// ═══════════════════════════════════════════════════════════════
// 4. FOCUS NAVIGATION
// ═══════════════════════════════════════════════════════════════

#[test]
fn focus_tracks_keyboard_click() {
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3);

    tree.focus_window(1);
    assert_eq!(tree.focused_window_id(), Some(1));

    tree.focus_window(3);
    assert_eq!(tree.focused_window_id(), Some(3));

    tree.focus_window(2);
    assert_eq!(tree.focused_window_id(), Some(2));
}

#[test]
fn focus_left_right_in_splith() {
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3); // [A, B, C*]

    assert_eq!(tree.focused_window_id(), Some(3));

    tree.move_focus(Direction::Left);
    assert_eq!(tree.focused_window_id(), Some(2));

    tree.move_focus(Direction::Left);
    assert_eq!(tree.focused_window_id(), Some(1));

    tree.move_focus(Direction::Right);
    assert_eq!(tree.focused_window_id(), Some(2));
}

#[test]
fn focus_up_down_in_splitv() {
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.split(Layout::SplitV);
    tree.insert_window(2);
    tree.insert_window(3); // SplitV[A, B, C*]

    assert_eq!(tree.focused_window_id(), Some(3));

    tree.move_focus(Direction::Up);
    assert_eq!(tree.focused_window_id(), Some(2));

    tree.move_focus(Direction::Up);
    assert_eq!(tree.focused_window_id(), Some(1));
}

// ═══════════════════════════════════════════════════════════════
// 5. WINDOW REMOVAL & REDISTRIBUTION
// ═══════════════════════════════════════════════════════════════

#[test]
fn remove_window_redistributes_equally() {
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3);

    tree.remove_window(2);

    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();

    assert_eq!(geoms.len(), 2);
    for (_, g) in &geoms {
        assert_eq!(g.width, 960, "each should be 50% after removing one");
    }
}

#[test]
fn remove_all_windows_cleans_containers() {
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3);

    tree.remove_window(1);
    tree.remove_window(2);
    tree.remove_window(3);

    assert_eq!(tree.window_geometries().len(), 0);
    // Workspace should be empty, no orphan containers
    let ws = tree.focused_workspace().unwrap();
    assert!(tree.children(ws).is_empty(), "workspace should be empty after removing all");
}

#[test]
fn remove_from_2x2_preserves_structure() {
    // Build 2x2: [SplitV[A, C] | SplitV[B, D]]
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.move_focus(Direction::Left);
    tree.split(Layout::SplitV);
    tree.insert_window(3);
    tree.move_focus(Direction::Right);
    tree.split(Layout::SplitV);
    tree.insert_window(4);

    // Remove D → right column should just have B
    tree.remove_window(4);

    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();

    assert_eq!(geoms.len(), 3);

    let g = |id: u64| -> Rect {
        geoms.iter().find(|(wid, _)| *wid == id).unwrap().1
    };

    // Left column: A and C should each be half height
    assert!(g(1).y < g(3).y, "A above C");
    // Right column: B should take full height
    assert_eq!(g(2).height, 1080, "B should take full height after D removed");
}

// ═══════════════════════════════════════════════════════════════
// 6. TABBED / STACKED FOCUS NAVIGATION (INC-1)
// ═══════════════════════════════════════════════════════════════

#[test]
fn focus_left_right_in_tabbed() {
    // Sway: Left/Right navigates between tabs in a Tabbed container
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2); // SplitH[w1, w2*]

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    tree.set_layout(container, Layout::Tabbed).unwrap();

    assert_eq!(tree.focused_window_id(), Some(2));

    let moved = tree.move_focus(Direction::Left);
    assert!(moved, "Left should navigate between tabs in Tabbed container");
    assert_eq!(
        tree.focused_window_id(),
        Some(1),
        "focus should move to window 1 after Left in Tabbed"
    );

    // And Right should go back
    let moved = tree.move_focus(Direction::Right);
    assert!(moved, "Right should navigate between tabs in Tabbed container");
    assert_eq!(
        tree.focused_window_id(),
        Some(2),
        "focus should move back to window 2 after Right in Tabbed"
    );
}

#[test]
fn focus_up_down_in_stacked() {
    // Sway: Up/Down navigates between items in a Stacked container
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2); // SplitH[w1, w2*]

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    tree.set_layout(container, Layout::Stacked).unwrap();

    assert_eq!(tree.focused_window_id(), Some(2));

    let moved = tree.move_focus(Direction::Up);
    assert!(moved, "Up should navigate in Stacked container");
    assert_eq!(
        tree.focused_window_id(),
        Some(1),
        "focus should move to window 1 after Up in Stacked"
    );

    let moved = tree.move_focus(Direction::Down);
    assert!(moved, "Down should navigate in Stacked container");
    assert_eq!(
        tree.focused_window_id(),
        Some(2),
        "focus should move back to window 2 after Down in Stacked"
    );
}

// ═══════════════════════════════════════════════════════════════
// 7. TABBED / STACKED MOVE WINDOW (INC-2)
// ═══════════════════════════════════════════════════════════════

#[test]
fn move_left_in_tabbed_reorders() {
    // Sway: move left in Tabbed reorders tabs
    // Tabbed[w1, w2*] → move left → Tabbed[w2*, w1]
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    tree.set_layout(container, Layout::Tabbed).unwrap();

    assert_eq!(tree.focused_window_id(), Some(2));

    let moved = tree.move_window(Direction::Left);
    assert!(moved, "move Left should work in Tabbed container");

    let ids = children_window_ids(&tree, container);
    assert_eq!(ids, vec![2, 1], "window 2 should move to index 0 in Tabbed");
}

#[test]
fn move_up_in_stacked_reorders() {
    // Sway: move up in Stacked reorders stack
    // Stacked[w1, w2*] → move up → Stacked[w2*, w1]
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    tree.set_layout(container, Layout::Stacked).unwrap();

    assert_eq!(tree.focused_window_id(), Some(2));

    let moved = tree.move_window(Direction::Up);
    assert!(moved, "move Up should work in Stacked container");

    let ids = children_window_ids(&tree, container);
    assert_eq!(ids, vec![2, 1], "window 2 should move to index 0 in Stacked");
}

// ═══════════════════════════════════════════════════════════════
// 8. TABBED / STACKED LAYOUT GEOMETRY (INC-5)
// ═══════════════════════════════════════════════════════════════

#[test]
fn tabbed_all_children_get_geometry() {
    // Sway: ALL children in Tabbed/Stacked get the same geometry as the
    // container area, even non-focused ones (important for IPC GET_TREE).
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3);

    let ws = tree.focused_workspace().unwrap();
    let container = tree.children(ws)[0];
    tree.set_layout(container, Layout::Tabbed).unwrap();

    // Only window 3 is focused
    assert_eq!(tree.focused_window_id(), Some(3));

    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();

    assert_eq!(geoms.len(), 3, "all 3 windows should have geometries");
    for (id, g) in &geoms {
        assert!(
            g.is_valid(),
            "window {} should have valid (non-zero) geometry, got {:?}",
            id,
            g
        );
    }

    // All windows should share the same geometry (overlapping, only focused shown)
    let g = |id: u64| -> Rect {
        geoms.iter().find(|(wid, _)| *wid == id).unwrap().1
    };
    assert_eq!(g(1), g(2), "all Tabbed children should have identical geometry");
    assert_eq!(g(2), g(3), "all Tabbed children should have identical geometry");
}

// ═══════════════════════════════════════════════════════════════
// 9. REMOVE WINDOW FOCUS SYNC (INC-6)
// ═══════════════════════════════════════════════════════════════

#[test]
fn remove_focused_window_syncs_focus_path() {
    // Sway: after removing the focused window, focus transfers to a sibling
    // and the entire focused_child path stays consistent.
    //
    // Build: SplitH [ SplitV[w1, w3*], w2 ]
    let mut tree = make_tree();
    tree.insert_window(1); // A
    tree.insert_window(2); // B → SplitH[A, B*]
    tree.move_focus(Direction::Left); // focus A
    tree.split(Layout::SplitV); // → SplitH[SplitV[A*], B]
    tree.insert_window(3); // C → SplitH[SplitV[A, C*], B]

    assert_eq!(tree.focused_window_id(), Some(3));

    // Remove the focused window (C)
    tree.remove_window(3);

    // Focus should transfer to a valid window
    let focused = tree.focused_window_id();
    assert!(
        focused.is_some(),
        "focus should not be None after removing focused window"
    );

    // Navigation from the new focused window should work correctly
    // If focus is on w1 (in SplitV), moving Right should reach w2
    if focused == Some(1) {
        let moved = tree.move_focus(Direction::Right);
        assert!(moved, "should be able to navigate Right from new focus");
        assert_eq!(tree.focused_window_id(), Some(2));
    }
}

// ═══════════════════════════════════════════════════════════════
// 10. INSERT WINDOW FOCUS SYNC (INC-7)
// ═══════════════════════════════════════════════════════════════

#[test]
fn insert_window_syncs_full_focus_path() {
    // Sway: newly inserted window always gets focus, and the entire
    // focused_child path from workspace to the new window is consistent.
    //
    // Build a nested structure:
    // SplitH[w1, w2*] → focus w1 → split v → SplitH[SplitV[w1*], w2]
    // → insert w3 → SplitH[SplitV[w1, w3*], w2]
    let mut tree = make_tree();
    tree.insert_window(1); // A
    tree.insert_window(2); // B → SplitH[A, B*]

    tree.move_focus(Direction::Left); // focus A
    tree.split(Layout::SplitV); // wrap A → SplitH[SplitV[A*], B]
    tree.insert_window(3); // C → SplitH[SplitV[A, C*], B]

    // Newly inserted window should be focused
    assert_eq!(
        tree.focused_window_id(),
        Some(3),
        "newly inserted window should be focused"
    );

    // Navigation should work from the newly focused window
    // C is in SplitV, moving Up should reach A
    let moved = tree.move_focus(Direction::Up);
    assert!(moved, "should navigate Up from C to A in SplitV");
    assert_eq!(tree.focused_window_id(), Some(1));

    // From A, moving Right should cross to B
    let moved = tree.move_focus(Direction::Right);
    assert!(moved, "should navigate Right from SplitV to B");
    assert_eq!(tree.focused_window_id(), Some(2));
}

// ═══════════════════════════════════════════════════════════════
// 11. FOCUS WRAPPING (INC-3)
// ═══════════════════════════════════════════════════════════════

#[test]
fn focus_wrapping_cycles_at_boundary() {
    // Sway (default focus_wrapping yes): moving past the last child
    // wraps around to the first child, and vice versa.
    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2);
    tree.insert_window(3); // SplitH[w1, w2, w3*]

    assert_eq!(tree.focused_window_id(), Some(3));

    // Move right from the rightmost window should wrap to leftmost
    let moved = tree.move_focus(Direction::Right);
    assert!(moved, "focus should wrap at right boundary");
    assert_eq!(
        tree.focused_window_id(),
        Some(1),
        "focus should wrap to first window"
    );

    // Move left from the leftmost window should wrap to rightmost
    let moved = tree.move_focus(Direction::Left);
    assert!(moved, "focus should wrap at left boundary");
    assert_eq!(
        tree.focused_window_id(),
        Some(3),
        "focus should wrap to last window"
    );
}

// ═══════════════════════════════════════════════════════════════
// 12. RESIZE ANCESTOR TRAVERSAL (INC-4)
// ═══════════════════════════════════════════════════════════════

#[test]
fn resize_finds_ancestor_with_matching_axis() {
    // Sway: resize grow width walks up from the focused window to find
    // the nearest ancestor container whose layout axis matches (SplitH
    // for Width). If the direct parent is SplitV, it skips it.
    //
    // Build: SplitH [ SplitV[A, B], C ]
    let mut tree = make_tree();
    let a_id = tree.insert_window(1); // A
    tree.insert_window(3); // C → SplitH[A, C*]
    tree.move_focus(Direction::Left); // focus A
    tree.split(Layout::SplitV); // → SplitH[SplitV[A*], C]
    tree.insert_window(2); // B → SplitH[SplitV[A, B*], C]

    // Focus A
    tree.focus_window(1);

    // Resize A with Width axis.
    // A's direct parent is SplitV (vertical axis → doesn't match Width).
    // The fix should walk up to SplitH (horizontal → matches Width)
    // and resize the SplitV's proportion within SplitH.
    let resized = tree.resize_container(a_id, ResizeAxis::Width, 10.0);
    assert!(
        resized,
        "resize Width should find SplitH ancestor, not stop at SplitV"
    );
}

/// Sway behavior: moving a window into a sibling CONTAINER enters it,
/// rather than swapping positions with the entire container.
///
/// Layout: SplitH[A, B*, SplitV[SplitH[c,d], SplitH[e,f]]]
/// Move B right: B enters the SplitV, not swaps with it.
#[test]
fn move_into_sibling_container_enters_it() {
    let mut tree = make_tree();

    // Build: SplitH[A, B]
    tree.insert_window(1); // A
    tree.insert_window(2); // B, focused

    // Now focus A and create the right-side structure
    tree.move_focus(Direction::Left); // focus A
    assert_eq!(tree.focused_window_id(), Some(1));

    // We need SplitH[A, B, SplitV[...]]
    // Easier approach: just build SplitH[A, B] then focus B and check move
    // Actually, let's test the simpler case: SplitH[A*, SplitV[B, C]]
    drop(tree);

    let mut tree = make_tree();
    tree.insert_window(1); // A
    tree.insert_window(2); // SplitH[A, B*]
    tree.split(Layout::SplitV); // SplitH[A, SplitV[B]]
    tree.insert_window(3); // SplitH[A, SplitV[B, C*]]

    // Focus A
    tree.move_focus(Direction::Left);
    assert_eq!(tree.focused_window_id(), Some(1));

    // Move A right: target is SplitV[B, C] (a container)
    // Sway enters the container, A becomes first child of SplitV
    let moved = tree.move_window(Direction::Right);
    assert!(moved, "move into sibling container should succeed");

    // A should now be inside the SplitV with B and C
    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();
    assert_eq!(geoms.len(), 3);

    // All windows should be in the right half (SplitV column)
    // since A entered the container rather than swapping
    let a = geoms.iter().find(|(id, _)| *id == 1).unwrap().1;
    let b = geoms.iter().find(|(id, _)| *id == 2).unwrap().1;

    // After entering SplitV, A should have same width as B (both full-width in SplitV)
    // This confirms A entered the container, not swapped with it
    assert_eq!(
        a.width, b.width,
        "A and B should have same width (both in SplitV): a={:?} b={:?}",
        a, b
    );
}

/// Sway behavior: move window past boundary promotes it into the parent
/// container. Move down then up should restore (or nearly restore) layout.
///
/// Layout: SplitH[W1, SplitV[W2, SplitH[W3, W4*]]]
/// Move W4 down: W4 promoted into SplitV → SplitH[W1, SplitV[W2, W3, W4*]]
#[test]
fn move_at_boundary_promotes_to_parent() {
    let mut tree = make_tree();

    // Build: SplitH[W1, SplitV[W2, SplitH[W3, W4*]]]
    tree.insert_window(1);
    tree.split(Layout::SplitV);
    tree.insert_window(2);
    tree.split(Layout::SplitH);
    tree.insert_window(3);
    // Verify: W4(=3) should be the 4th window... wait.
    // insert 1 → SplitH[W1]
    // split V → SplitH[SplitV[W1]]   ... hmm
    // Let me trace: after split(SplitV), W1 is wrapped in SplitV
    // insert 2 → SplitV[W1, W2*]
    // split H → SplitV[W1, SplitH[W2]]
    // insert 3 → SplitH[W2, W3*]
    // So tree = SplitH_root [ SplitV[ W1, SplitH[W2, W3*] ] ]
    // We need 4 windows. Let me insert one more.
    tree.insert_window(4);
    // After insert 4 in SplitH[W2, W3*] → SplitH[W2, W3, W4*]
    // Hmm, that's 3 in a row. Let me rethink.

    // Actually, build the layout manually:
    // 1. Insert W1
    // 2. Insert W2 (now SplitH[W1, W2*])
    // 3. Split V (wrap W2 in SplitV → SplitH[W1, SplitV[W2]])
    // 4. Insert W3 (into SplitV → SplitH[W1, SplitV[W2, W3*]])
    // 5. Split H (wrap W3 in SplitH → SplitH[W1, SplitV[W2, SplitH[W3]]])
    // 6. Insert W4 (into SplitH → SplitH[W1, SplitV[W2, SplitH[W3, W4*]]])
    // That's the layout we want!
    drop(tree);

    let mut tree = make_tree();
    tree.insert_window(1);
    tree.insert_window(2); // SplitH[W1, W2*]
    tree.split(Layout::SplitV); // SplitH[W1, SplitV[W2]]
    tree.insert_window(3); // SplitH[W1, SplitV[W2, W3*]]
    tree.split(Layout::SplitH); // SplitH[W1, SplitV[W2, SplitH[W3]]]
    tree.insert_window(4); // SplitH[W1, SplitV[W2, SplitH[W3, W4*]]]

    assert_eq!(tree.focused_window_id(), Some(4));

    // Move W4 down: should promote W4 into the SplitV as a new sibling
    let moved = tree.move_window(Direction::Down);
    assert!(moved, "move down at boundary should promote to parent");

    // W4 should now be the last child in SplitV[W2, W3, W4*]
    // (SplitH[W3] squashed to just W3)
    assert_eq!(tree.focused_window_id(), Some(4));

    // Compute layout and verify all 4 windows have valid geometry
    let root = tree.root();
    tree.compute_layout(root, screen(), &GapsConfig::default());
    let geoms = tree.window_geometries();
    assert_eq!(geoms.len(), 4, "should still have 4 windows");
    assert!(
        geoms.iter().all(|(_, r)| r.width > 0 && r.height > 0),
        "all windows should have valid geometry"
    );
}
