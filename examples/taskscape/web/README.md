# Taskscape for the web

The Tauri app's frontend (`../tauri/main/src`, `../tauri/common-ui/src`) in a browser, and a Rust
server (`server/`) that answers it over HTTP with `taskscape_common`'s `Store`, the storage the
desktop apps use. The product and the design are the same; what a page does differently is the
web's own (below). Why it exists: `../WHY.md`.

```sh
scripts/build.sh      # npm ci, the icon subset, the frontend into dist/, the server (cargo, release)
scripts/serve.sh      # http://127.0.0.1:7430 on ~/.taskscape (PORT=… for another port)
```

Needs Node 22, Rust and Python 3 (the icon subset runs in `.venv/`, made on the first build).

## How it is built

- **Nothing is copied.** Vite compiles the shared sources where they are (`vite.config.ts`).
  Modules that talk to Tauri or draw desktop chrome are swapped for the web's versions in
  `src/app/` (`OVERRIDES`; each file's header says what differs). Two shared modules get a few
  exact lines changed (`PATCHES`): the round check marks a task done (the design review's
  direction), and the empty list names the new-task shortcut as bound.
- **Tauri IPC → HTTP.** `src/app/api.ts` has the same names and types as the app's `api.ts`;
  each command is a request under `/api`. Tauri's events between windows are a page-local bus
  that also reaches the app's other tabs (`src/platform/events.ts`).
- **The server** mounts the common crate's `data_router` (lists, tasks) and adds what the
  commands need (`server/src/api.rs`): projects, the Trash, notes, uploads, settings, shortcuts
  (with web defaults, `server/src/hotkeys.rs`), list import and export. It serves `dist/`, with
  `index.html` for any deep link. A failed API request is logged; with `TASKSCAPE_ALERT_URL`
  set, a server error is also posted there (`server/src/alerts.rs`).

## What the browser does differently

- **Addresses**: `/lists/<list>`, `/lists/<list>/tasks/<task>`, `/projects/<project>`. A list or
  project change is a history entry (Back and Forward work); the tab's title follows the
  selection.
- **No window chrome**: the bar is the page's header. Settings is a dialog in the page, and so
  is quick capture (⌘↩ / Ctrl+↩, or the bolt in the header). Nothing opens a window.
- **Shortcuts** leave the browser's own chords alone: new task Alt+N, new list Alt+Shift+N,
  list 1–9 Alt+1–9; the rest are the catalog's, with Ctrl as ⌘ off a Mac.
- **Files** are uploaded (picked or dropped on Attachments) and opened or downloaded in the
  browser; a list is exported as a download and imported from a picked file. A page can't take
  screenshots, so there are none.
- **Narrow windows** (under 960 px): the details drawer goes over the pane, with a scrim.

The omnicui target is `../omnicui/targets/web` (`OMNICUI=1 scripts/build.sh` adds its page,
`omnicui.html`, and the server's test hooks). Progress: `PROGRESS.md`; the report: `GAPS-P2.txt`.
