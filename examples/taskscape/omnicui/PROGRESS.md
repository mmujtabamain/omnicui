# P0 progress

Times are UTC, 12 Nov 2026.

- 19:15 Started from prompts/P0.md. Read omnicui's README and SPEC (§1, §4.2–4.4, §4.8, §5),
  examples/todo (PROCESS.md, the desktop project's contracts and web target) and the mobile
  review, then the Tauri app: AGENTS.md, .llm/, the design review, main/src, common-ui, the tray,
  and the Rust commands.
- 19:30 Decided the components (eleven surfaces), and how the target reaches the app: its real
  frontend built with its own Vite and Tailwind, in Chromium with a macOS user agent, every Tauri
  command through Tauri's mockIPC to a Rust server over taskscape_common's Store.
- 19:30 Fixtures (standard, empty, nolists, long) and the taskscape-fixtures crate that loads
  them through Store into a fresh data folder; its test passes.
- 19:40 The target's server (server/), the app's registry entry points
  (tauri/main/src/omnicui/registry.tsx, tauri/tray/src/omnicui/registry.tsx), the harness, the
  build script and the adapter (from the todo example's web target).
- 19:55 The task pane contract passes on tauri (28 pass, 1 not applicable). Screenshots posted.
- 20:05 Inspector (24/24). React's 300 ms reveal of a suspended lazy component made the note
  editor appear after the page looked settled: the registry renders the lazy editor once before
  mounting.
- 20:10 Title bar (18/18), palette (16/16), sheets (16/16), toast, Trash, filter.
- 20:20 Settings, the capture bar (11/11), the whole window (15/15). platforms.yaml: 18 inputs.
- 20:30 DESIGN.md (tokens as sRGB hex, the window measured at 1180×760) and SPEC-PRODUCT.md.
- 20:38 setup.sh from scratch: 25 s; the adapter answers advertise in about 1 s. Full check 1.
- 20:55 Kept only the part tags in the app's diff (put back format-only hunks). Full check 2:
  177 pass · 0 fail · 12 not measured · 1 not applicable · 0 warnings, 14 min 37 s.
