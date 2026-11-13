# P2: the web port, its server and its target

13 Nov 2026, 06:19–08:28 UTC, in three runs: the build, then two follow-ups from the coordinator
(the narrow window and the capture bar under contract; the capture shortcut). 888 transcript
lines. Its own record: `../web/PROGRESS.md`, `../web/GAPS-P2.txt`.

**What it did.**
- The web app is the Tauri frontend compiled where it stands: `web/vite.config.ts` swaps the
  modules that talk to Tauri or draw desktop chrome for web versions and patches two shared
  files by exact hunks.
- A Rust server over the Store reuses the common crate's `data_router`, serves the app for any
  deep link, logs failures and can alert. Its test hooks are behind a cargo feature.
- Deep links and history, the document title, Settings and quick capture drawn in the page,
  uploads, and a narrow layout with the drawer as an overlay.
- A Playwright target set up once (45 s); start-up 1.1–1.6 s.

**Results.** Three full checks, web and tauri: 334 pass with 5 fails (all in P0's contracts), 350
pass with 5 warnings, then **351 pass, 0 fail, 1 warning** (fixed after by P0's crop of the
capture screenshots). 174 web tests in 14 min 41 s. Settings opens 25–56 ms after its shortcut.

**What it decided or proposed.** No Meta-as-Ctrl or glyph tricks to pass tests expecting the
Mac's ⌘; it proposed named inputs and per-platform text instead, and P0 adopted them. It
proposed the narrow component, web-only history and capture tests (adopted), and
`settingsClosed` (not adopted). It matched the tray's notes, which open at once, rather than the
animation DESIGN.md wrongly described.

**What went wrong**, its own:
- a router bug that broke the first page;
- Escape that didn't close Settings;
- a refused shortcut reported as a server error;
- PROGRESS.md times written ahead of the clock;
- the pane contract not re-run after a patch;
- icons missing from the subset;
- the capture shortcut living outside the bar.

**What it found for others.** Four tests in P0's contracts expecting ⌘ on the web, P0's
round-check test contradicting the spec, and the tauri target's blank capture screenshots.
