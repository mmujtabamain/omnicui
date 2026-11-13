# P1: the GPUI port and its target

13 Nov 2026, 06:19–09:27 UTC, one run, with messages from the coordinator on the way (chat
labels, a status report, the disk). 1,206 transcript lines. Its own record:
`../gpui/PROGRESS.md`, `../gpui/GAPS-P1.txt`.

**What it did.**
- **Feasibility first, within 30 minutes:** gpui 0.2.2 pinned from crates.io; a real window
  under Xvfb with Mesa's lavapipe; real XTEST input and screenshots read back from the X server.
  It chose that over GPUI's test platform, which draws no pixels.
- **A native app linked to taskscape_common**, with every surface in SPEC-PRODUCT.md:
  - Settings, sheets and the palette drawn in the window, ready on the first frame;
  - a keymap from the hotkey catalog, app menus, focus handles, a virtualized list, a
    client-side title bar, and DESIGN.md's motion.
- **One long-lived adapter**, set up once; start-up 0.2–0.3 s, a mount 0.2–0.5 s (fixtures in
  `/dev/shm`).

**Results.** Three full checks, gpui and tauri: 344 pass with 10 fails (tauri's, its harness
rebuilt by P0 during the run), 352 pass, then **353 pass, 0 fail, 2 warnings**, both narrowed or
fixed after in focused runs. 194 gpui tests in 3 min 51 s, against 13 min 41 s on tauri.

**What it decided.** `list()` instead of `uniform_list` (two row heights), a filled done check,
and no copy of the webview's ~50 ms blank before the inspector rises (WHY.md); all accepted by P0.
Scale is drawn as a growing card, since GPUI can't transform a box.

**What went wrong**:
- a GPUI app that quit when its last window closed;
- key bindings that clicked focused switches twice;
- a double hover that panicked GPUI and broke three runs;
- a shared run folder that another agent's run replaced (all its runs use `--out` since);
- a build folder that reached 11 GB against a 10 GB limit (brought down to 1.9 GB);
- PROGRESS.md entries timed after the run ended (09:40, 10:10).

**Not ported yet:** the global ⌘↩ shortcut and the menu-bar icon (GPUI has neither), screenshot
capture off macOS, the lightbox, drag and drop, resizing, import/export, tab rename in place,
search scope, the notes toolbar and @mentions, and the done check's drawn tick.
