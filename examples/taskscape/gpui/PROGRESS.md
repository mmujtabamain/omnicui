# P1 progress (the GPUI port)

Times are UTC, 13 Nov 2026.

- 06:10 Started from prompts/P1.md. Read WHY.md, SPEC-PRODUCT.md, DESIGN.md, GAPS-P0.txt, the
  contracts, omnicui's README and SPEC (§1, §4.2–4.4, §4.8, §5), the todo example's PROCESS.md,
  the mobile review, and the tauri target (adapter, server).
- 06:40 Feasibility: gpui 0.2.2 from crates.io pinned; a real window renders under Xvfb with
  Mesa's lavapipe (blade on Vulkan, llvmpipe); XTEST input reaches GPUI's own X11 path; XGetImage
  reads the window back. (libc pinned to 0.2.186: xattr 0.2.3, under gpui's http client, doesn't
  build with 0.2.191.) The X11, xkbcommon and fontconfig dev packages were unpacked without root, with
  PKG_CONFIG_PATH, LIBRARY_PATH and LD_LIBRARY_PATH pointing at them.
- 07:05 The app compiles: workspace (state and actions), pane, inspector, title bar, sheets,
  palette, toast, filter, Trash, Settings, capture bar, text field, part tags, motion clock.
- 07:12 The adapter mounts the pane and reads it; first screenshot. Mount ~1.2 s (0.8 s of it the
  fixture loader), the first one ~7 s.
- 07:30 Every contract passes its behaviour on gpui, one component at a time: pane 31/31,
  inspector 24/24, title bar 18/18, sheet 16/16, palette 15/15, toast 7/7, Trash 7/7, filter
  7/7, Settings 26 pass and 14 not measured (as on tauri), capture 11/11, window 14/14. Two
  fixes on the way: GPUI's X11 platform ends the app when its last window closes (the adapter
  keeps an 8×8 window in a far corner); GPUI clicks a focused element itself on Enter/Space key
  up, so a key binding that also pressed it toggled switches twice.
- 07:22 Fixtures are written to /dev/shm when there is one: a mount went from 0.8–7 s (SQLite
  commits on a busy disk) to ~0.3 s. Runs go to their own --out folder: the project's default
  run folder is shared with P2's runs, and one replaced mine mid-run.
- 07:35 Cross-design runs (check gpui tauri <component>) started. Pane: colours 0 % and layout
  1.8–4.4 % everywhere but one colour warning (the done lamp in the long list, dark). Inspector:
  the rise traced from its first frame after the press, not from rest (fixed: two frames before
  the steps).
- 07:55 Fixed from the cross-design runs: the backdrop behind sheets and the palette (black 30 %,
  160 ms fade, from the app's Backdrop), sheet buttons (the app's KeyAction), the filter panel
  redrawn to the app's layout, the toast. A double `.hover()` panicked GPUI in a debug build and
  broke three runs (fixed).
- 08:12 Full check 1 (gpui + tauri): gpui 176 pass, 0 fail, 14 not measured, 4 not applicable;
  4m28s for gpui. The 10 fails were tauri's, its harness rebuilt during the run.
- 08:22 Clips and traces on the app's held clock, a frame captured only once presented: the
  project menu's clip went from 71–83 ms to 20 ms, the opacity traces to 0.6–17 ms. Full check 2.
- 08:45 Full check 2: 352 pass, 0 fail, 1 not advertised (P0's new capture test, `do: summon`:
  done since, the bar hides on Escape and comes back with its draft), 3 warnings.
- 09:15 Full check 3 (the last): 353 pass · 0 fail · 0 broken · 0 not advertised · 26 not
  measured · 9 not applicable · 2 warnings; gpui 176 pass, 3m51s.
- 09:40 After it, in focused runs: the capture card redrawn from the tray's (layout 9.2 %, its
  notes open at once as the tray's do now), shadows that fade in with their box (project menu
  clip 92 → 26–57 ms), stale X displays reclaimed and Xvfb tied to the adapter's life. Sheet,
  palette, window, title bar, capture pass with no warning against tauri.
- 10:10 The build folder was 11 GB: dev profile now line tables only, no incremental cache;
  rebuilt from scratch in 11m29s to 1.9 GB. Pane 31/31 and capture 11/11 again after it.
