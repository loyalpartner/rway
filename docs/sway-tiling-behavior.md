# Sway Tiling Window Management Behavior Specification

> Based on analysis of swaywm/sway source code (master branch) and i3 user guide.
> This document describes the **precise behavioral semantics** that rway must implement
> for Sway compatibility. Code references point to the swaywm/sway repository.

---

## 1. Tree Data Structure

### 1.1 Node Hierarchy

```
Root
  +-- Output (physical monitor)
       +-- Workspace
            +-- Tiling children (list of Container)
            +-- Floating children (list of Container)
            +-- [fullscreen container, if any]
```

### 1.2 Container Types

A `Container` is either:
- **Leaf (view)**: holds exactly one application window (`view != NULL`)
- **Split (branch)**: holds a list of child containers (`children` list, `view == NULL`)

### 1.3 Layout Enum

```c
enum sway_container_layout {
    L_NONE,     // unspecified
    L_HORIZ,    // horizontal split (splith) -- children side by side
    L_VERT,     // vertical split (splitv) -- children stacked vertically
    L_STACKED,  // stacking layout -- only focused child visible, title bars stacked
    L_TABBED,   // tabbed layout -- only focused child visible, title tabs horizontal
};
```

### 1.4 Parallel Direction Mapping

The `is_parallel()` helper determines if a layout orientation matches a movement direction:

| Layout         | Parallel Directions |
|----------------|---------------------|
| `L_HORIZ`      | LEFT, RIGHT         |
| `L_TABBED`     | LEFT, RIGHT         |
| `L_VERT`       | UP, DOWN            |
| `L_STACKED`    | UP, DOWN            |

This mapping is fundamental to navigation, movement, and resize algorithms.

---

## 2. Window Insertion

Source: `sway/tree/view.c` (`view_map()`), `sway/tree/container.c`, `sway/tree/workspace.c`

### 2.1 Insertion Algorithm

When a new window (view) opens:

1. **Create container**: `container_create(view)` allocates a new leaf container.
2. **Select workspace**: `select_workspace(view)` determines target workspace by checking:
   - Window assignment rules (`for_window` criteria matching `CT_ASSIGN_WORKSPACE`)
   - PID-based launcher context (inheriting parent process's workspace)
   - Currently focused workspace (fallback)
3. **Find insertion point**: Examine the focused node on the target workspace:
   - If focused node is a **tiling container**: use it as `target_sibling`
   - If focused node is a **floating container**: find the last focused tiling container via `seat_get_focus_inactive_tiling()`
   - If no tiling container exists: add directly to workspace
4. **Insert**:
   - If `target_sibling` exists: `container_add_sibling(target_sibling, new_container, after=1)` -- inserts **after** the focused container
   - If no target: `workspace_add_tiling(workspace, new_container)` -- appends to workspace children

### 2.2 Effect of `workspace_layout`

When `workspace_layout` is set to a non-default value (stacking/tabbed), `workspace_add_tiling()` wraps the new container:

```
if config->default_layout != L_NONE:
    new_container = container_split(new_container, config->default_layout)
```

This creates an intermediate split container with the configured layout, so all top-level workspace children share the specified layout.

### 2.3 Effect of `split` Command on Insertion

The `split` command (see Section 6) wraps the focused container in a new split container with the specified orientation. The next window that opens becomes a **sibling inside this split container**, because the focused leaf is now inside the split container, and `container_add_sibling()` inserts the new window after it.

Example:
```
Before split v:    [A]  [B*]  [C]       (parent: H)
After split v:     [A]  [  ]  [C]       (parent: H)
                        [B*]             (new V container)
New window opens:  [A]  [  ]  [C]       (parent: H)
                        [B ]             (V container)
                        [D*]             (inserted after B)
```

### 2.4 Size Fraction Assignment

New children receive `width_fraction = 0` (or `height_fraction = 0`). During the next `arrange()` call, the layout algorithm assigns them an equal share of existing space (see Section 8).

---

## 3. Focus Navigation (focus left/right/up/down)

Source: `sway/commands/focus.c` (`node_get_in_direction_tiling()`)

### 3.1 Algorithm: `node_get_in_direction_tiling`

```
Input: container, seat, direction, descend
Output: target node to focus (or NULL)

1. Initialize:
   - wrap_candidate = NULL
   - current = container

2. Loop while current != NULL:
   a. If current is FULLSCREEN_GLOBAL: return NULL (blocked)
   b. If current is FULLSCREEN_WORKSPACE: try cross-output, return

   c. Get parent_layout = container_parent_layout(current)
   d. Check if parent_layout is PARALLEL to direction:

      - If NOT parallel:
        - current = current.parent (ascend)
        - continue loop

      - If parallel:
        - siblings = container_get_siblings(current)
        - index = current's index in siblings
        - offset = -1 for LEFT/UP, +1 for RIGHT/DOWN
        - desired = index + offset

        - If desired is VALID (0 <= desired < siblings.length):
          - target = siblings[desired]
          - If descend=true:
              return seat_get_focus_inactive_view(seat, target)
            Else:
              return target  (for "focus sibling" variant)

        - If desired is OUT OF BOUNDS:
          - Store wrap_candidate = siblings[opposite_end]
          - If config.focus_wrapping == WRAP_FORCE:
              return seat_get_focus_inactive_view(seat, wrap_candidate)
          - current = current.parent (ascend)
          - continue loop

3. After exhausting tree (current == NULL):
   - Try cross-output: output_get_in_direction()
   - If adjacent output found AND config.focus_wrapping != WRAP_WORKSPACE:
       return node on adjacent output
   - Else if wrap_candidate exists:
       return seat_get_focus_inactive_view(seat, wrap_candidate)
   - Else: return NULL
```

### 3.2 Descend Behavior

When `descend = true` (default), after finding the target container, the algorithm calls `seat_get_focus_inactive_view()` to find the **deepest previously-focused leaf** inside that container. This preserves focus history -- you return to where you were last focused inside that container.

When `descend = false` (the `focus sibling` variant), focus lands on the container itself rather than descending into it.

### 3.3 Focus Wrapping Modes

| Mode | Behavior |
|------|----------|
| `yes` (default) | When reaching container edge, ascend to parent and try there. If no parent, try cross-output. If no output, wrap to opposite end of the outermost container. |
| `no` | When reaching container edge, ascend to parent and try there. If no parent, try cross-output. If no output, do nothing (focus stays). |
| `force` | Immediately wrap to opposite end of the CURRENT container when reaching its edge. Does NOT ascend to parent first. |
| `workspace` | Like `yes`, but wraps within the workspace boundary. Does NOT cross to other outputs. |

### 3.4 L_TABBED / L_STACKED Navigation

Tabbed and stacked layouts are treated as:
- `L_TABBED` -> parallel to LEFT/RIGHT (like `L_HORIZ`)
- `L_STACKED` -> parallel to UP/DOWN (like `L_VERT`)

So `focus left/right` cycles through tabs, and `focus up/down` cycles through stacked windows.

### 3.5 Floating Navigation

For floating windows, navigation uses **geometric distance**:
```
For each floating sibling on the workspace:
    dx = sibling.center_x - current.center_x
    dy = sibling.center_y - current.center_y
    distance = (horizontal direction) ? dx : dy
    if distance > 0 in correct direction AND distance < closest:
        closest = sibling
```

---

## 4. Window Movement (move left/right/up/down)

Source: `sway/commands/move.c` (`container_move_in_direction()`, `container_move_to_container_from_direction()`)

### 4.1 Algorithm: `container_move_in_direction`

```
Input: container, direction, move_amount (unused for tiled)
Output: success/failure

Phase 1 -- Fullscreen handling:
  - FULLSCREEN_GLOBAL: return false
  - FULLSCREEN_WORKSPACE: move to next output
  - FULLSCREEN_NONE: proceed

Phase 2 -- Find parallel ancestor:
  offset = -1 for LEFT/UP, +1 for RIGHT/DOWN
  ancestor = NULL
  current = container
  wrapped = false

  Loop:
    - Block if current is fullscreen or floating
    - parent_layout = container_parent_layout(current)

    - If NOT parallel:
      - If no parent (at workspace level):
          workspace_wrap_children(workspace)  // wrap all tiling children in a container
          set workspace layout to L_HORIZ (for LEFT/RIGHT) or L_VERT (for UP/DOWN)
          reset container dimensions
          wrapped = true
          continue loop
      - Else: current = current.parent, continue

    - If parallel:
      ancestor = current's parent (or workspace)
      break

Phase 3 -- Find target:
  siblings = container_get_siblings(current)
  desired = index_of(current) + offset
  target = (desired in bounds) ? siblings[desired] : NULL

Phase 4 -- Execute move:
  Case A -- Simple move (current == container, target exists):
    container_move_to_container_from_direction(container, target, direction)

  Case B -- No target, no parent:
    container_move_to_next_output(container, direction)

  Case C -- No target, has parent:
    Ascend: current = ancestor, restart loop

  Case D -- Complex move (current != container, target exists):
    container_move_to_container_from_direction(container, target, direction)

  Case E -- Complex move (current != container, no target):
    If not wrapped AND parent has single child:
      move to next output
    Else:
      Promote: insert container into ancestor's parent at (index + offset)
      Reap empty containers

Phase 5 -- Cleanup:
  container_reap_empty(old_parent)
  workspace_squash(workspace)
  Focus the moved container
  Arrange affected workspaces
```

### 4.2 Algorithm: `container_move_to_container_from_direction`

This handles the actual insertion of a moved container into a destination.

```
Input: container, destination, direction

Case 1 -- Destination is a LEAF (has a view):
  If same parent and same workspace:
    SWAP positions in sibling list (simple exchange)
  Else:
    offset = 0 for LEFT/UP, 1 for RIGHT/DOWN
    index = index_of(destination) + offset
    If destination has parent:
      container_insert_child(destination.parent, container, index)
    Else:
      workspace_insert_tiling(destination.workspace, container, index)
    Reset container width/height fractions to 0
    workspace_squash(workspace)

Case 2 -- Destination is a CONTAINER (no view):
  layout = destination.layout
  Check is_parallel(layout, direction):

  If PARALLEL:
    index = (RIGHT or DOWN) ? 0 : destination.children.length
    container_insert_child(destination, container, index)
    Reset container dimensions/fractions to 0
    workspace_squash(workspace)

  If PERPENDICULAR:
    focus_child = seat_get_active_tiling_child(seat, destination)
    If no children:
      container_add_child(destination, container)
    Else:
      RECURSE: container_move_to_container_from_direction(container, focus_child, direction)
```

### 4.3 Key Behaviors Summary

| Scenario | Behavior |
|----------|----------|
| Move within same container (parallel) | **Swap** with adjacent sibling |
| Move to sibling leaf container | Become sibling of that leaf (same parent) |
| Move to sibling split container (parallel) | Enter container at near edge (index 0 or last) |
| Move to sibling split container (perpendicular) | Recurse into focused child |
| Move past container boundary (no sibling) | Ascend to parent, try there |
| Move past workspace boundary | Move to adjacent output's workspace |
| No parallel ancestor exists | Wrap workspace children, change workspace layout |

### 4.4 Workspace Wrapping

When `container_move_in_direction` cannot find a parallel ancestor at the workspace level, it performs **workspace wrapping**:

1. All tiling children of the workspace are moved into a newly created container
2. The workspace's layout is set to match the movement direction (`L_HORIZ` for LEFT/RIGHT, `L_VERT` for UP/DOWN)
3. The moved container can now be extracted from the wrapper

This ensures movement always has a meaningful result.

---

## 5. Resize

Source: `sway/commands/resize.c`, `sway/input/seatop_resize_tiling.c`

### 5.1 Command Syntax

```
resize grow|shrink width|height [<amount> [px|ppt]]
resize grow|shrink left|right|up|down [<amount> [px|ppt]]
resize set [width] <width> [px|ppt] [height] <height> [px|ppt]
```

### 5.2 Default Values

- Default amount: **10**
- Default unit for tiled: **ppt** (percentage points)
- Default unit for floating: **px** (pixels)

### 5.3 Finding the Resize Parent

`container_find_resize_parent(container, axis)`:

```
Walk up from container through ancestors:
  For each ancestor:
    - Check parent layout matches axis (L_HORIZ for horizontal, L_VERT for vertical)
    - Check parent has multiple children (siblings.length > 1)
    - Check container is not at edge (not first for shrink-left, not last for shrink-right)
  Return first matching ancestor, or NULL
```

### 5.4 PPT Calculation

When using percentage points (ppt):
```
parent = find parent with matching layout orientation
amount_px = parent.dimension * (ppt / 100.0)
```

If no matching parent found, uses workspace dimension instead.

### 5.5 Tiled Resize Algorithm: `container_resize_tiled`

```
Input: container, amount_px

1. Determine resize participants:
   - For AXIS (width/height): ALL siblings participate
   - For DIRECTION (left/right/up/down): only container and ONE adjacent sibling

2. Convert px to fraction:
   amount_fraction = amount_px / parent.child_total_dimension

3. Apply to resized container:
   container.fraction += amount_fraction

4. Distribute inverse among siblings:
   sibling_fraction = amount_fraction / (participant_count - 1)
   For each other participant:
     sibling.fraction -= sibling_fraction

5. Safety check:
   If any container.fraction would produce dimension < MIN_SANE_W/H:
     Abort resize (no change applied)

6. Trigger arrange_container(parent) to recalculate geometry
```

### 5.6 Mouse Resize (Tiled)

Source: `sway/input/seatop_resize_tiling.c`

**Initiation**: Hovering cursor over a tiled container border shows `col-resize` or `row-resize` cursor. Clicking initiates `seatop_begin_resize_tiling()`.

**Algorithm**:
```
1. Record initial cursor position (ref_lx, ref_ly)
2. Find resize participants:
   - container = the container whose border was clicked
   - sibling = container_get_resize_sibling() based on edge
     - LEFT/TOP edge: sibling at (index - 1)
     - RIGHT/BOTTOM edge: sibling at (index + 1)

3. On pointer motion:
   moved_x = cursor_x - ref_lx
   moved_y = cursor_y - ref_ly

   amount = (edge is LEFT/TOP) ? -moved : +moved
   container_resize_tiled(container, amount)
```

### 5.7 Floating Modifier Mouse Resize

With `floating_modifier` key held:
- **Right click drag** on a tiled window resizes it by adjusting the split ratio
- The resize edge is determined by which quadrant of the window the cursor is in

---

## 6. Split Command

Source: `sway/commands/split.c`, `sway/tree/container.c` (`container_split()`)

### 6.1 Command Variants

| Command | Effect |
|---------|--------|
| `split h` / `split horizontal` | Set horizontal split layout |
| `split v` / `split vertical` | Set vertical split layout |
| `split toggle` / `split t` | Toggle: if parent is V, switch to H; otherwise V |
| `split none` / `split n` | Undo split (flatten if only child) |

### 6.2 `container_split(child, layout)` Algorithm

```
1. Shortcut check:
   If child has exactly 1 sibling (including itself) in parent
   AND parent layout is L_HORIZ or L_VERT:
     Just change parent.layout = layout
     Return child (no new container created)

2. Create new container:
   new_parent = container_create(NULL)  // branch node, no view
   Copy child's geometry to new_parent (x, y, width, height, fractions)
   Set new_parent.layout = layout

3. Replace in tree:
   container_replace(child, new_parent)  // new_parent takes child's position
   container_add_child(new_parent, child) // child becomes new_parent's only child

4. Transfer focus if needed
```

### 6.3 `container_squash(container)` -- Redundant Node Removal

After tree operations, redundant intermediate containers are cleaned up:

```
Conditions for squashing a parent-child pair:
  - Parent layout is L_HORIZ or L_VERT
  - Child layout is L_HORIZ or L_VERT
  - Parent and child layouts are NOT parallel (different orientations)
  - Grandparent layout IS parallel to child layout

Action:
  Promote all grandchildren from child to parent's position
  Destroy the empty child container
```

### 6.4 `split none` (Unsplit)

Reverses a split by flattening: if the focused container is the only child of a split container, the split container is removed and the child takes its place in the grandparent.

---

## 7. Layout Command

Source: `sway/commands/layout.c`

### 7.1 Direct Layout Commands

| Command | Effect |
|---------|--------|
| `layout splith` | Set container layout to `L_HORIZ` |
| `layout splitv` | Set container layout to `L_VERT` |
| `layout tabbed` | Set container layout to `L_TABBED` |
| `layout stacking` | Set container layout to `L_STACKED` |
| `layout default` | Determined by `default_orientation` config |

### 7.2 Toggle Behavior

**`layout toggle` (no args)**:
```
If current layout is L_HORIZ -> L_VERT
If current layout is L_VERT -> L_HORIZ
If other (tabbed/stacking) -> restore saved split layout
  If no saved layout -> use config.default_orientation
  If default_orientation is auto -> use output dimensions (vertical if taller, else horizontal)
```

**`layout toggle split`**: Same as `layout toggle` (only cycles between splith and splitv).

**`layout toggle all`**: Cycles through all layouts:
```
L_HORIZ -> L_VERT -> L_STACKED -> L_TABBED -> L_HORIZ
```

**`layout toggle <layout1> <layout2> ...`**: Cycles through the specified layouts in order:
```
Find current layout in argument list
Next layout = argument[(current_index + 1) % argument_count]
```

### 7.3 Single-Child Flattening

When a layout change results in a container with only one child, the code may flatten the tree by promoting the child and destroying the intermediate container.

---

## 8. Size Distribution Algorithm (Arrange)

Source: `sway/tree/arrange.c`

### 8.1 Width/Height Fractions

Each container has `width_fraction` and `height_fraction` fields (doubles). These represent the **proportion** of available space the container should receive.

- Sum of all children's fractions in a container is normalized to **1.0**
- New children start with fraction **<= 0** (unassigned)
- Fractions are assigned during `arrange()` calls

### 8.2 `apply_horiz_layout` / `apply_vert_layout` Algorithm

```
Phase 1 -- Count and sum:
  total_fraction = 0
  new_children_count = 0
  For each child:
    If child.fraction <= 0: new_children_count++
    Else: total_fraction += child.fraction

Phase 2 -- Assign fractions to new children:
  For each child where fraction <= 0:
    If total_fraction <= 0:
      child.fraction = 1.0  // all children are new
    Else if new_children_count < total_children:
      child.fraction = total_fraction / (total_children - new_children_count)
      // new child gets equal share of existing space
    Else:
      child.fraction = total_fraction

Phase 3 -- Normalize:
  total = sum of all fractions
  For each child:
    child.fraction /= total  // now sum == 1.0

Phase 4 -- Calculate gaps:
  inner_gap = workspace.gaps.inner  (zero if any ancestor is tabbed/stacked)
  total_gap = min(
    inner_gap * (children_count - 1),
    max(0, parent_dimension - MIN_SANE_DIMENSION * children_count)
  )
  inner_gap = floor(total_gap / (children_count - 1))

Phase 5 -- Position and size children:
  child_total_dimension = parent_dimension - total_gap
  current_pos = parent.x  (or parent.y for vertical)

  For each child (index i):
    child.pos = current_pos
    child.cross_pos = parent.cross_pos  // full extent in cross direction
    child.cross_dimension = parent.cross_dimension

    If i == last child:
      child.dimension = parent.pos + parent.dimension - child.pos  // absorb rounding error
    Else:
      child.dimension = round(child.fraction * child_total_dimension)

    current_pos += child.dimension + inner_gap
```

### 8.3 Tabbed / Stacked Layout

For `L_TABBED` and `L_STACKED`, all children occupy the **same position and size** (overlapping), with title bars providing navigation:

- **Tabbed**: Single row of title tabs at top, all children at same coords below tabs
- **Stacked**: Vertically stacked title bars at top, all children at same coords below all titles

```
For tabbed:
  title_height = one title bar height
  Each child: x = parent.x, y = parent.y + title_height
              width = parent.width, height = parent.height - title_height

For stacked:
  total_title_height = title_bar_height * children_count
  Each child: x = parent.x, y = parent.y + total_title_height
              width = parent.width, height = parent.height - total_title_height
```

---

## 9. Focus Parent / Focus Child

Source: `sway/commands/focus.c`

### 9.1 `focus parent`

```
1. Get current focused container
2. If container is in FULLSCREEN mode: do nothing (blocked)
3. parent = node_get_parent(container.node)
4. If parent exists: seat_set_focus(seat, parent)
```

**Effect**: Focus moves up one level in the tree. The focused node is now a branch container, not a leaf. Subsequent operations (move, layout, split) will affect this branch and all its descendants.

### 9.2 `focus child`

```
1. Get current focused node
2. child = seat_get_active_tiling_child(seat, node)
3. If child exists: seat_set_focus(seat, child)
```

**Effect**: Focus moves down one level to the most recently focused child within the current container. Can be called repeatedly to descend to a leaf.

### 9.3 Use Pattern

```
focus parent  ->  focus parent  ->  [operation on whole subtree]  ->  focus child  ->  focus child
```

This pattern allows selecting a larger region of the tree, performing an operation (like move or layout change), then returning focus to a specific window.

---

## 10. Configuration Options Affecting Tiling

### 10.1 `default_orientation`

Controls the initial split direction for new containers:
- `horizontal`: New containers default to `L_HORIZ`
- `vertical`: New containers default to `L_VERT`
- `auto` (default): Choose based on output dimensions -- `L_VERT` if output is taller than wide, `L_HORIZ` otherwise

### 10.2 `workspace_layout`

Controls the initial layout when a workspace is populated:
- `default`: Use `default_orientation` for the first split
- `stacking`: New workspace containers use `L_STACKED`
- `tabbed`: New workspace containers use `L_TABBED`

### 10.3 `focus_wrapping`

Controls focus navigation boundary behavior:
- `yes` (default): Wrap to opposite edge after exhausting tree, allow cross-output
- `no`: Stop at boundary, no wrapping
- `force`: Wrap immediately at current container boundary
- `workspace`: Wrap within workspace, no cross-output

### 10.4 `smart_gaps`

When enabled, gaps are only applied when a workspace has more than one visible child.

### 10.5 `smart_borders`

When enabled, borders are only drawn when there are multiple visible children.

### 10.6 `tiling_drag`

When enabled, allows dragging tiled containers with the mouse to rearrange them.

---

## 11. Container Lifecycle

### 11.1 Creation

`container_create(view)`:
- If `view != NULL`: creates a leaf container wrapping the view
- If `view == NULL`: creates a branch container (for split containers)
- Initializes scene tree (title bar, borders)

### 11.2 Destruction

`container_begin_destroy(container)`:
- Mark as destroying
- Emit destroy signal
- Clear fullscreen state
- Detach from parent/workspace

`container_reap_empty(container)`:
- Recursively destroys empty (childless) non-view containers
- Climbs up parent chain destroying empties
- Triggers workspace destruction if workspace becomes empty

### 11.3 Squashing

After any tree modification (insert, remove, move), `workspace_squash()` is called to clean up redundant intermediate containers (see Section 6.3).

---

## 12. Cross-Output Navigation and Movement

### 12.1 Focus Across Outputs

When `node_get_in_direction_tiling` exhausts the tree without finding a target:
1. Call `output_get_in_direction(current_output, direction)` to find adjacent output
2. If found and `focus_wrapping != WRAP_WORKSPACE`: focus node on that output
3. Uses the most recently focused window on the target output's active workspace

### 12.2 Move Across Outputs

When `container_move_in_direction` has no target sibling and no further parent:
1. Call `container_move_to_next_output(container, direction)`
2. This finds the adjacent output via `output_get_in_direction()`
3. Moves container to that output's active workspace
4. Container is inserted as tiling child on the new workspace

---

## 13. Floating Containers (Brief)

While this document focuses on tiling, key floating interactions:

- `toggle_floating`: Converts between tiling and floating
- Floating containers are **not part of the tiling tree** -- they have separate lists per workspace
- `floating_modifier` key enables move (left click) and resize (right click) of floating windows
- `focus mode_toggle` switches focus between the tiling layer and floating layer
- When a floating window is focused, directional focus uses geometric distance rather than tree structure

---

## 14. Fullscreen Interactions

- `FULLSCREEN_WORKSPACE`: Container fills entire workspace. Blocks tiling focus navigation within that workspace but allows cross-output movement.
- `FULLSCREEN_GLOBAL`: Container fills entire root (all outputs). Blocks all focus navigation and movement.
- `focus parent` is blocked during fullscreen.
- `move` during `FULLSCREEN_WORKSPACE` delegates to output movement.

---

## Summary of Key Algorithms for rway Implementation

| Operation | Core Function | Key Behavior |
|-----------|--------------|--------------|
| Insert window | `view_map` | After focused sibling, equal fraction |
| Focus direction | `node_get_in_direction_tiling` | Walk up tree for parallel parent, descend into target |
| Move direction | `container_move_in_direction` | Walk up for parallel parent, swap/reparent/promote |
| Resize | `container_resize_tiled` | Adjust fractions, distribute inverse to siblings |
| Split | `container_split` | Wrap in new parent OR change existing parent layout |
| Layout | `cmd_layout` | Set layout, toggle cycles, flatten single-child |
| Arrange | `apply_horiz/vert_layout` | Normalize fractions, distribute proportionally |
| Squash | `container_squash` | Remove redundant intermediate containers |
