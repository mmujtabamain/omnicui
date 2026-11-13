# Taskscape in GPUI

The native port of Taskscape (`../tauri/`), in GPUI, Zed's Rust UI framework (the `gpui` crate,
0.2.2, from crates.io). It links `taskscape_common` (`../tauri/common`) for storage, settings and
the hotkey catalog, so the data and its rules are the Tauri app's own. There is no web interface
to load: the window, Settings, every sheet and the palette are drawn in the window from its first
frame (WHY.md).

```sh
cargo run --bin taskscape-gpui              # the main window, on ~/.taskscape
cargo run --bin taskscape-gpui -- --capture # the capture bar, as a pop-up window
```

On Linux it needs the X11, xkbcommon, fontconfig and Vulkan libraries GPUI links, and a Vulkan
driver (with no GPU, Mesa's lavapipe). The omnicui target that checks it is
`../omnicui/targets/gpui` (its `setup.sh` builds both).

| file | what |
|---|---|
| `src/workspace.rs` | the window's state and actions: data from the store, panes, rail, undo, what is open over it |
| `src/pane.rs`, `inspector.rs`, `titlebar.rs`, `sheet.rs` (sheets, palette, toast), `filter.rs`, `trash.rs`, `settings.rs`, `capture.rs` | the surfaces |
| `src/keys.rs` | actions, and the keymap generated from the hotkey catalog (with the user's overrides) |
| `src/text_field.rs` | the text field: GPUI's text system and the platform's text input |
| `src/theme.rs`, `src/motion.rs` | DESIGN.md's tokens, and its durations and easings on one clock |
| `src/parts.rs` | part tags (GPUI has no accessibility tree): a test tool reads them, the app doesn't |
| `src/model.rs` | the tree, completion, copy, search and the filter view, without drawing |

What is not ported yet, and why, is in `GAPS-P1.txt`.
