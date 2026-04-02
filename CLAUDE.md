# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language

- All code comments and log messages (tracing) must be in **English**.
- Git commit messages in English.
- Respond to the user in Chinese (简体中文).

## Build & Test Commands

```bash
cargo build --workspace                     # Build all 5 crates
cargo test --workspace                      # Run all tests (~342)
cargo test -p rway-tiling                   # Tiling engine only
cargo test -p rway-harness                  # Compatibility tests (unit test mode)
cargo run -p rway-harness -- report         # Full compatibility report
cargo run -p rway-harness -- report window  # Single category report
cargo clippy --workspace -- -D warnings     # Lint (must pass before commit)
```

**Gate checklist (all must pass before commit):**
1. `cargo clippy --workspace -- -D warnings`
2. `cargo test --workspace`
3. `cargo run -p rway-harness -- report` (no regressions)

## Architecture

rway is a Sway-compatible Wayland compositor built on Smithay. Users can reuse existing Sway config files, keybindings, and IPC tools (swaymsg, waybar).

### Crate Dependency Layers

```
Layer 0: rway-tiling (thiserror)     — Pure tiling engine, arena N-ary tree
         rway-config (thiserror)     — Sway config file parser
Layer 1: rway-ipc (calloop, serde)   — i3-ipc binary protocol, Unix socket server
Layer 2: rway (smithay + Layer 0-1)  — Main compositor binary
Layer 3: rway-harness (Layer 0-1)    — Compatibility test framework (does NOT depend on rway)
```

**Rule:** Layer N may only depend on Layer < N. No circular dependencies.

### rway-tiling: All Operations Are `impl Tree` Methods

The tiling engine uses Smithay-style architecture: all operations are methods on `Tree`, not free functions.

```rust
// Correct — method call
self.tiling.insert_window(window_id);
self.tiling.compute_layout(root, available, &gaps);
let geoms = self.tiling.window_geometries();

// Wrong — C-style free function (legacy, do not use)
commands::insert_window(&mut self.tiling, window_id);
```

Key `Tree` method groups (~40 methods total):
- **Workspace:** `add_output()`, `add_workspace()`, `focused_workspace()`, `switch_workspace()`, `workspaces()`
- **Window:** `insert_window()`, `remove_window()`, `focused_window_id()`, `toggle_floating()`, `set_fullscreen()`
- **Navigation:** `move_focus()`, `move_window()`, `focus_parent()`, `focus_child()`
- **Layout:** `split()`, `set_layout()`, `compute_layout()`, `window_geometries()`
- **Error handling:** Methods return `Result<_, TilingError>` for fallible operations

### rway-harness: 8 Test Categories

Tests in `rway-harness/src/categories/`: config, ipc, tiling, window, workspace, input, output, appearance.

## Coding Standards

- **Smithay style:** Behavior as `impl` methods on structs, not free functions.
- **No `unwrap()`** in production code. Use `let Some(...) else { return }`, `?`, or `expect("reason")` for invariants.
- **Visibility:** Use `pub(crate)` for items not needed outside the crate. Never default to `pub`.
- **Error types:** Use `thiserror` derive. The tiling engine has `TilingError` with 9 variants.
- **No `data.clone()` hacks:** When borrowing conflicts occur, extract lightweight decision info to local variables instead of cloning entire enums.
- **Test-first:** RED → GREEN → REFACTOR. Write tests before implementation.
- **Size limits:** Functions < 50 lines, files < 800 lines.

## Sway Compatibility Workflow

1. Read `docs/sway-spec.md` for the feature's Sway behavior specification
2. Write test in `rway-harness/src/categories/` (RED)
3. Implement in the corresponding crate (GREEN)
4. Run gate checklist

## Key Files

| File | Role |
|------|------|
| `rway/src/state.rs` | `RwayState` — compositor master state |
| `rway/src/main.rs` | Startup, backend selection (winit on desktop, udev on TTY) |
| `rway/src/input/keybindings.rs` | Keybinding matching and action dispatch |
| `rway/src/ipc.rs` | IPC message routing |
| `rway-tiling/src/tree.rs` | `Tree` struct + all `impl Tree` methods (tiling core) |
| `rway-tiling/src/error.rs` | `TilingError` enum |
| `rway-config/src/parser.rs` | Sway config parser |
| `rway-config/src/types.rs` | `Config`, `Action`, `Keybinding` types |
| `rway-ipc/src/protocol.rs` | i3-ipc binary protocol codec |
| `docs/sway-spec.md` | Sway feature spec with implementation status tracking |
