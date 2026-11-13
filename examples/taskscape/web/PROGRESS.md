# P2 progress

Times are UTC, 13 Nov 2026.

- 06:19 Started from prompts/P2.md. Read WHY.md, P0's SPEC-PRODUCT.md, DESIGN.md, GAPS-P0.txt,
  platforms.yaml, every contract, the tauri target (adapter, harness, server, registry), and the
  Tauri app's frontend (App, api.ts, the stores, the modules that import Tauri).
- 06:25 Decided the shape: the web app is the Tauri frontend compiled from where it is
  (tauri/main/src, tauri/common-ui/src), with the modules that talk to Tauri or draw desktop
  chrome swapped for web versions (web/vite.config.ts, OVERRIDES; src/app/). Tauri's event API is
  a page-local bus (+ BroadcastChannel between tabs).
- 06:30 web/server: axum over taskscape_common's Store: data_router + the routes the commands
  need, the built frontend with a deep-link fallback, a failure log/alert layer; the test hooks
  (fixture loader, domain events from the API's requests) behind a cargo feature.
- 06:45 The first page renders at 1180x760 with the standard fixture; the address follows the
  list (/lists/<id>), the title follows the selection.
- 06:50 The target (omnicui/targets/web, from the reference web template, its adapter from the
  tauri target's): setup.sh builds the app with omnicui.html and the server with its test hooks
  once; start-up 1.1–1.6 s. The registry follows the tauri app's surface for surface.
- 06:53 pane 29/31 on the first run; the round check (design review) and the empty-state shortcut
  needed two exact patches of shared modules (vite.config.ts PATCHES). Then 30/31: the empty text
  hardcodes ⌘ (GAPS-P2.txt 1).
- 06:58 inspector 24/24, titlebar 16 + 2 n/a, window 13 + 1 n/a, sheet 16/16, palette 15/15,
  filter 7/7, trash 6 + 1 n/a, toast 7/7. Settings 24/27 + 12 not measured: the three tests that
  send ⌘ chords or read ⌘ glyphs (GAPS-P2.txt 1). Escape with no focus didn't close Settings
  (my bug, fixed). Screenshots: first page, deep link, narrow layout.
- 07:01 Full check 1 (web and tauri, the comparisons) started.
- 07:18 Full check 1, web and tauri: 334 pass · 5 fail · 0 broken · 0 not advertised · 24 not
  measured · 17 not applicable · 0 warnings (web 157 pass, 5 fail; 79 comparisons, none over its
  threshold). web 13 min 19 s, start-up 1.3 s. The fifth fail: the pane's round-check test expects
  no cascade, which the spec and the inspector's test contradict (GAPS-P2.txt 2). Report
  screenshot posted. Settings opens in 25–56 ms (by hand).
- 07:25 GAPS-P2.txt written. Not done: the narrow layout and the web's idioms (URLs, history,
  quick capture, uploads) are under no contract; proposals in GAPS-P2.txt 4, 7, 8.
- 07:40 P0 adopted proposals 4, 7, 8 (b73e6e8). Mounted `taskscape.narrow` (the App at 820×700,
  `drawerScrim` on the scrim) and `taskscape.capture` (the in-page bar alone; it now follows the
  tray's card and is kept alive while hidden). narrow 3/3; capture 8 pass, 3 n/a, 2 visibility
  warnings (GAPS-P2.txt 12).
- 07:58 Full check 2, web and tauri: 350 pass · 0 fail · 0 broken · 0 not advertised · 24 not
  measured · 14 not applicable · 3 platform exceptions · 5 warnings, all in capture: the hidden
  bar's read (12), tauri's blank capture screenshots (14), and my missing icons (OWN 7, fixed;
  `check web tauri capture` then: layout 0.6% / 3.0%).
- 08:20 After P0's 1662f38: the bar answers the capture shortcut itself (the standalone mount
  didn't), and the notes open at once, as the tray's (GAPS-P2.txt 16). capture vs tauri: 20 pass,
  2 n/a, 1 warning (GAPS-P2.txt 15, for P0).
- 08:37 Full check 3 (the last), web and tauri: 351 pass · 0 fail · 0 broken · 0 not advertised ·
  24 not measured · 13 not applicable · 3 platform exceptions · 1 warnings. web 14 min 41 s,
  start-up 1.1 s.
